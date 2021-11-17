mod mutation_builder;
mod postgraphile_introspection;
#[cfg(test)]
#[path = "./test.rs"]
mod test;

use self::mutation_builder::build_mutations;
use crate::server_side_json_builder::ServerSidePoggers;
use inflector::Inflector;
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
use postgraphile_introspection::{introspection_query_data, IntrospectionOutput};
use std::collections::HashMap;

pub struct GraphQLType {
    pub field_to_types: HashMap<String, (String, usize)>,
    pub table_name: String,
}

#[derive(Debug)]
pub struct GraphQLEdgeInfo {
    pub graphql_field_name: (String, String),
    pub incoming_node_fields: Vec<String>,
    pub outgoing_node_fields: Vec<String>,
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
    let IntrospectionOutput {
        type_map,
        class_map,
        attribute_vec,
        constraint_map,
    } = introspection_query_data();

    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let mut query_to_type: HashMap<String, Operation> = HashMap::new();

    for class in class_map.values() {
        let mut field_to_types: HashMap<String, (String, usize)> = HashMap::new();

        //iterate over the fields of this parent
        for field in attribute_vec.iter().filter(|att| att.class_id == class.id) {
            //convert the data type to the corresponding data type
            let mut closure_index = match &*type_map.get(&field.type_id).unwrap().name {
                "integer" | "smallint" | "bigint" => POG_NULLABLE_INT,
                "character varying" | "text" => POG_NULLABLE_STR,
                "timestamp with time zone" => POG_NULLABLE_TIMESTAMPTZ,
                "timestamp without time zone" => POG_NULLABLE_TIMESTAMP,
                "double precision" | "float" => POG_NULLABLE_FLOAT,
                "boolean" => POG_NULLABLE_BOOLEAN,
                "json" | "jsonb" => POG_NULLABLE_JSON,
                other => panic!("Encountered unhandled type {}", other),
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
        g.add_node(GraphQLType{field_to_types, table_name: class.name, primary_keys: class.})
    }
    ServerSidePoggers {
        query_to_type,
        g,
        local_id: 0,
        num_select_cols: 0,
    }
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
