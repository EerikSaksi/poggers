#[cfg(test)]
#[path = "./test.rs"]
mod test;
use crate::build_schema::read_database;
use crate::handle_query::postgres_query_builder::PostgresBuilder;
use crate::handle_query::Poggers;
use inflector::Inflector;
use petgraph::graph::DiGraph;
use petgraph::prelude::NodeIndex;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct GraphQLType {
    pub terminal_fields: HashSet<String>,
    pub table_name: String,
    pub primary_keys: Vec<String>,
}

#[derive(Debug)]
pub struct GraphQLEdgeInfo {
    pub graphql_field_name: (String, String),
    pub foreign_keys: Vec<String>,
}

pub struct QueryEdgeInfo {
    pub is_many: bool,
    pub node_index: NodeIndex<u32>,
}

#[allow(dead_code)]
pub fn create(database_url: &str) -> Poggers<PostgresBuilder> {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();

    for current_row in read_database::read_tables(database_url).unwrap().iter() {
        let table_name: String = current_row.get("table_name");
        let column_name: String = current_row.get("column_name");
        let foreign_table_name: Option<String> = current_row.get("foreign_table_name");
        //let nullable: &str = current_row.get("nullable");

        //add this table as a node if no such node

        if let Some(foreign_table_name) = foreign_table_name {
            //unwrap as we inserted this node if it was missing right above
            //we dont know about the foreign table
            let parent_index_optional = g.node_indices().find(|i| g[*i].table_name == table_name);
            let parent_index = match parent_index_optional {
                Some(index) => index,
                None => g.add_node(GraphQLType {
                    terminal_fields: HashSet::new(),
                    table_name: table_name.clone(),
                    primary_keys: vec!["id".to_string()],
                }),
            };
            let child_index_optional = g
                .node_indices()
                .find(|i| g[*i].table_name == foreign_table_name);

            //either return the index we found or insert and return the index of that new item
            let child_index = match child_index_optional {
                Some(foreign_index) => foreign_index,
                None => g.add_node(GraphQLType {
                    terminal_fields: HashSet::new(),
                    table_name: foreign_table_name.clone(),
                    primary_keys: vec!["id".to_string()],
                }),
            };

            //create one to many relationship
            g.add_edge(
                child_index,
                parent_index,
                GraphQLEdgeInfo {
                    graphql_field_name: (
                        foreign_table_name.to_camel_case().to_plural(),
                        table_name.to_camel_case(),
                    ),
                    foreign_keys: vec![column_name.to_string()],
                },
            );
            query_to_type.insert(
                foreign_table_name.clone().to_camel_case().to_plural(),
                QueryEdgeInfo {
                    node_index: parent_index,
                    is_many: true,
                },
            );
            query_to_type.insert(
                foreign_table_name.clone().to_camel_case(),
                QueryEdgeInfo {
                    node_index: parent_index,
                    is_many: false,
                },
            );
        }

        g.node_weights_mut().for_each(|graphql_type| {
            if graphql_type.table_name == table_name {
                graphql_type.terminal_fields.insert(column_name.clone());
            }
        });
    }
    Poggers {
        g,
        query_to_type,
        local_id: 0,
        query_builder: PostgresBuilder {},
    }
}
