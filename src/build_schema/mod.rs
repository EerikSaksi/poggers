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

    //for every class, add all its attributes and all
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
        g.add_node(GraphQLType {
            field_to_types,
            table_name: class.name,
        });
    }

    for constraint in constraint_map.values() {
        //find the node corresponding to the constraint
        let node = g
            .node_weights()
            .find(|n| n.table_name == class_map.get(&constraint.class_id).unwrap().name);

        //if is foreign constraint
        if let Some(foreign_class_id) = constraint.foreign_class_id {
            //find the parent being referred to
            let parent_node = g
                .node_weights()
                .find(|n| n.table_name == class_map.get(&foreign_class_id).unwrap().name);

            let child_foreign_cols = {
                let child_atts = attribute_vec
                    .iter()
                    //first narrow down the attributes to those belonging to this class
                    .filter(|att| att.class_id == constraint.class_id)
                    //then narrow down the class atts to those in the attribute vector
                    .filter(|att| constraint.key_attribute_nums.contains(&att.num));
            };
        }
    }

    ServerSidePoggers {
        query_to_type,
        g,
        local_id: 0,
        num_select_cols: 0,
    }
}


