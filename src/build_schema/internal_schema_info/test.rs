use super::*;
use petgraph::graph::Edge;
use postgres::{Client, NoTls};
fn assert_some_edge_eq(expected: &GraphQLEdgeInfo, edges: &[Edge<GraphQLEdgeInfo>]) {
    for edge in edges {
        let GraphQLEdgeInfo {
            graphql_field_name,
            foreign_keys,
        } = &edge.weight;
        if graphql_field_name == &expected.graphql_field_name
            && foreign_keys.len() == expected.foreign_keys.len()
            && foreign_keys
                .iter()
                .zip(&expected.foreign_keys)
                .all(|(a, b)| a.0 == b.0 && a.1 == b.1)
        {
            return;
        }
    }
    panic!("No edge found:\n\n {:?}\n\n", expected);
}

//fn check_single_connection() {
//    let Poggers {g, local_id: _, query_to_type: _, query_builder: _} = 
//    assert_some_edge_eq(
//        &GraphQLEdgeInfo {
//            one_to_many: true,
//            foreign_keys: vec![("id".to_string(), "parent_table_id".to_string())],
//            graphql_field_name: "parentTables".to_string(),
//        },
//        g.raw_edges(),
//    );
//    assert_some_edge_eq(
//        &GraphQLEdgeInfo {
//            one_to_many: false,
//            foreign_keys: vec![("parent_table_id".to_string(), "id".to_string())],
//            graphql_field_name: "parentTable".to_string(),
//        },
//        g.raw_edges(),
//    );
//    assert_eq!(g.raw_edges().len(), 2);
//}
