mod field_to_operation;
mod postgraphile_introspection;

#[cfg(test)]
#[path = "./test.rs"]
mod test;
use crate::server_side_json_builder::ServerSidePoggers;
use deadpool_postgres::tokio_postgres::Client;

use convert_case::{Case, Casing};
use inflector::Inflector;
use petgraph::graph::DiGraph;
use petgraph::prelude::NodeIndex;
use postgraphile_introspection::{introspection_query_data, IntrospectionOutput};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GraphQLType {
    pub field_to_types: HashMap<String, (String, usize)>,
    pub table_name: String,
    pub primary_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GraphQLFieldNames {
    pub incoming: String,
    pub outgoing: String,
}

#[derive(Debug, Clone)]
pub struct GraphQLEdgeInfo {
    pub graphql_field_name: GraphQLFieldNames,
    pub incoming_node_cols: Vec<String>,
    pub outgoing_node_cols: Vec<String>,
}

#[derive(Clone)]
pub enum Operation {
    Query(bool, NodeIndex<u32>),
    Delete(NodeIndex<u32>),
    Update(NodeIndex<u32>),
    Insert(NodeIndex<u32>),
}
static POG_INT: usize = 0;
static POG_STR: usize = 1;
static POG_FLOAT: usize = 2;
static POG_TIMESTAMP: usize = 3;
static POG_TIMESTAMPTZ: usize = 4;
static POG_BOOLEAN: usize = 5;
static POG_JSON: usize = 6;
static POG_NULLABLE_INT: usize = 7;

#[allow(dead_code)]
pub async fn create(client: &Client) -> ServerSidePoggers {
    let IntrospectionOutput {
        type_map,
        class_map,
        attribute_map,
        constraint_map,
    } = introspection_query_data(client).await;

    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let mut field_to_operation: HashMap<String, Operation> = HashMap::new();

    //for every class, add all its attributes and all
    for class in class_map.values() {
        let mut field_to_types: HashMap<String, (String, usize)> = HashMap::new();

        //iterate over the fields of this parent
        for field in attribute_map
            .values()
            .filter(|att| att.class_id == class.id)
        {
            //convert the data type to the corresponding data type
            let mut closure_index = match &*type_map.get(&field.type_id).unwrap().name {
                "int4" | "int2" | "smallint" | "bigint" => POG_INT,
                "character varying" | "text" | "varchar" => POG_STR,
                "timestamp with time zone" => POG_TIMESTAMPTZ,
                "timestamp" => POG_TIMESTAMP,
                "double precision" | "float8" | "numeric" => POG_FLOAT,
                "boolean" => POG_BOOLEAN,
                "json" | "jsonb" => POG_JSON,
                other => {
                    if !["_text", "tsvector"].contains(&other) {
                        panic!(
                            "Encountered unhandled type {} from table {}.{}",
                            other, class.name, field.name
                        )
                    }
                    0
                }
            };

            //if the field is null then offset by where the null fields start
            if !field.is_not_null {
                closure_index += POG_NULLABLE_INT;
            }

            //insert mapping of the graphql name (e.g commentUpvotes) to the closure and column
            //name (which can be used to fetch this column correctly, e.g in this case fetch
            //comment_upvotes as integer)
            field_to_types.insert(
                field.name.to_camel_case(),
                (field.name.to_string(), closure_index),
            );
        }
        g.add_node(GraphQLType {
            field_to_types,
            table_name: class.name.to_string(),
            primary_keys: vec![],
        });
    }

    for constraint in constraint_map.values() {
        //find the node corresponding to the constraint
        let node = g
            .node_indices()
            .find(|n| g[*n].table_name == class_map.get(&constraint.class_id).unwrap().name)
            .unwrap();

        //if is foreign constraint
        if let Some(foreign_class_id) = &constraint.foreign_class_id {
            //find the parent being referred to
            let parent_node = g
                .node_indices()
                .find(|n| {
                    g[*n].table_name == class_map.get(&foreign_class_id.to_owned()).unwrap().name
                })
                .unwrap();

            //attribute map indexes
            let child_foreign_cols = constraint
                .key_attribute_nums
                .iter()
                .map(|num| {
                    attribute_map
                        .get(&(constraint.class_id.to_string(), *num))
                        .unwrap()
                        .name
                        .to_string()
                })
                .collect::<Vec<String>>();

            let parent_primary_cols = constraint
                .foreign_key_attribute_nums
                .iter()
                .map(|num| {
                    attribute_map
                        .get(&(foreign_class_id.to_string(), *num))
                        .unwrap()
                        .name
                        .to_string()
                })
                .collect::<Vec<String>>();

            g.add_edge(
                node,
                parent_node,
                GraphQLEdgeInfo {
                    outgoing_node_cols: parent_primary_cols,
                    graphql_field_name: GraphQLFieldNames {
                        //the incoming edge is referred to singularily (many to one) whilst the
                        //outgoing by one to many (plural)
                        incoming: gen_edge_field_name(
                            &g[node].table_name,
                            &child_foreign_cols,
                            true,
                        ),
                        outgoing: gen_edge_field_name(
                            &g[parent_node].table_name,
                            &child_foreign_cols,
                            false,
                        ),
                    },
                    incoming_node_cols: child_foreign_cols,
                },
            );
        }
        //if no foreign keys then assume primary key constraint
        else {
            let pks = constraint
                .key_attribute_nums
                .iter()
                .map(|num| {
                    attribute_map
                        .get(&(constraint.class_id.to_string(), *num))
                        .unwrap()
                        .name
                        .to_string()
                })
                .collect::<Vec<String>>();
            g[node].primary_keys = pks;
        }
    }

    //create queries for tables
    for class in class_map.values() {
        let node = g
            .node_indices()
            .find(|n| g[*n].table_name == class.name)
            .unwrap();
        field_to_operation::build_mutation(node, &mut field_to_operation, class);
    }
    ServerSidePoggers {
        field_to_operation,
        g,
    }
}
fn gen_edge_field_name(table_name: &str, foreign_cols: &[String], pluralize: bool) -> String {
    [
        &if pluralize {
            table_name.to_camel_case().to_plural()
        } else {
            table_name.to_camel_case().to_singular()
        },
        "By",
        &foreign_cols
            .iter()
            .map(|fk| fk.to_case(Case::UpperCamel))
            .collect::<Vec<String>>()
            .join("And"),
    ]
    .concat()
}

#[allow(dead_code)]
pub async fn get_pogg_and_client() -> (ServerSidePoggers, Client) {
    let (client, connection) = deadpool_postgres::tokio_postgres::connect(
        "postgres://postgres:postgres@127.0.0.1:5432/pets",
        deadpool_postgres::tokio_postgres::NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let pogg = create(&client).await;
    (pogg, client)
}
