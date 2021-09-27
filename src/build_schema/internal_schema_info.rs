use crate::build_schema::{convert_pg_to_gql, read_database};
use petgraph::{data::Build, graph::DiGraph, visit::IntoNodeReferences};
use std::collections::HashSet;

struct GraphQLType<'a> {
    fields: HashSet<&'a str>,
    table_name: &'a str,
}

fn create() {
    let types: Vec<GraphQLType> = Vec::new();
    let mut g: DiGraph<GraphQLType, ()> = DiGraph::new();
    for current_row in read_database::read_tables().unwrap().iter() {
        let table_name: &str = current_row.get("table_name");
        //let column_name: &str = current_row.get("column_name");
        //let nullable: &str = current_row.get("nullable");
        let data_type: &str = current_row.get("data_type");
        let foreign_table_name: Option<&str> = current_row.get("foreign_table_name");
        let graphql_type = convert_pg_to_gql(data_type);

        //add this table as a node if no such node
        let foreign_index_optional = g.node_indices().find(|i| g[*i].table_name == table_name);
        let source_index = match foreign_index_optional {
            Some(index) => index,
            None => g.add_node(GraphQLType {
                fields: HashSet::new(),
                table_name,
            }),
        };

        if let Some(foreign_table_name) = foreign_table_name {
            //unwrap as we inserted this node if it was missing right above
            //we dont know about the foreign table
            let foreign_index_optional = g
                .node_indices()
                .find(|i| g[*i].table_name == foreign_table_name);

            //either return the index we found or insert and return the index of that new item
            let foreign_index = match foreign_index_optional {
                Some(foreign_index) => foreign_index,
                None => g.add_node(GraphQLType {
                    fields: HashSet::new(),
                    table_name: foreign_table_name,
                }),
            };
            g.add_edge(source_index, foreign_index, ());
        }
        g[*source_index];

        //current graphql type is either an existing item in the vector, or a newly inserted item
    }
}
