mod mutation_builder;
mod read_database;
#[cfg(test)]
#[path = "./test.rs"]
mod test;

use crate::server_side_json_builder::ServerSidePoggers;
use inflector::Inflector;
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
use std::collections::HashMap;

use self::mutation_builder::build_mutations;

pub struct GraphQLType {
    pub field_to_types: HashMap<String, (String, usize)>,
    pub table_name: String,
    pub primary_keys: Vec<String>,
}

#[derive(Debug)]
pub struct GraphQLEdgeInfo {
    pub graphql_field_name: (String, String),
    pub foreign_keys: Vec<String>,
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
static POG_NULLABLE_STR: usize = 8;
static POG_NULLABLE_FLOAT: usize = 9;
static POG_NULLABLE_TIMESTAMP: usize = 10;
static POG_NULLABLE_TIMESTAMPTZ: usize = 11;
static POG_NULLABLE_BOOLEAN: usize = 12;
static POG_NULLABLE_JSON: usize = 13;

#[allow(dead_code)]
pub fn create(database_url: &str) -> ServerSidePoggers {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();

    for current_row in read_database::read_tables(database_url).unwrap().iter() {
        let table_name: String = current_row.get("table_name");
        let parent_table_name: Option<String> = current_row.get("parent_table");
        let column_name: String = current_row.get("column_name");
        let is_primary: bool = current_row.get("is_primary");
        let nullable: &str = current_row.get("is_nullable");
        let child_index = find_or_create_node(&mut g, &table_name);

        //if this is a primary key and we have not added it yet (a child might add it for us if it
        //needs to add its foreign keys) then add it
        if is_primary && !g[child_index].primary_keys.contains(&column_name) {
            g[child_index].primary_keys.push(column_name.to_string());
        }

        //create this child, or find it if already exists
        //
        let child_index = find_or_create_node(&mut g, &table_name);
        if let Some(parent_table_name) = parent_table_name {
            let parent_index = find_or_create_node(&mut g, &parent_table_name);

            //find edge or create one if missing
            let edge = match g.find_edge(child_index, parent_index) {
                Some(e) => e,
                None => g.add_edge(
                    child_index,
                    parent_index,
                    GraphQLEdgeInfo {
                        graphql_field_name: (
                            table_name.to_camel_case().to_plural(),
                            parent_table_name.to_camel_case(),
                        ),
                        foreign_keys: vec![],
                    },
                ),
            };

            let parent_column: String = current_row.get("parent_column");
            handle_foreign_key(&mut g, parent_index, edge, &parent_column, &column_name);
        }

        //depending on the value of the datatype select the correct closure index
        let data_type: &str = current_row.get("data_type");
        let data_type_index = match nullable {
            "YES" => match data_type {
                "integer" | "smallint" | "bigint" => POG_NULLABLE_INT,
                "character varying" | "text" => POG_NULLABLE_STR,
                "timestamp with time zone" => POG_NULLABLE_TIMESTAMPTZ,
                "timestamp without time zone" => POG_NULLABLE_TIMESTAMP,
                "double precision" | "float" => POG_NULLABLE_FLOAT,
                "boolean" => POG_NULLABLE_BOOLEAN,
                "json" | "jsonb" => POG_NULLABLE_JSON,
                other => panic!("Encountered unhandled type {}", other),
            },
            "NO" => match data_type {
                "integer" | "smallint" | "bigint" => POG_INT,
                "character varying" | "text" => POG_STR,
                "timestamp with time zone" => POG_TIMESTAMPTZ,
                "timestamp without time zone" => POG_TIMESTAMP,
                "double precision" | "float" => POG_FLOAT,
                "boolean" => POG_BOOLEAN,
                "json" | "jsonb" => POG_JSON,
                other => panic!("Encountered unhandled type {}", other),
            },
            other => panic!("Nullable was {}", other),
        };
        g[child_index]
            .field_to_types
            .insert(column_name.to_camel_case(), (column_name, data_type_index));
    }
    let mut query_to_type: HashMap<String, Operation> = HashMap::new();
    for node_index in g.node_indices() {
        let GraphQLType {
            table_name,
            primary_keys: _,
            field_to_types: _,
        } = &g[node_index];

        //many query (no filter)
        query_to_type.insert(
            table_name.clone().to_plural().to_camel_case(),
            Operation::Query(true, node_index),
        );

        //to one (by primary key)
        query_to_type.insert(
            table_name.clone().to_camel_case(),
            Operation::Query(false, node_index),
        );
    }
    build_mutations(&g, &mut query_to_type);
    ServerSidePoggers::new(g, query_to_type)
}
fn find_or_create_node(
    g: &mut DiGraph<GraphQLType, GraphQLEdgeInfo>,
    table_name: &str,
) -> NodeIndex<u32> {
    let node_index_optional = g.node_indices().find(|i| g[*i].table_name == table_name);
    match node_index_optional {
        Some(foreign_index) => foreign_index,
        None => g.add_node(GraphQLType {
            field_to_types: HashMap::new(),
            table_name: table_name.to_string(),
            primary_keys: vec![],
        }),
    }
}

fn handle_foreign_key(
    g: &mut DiGraph<GraphQLType, GraphQLEdgeInfo>,
    node: NodeIndex<u32>,
    edge: EdgeIndex<u32>,
    new_parent_pk: &str,
    new_child_fk: &str,
) {
    //check if the parent already has this primary key. If not we need to insert it into the parent
    //first, before we know where it goes into the child
    let parent_pk_index = g[node]
        .primary_keys
        .iter()
        .position(|pk| pk == new_parent_pk);

    //the index of the childs fk needs to match the index of the parent's pk
    match parent_pk_index {
        Some(i) => {
            //ensure no out of bounds
            while g[edge].foreign_keys.len() < g[node].primary_keys.len() {
                g[edge].foreign_keys.push("".to_string());
            }
            g[edge].foreign_keys[i] = new_child_fk.to_string();
        }
        None => {
            g[node].primary_keys.push(new_parent_pk.to_string());
            let right_most = g[node].primary_keys.len() - 1;

            //no out of bounds
            while g[edge].foreign_keys.len() < g[node].primary_keys.len() {
                g[edge].foreign_keys.push("".to_string());
            }
            //last element
            g[edge].foreign_keys[right_most] = new_child_fk.to_string();
        }
    }
}
