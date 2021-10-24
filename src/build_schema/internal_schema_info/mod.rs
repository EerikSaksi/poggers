#[cfg(test)]
#[path = "./test.rs"]
mod test;
use crate::build_schema::read_database;
use crate::handle_query::Poggers;
use crate::server_side_json_builder::ServerSidePoggers;
use inflector::Inflector;
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
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
pub fn create(database_url: &str) -> ServerSidePoggers {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();

    for current_row in read_database::read_tables(database_url).unwrap().iter() {
        let table_name: String = current_row.get("table_name");
        let parent_table_name: Option<String> = current_row.get("parent_table");
        let column_name: String = current_row.get("column_name");
        let is_primary: bool = current_row.get("is_primary");
        let child_index = find_or_create_node(&mut g, &table_name);
        if is_primary && !g[child_index].primary_keys.contains(&column_name) {
            g[child_index].primary_keys.push(column_name.to_string());
        }

        if let Some(parent_table_name) = parent_table_name {
            let parent_index = find_or_create_node(&mut g, &parent_table_name);

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

        query_to_type.insert(
            table_name.clone().to_plural().to_camel_case(),
            QueryEdgeInfo {
                node_index: child_index,
                is_many: true,
            },
        );
        g.node_weights_mut().for_each(|graphql_type| {
            if graphql_type.table_name == table_name {
                graphql_type.terminal_fields.insert(column_name.clone());
            }
        });
    }
    ServerSidePoggers {
        g,
        query_to_type,
        local_id: 0,
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
            terminal_fields: HashSet::new(),
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
    let parent_pk_index = g[node]
        .primary_keys
        .iter()
        .position(|pk| pk == &new_parent_pk);

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
            let a = &g[node].primary_keys;

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
