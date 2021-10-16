//use super::*;
//use petgraph::graph::Edge;
//use postgres::{Client, NoTls};
//fn connect_create_schema(query: &str) -> Result<Poggers<PostgresBuilder>, postgres::Error> {
//    let mut client = Client::connect(
//        "postgres://eerik:Postgrizzly@localhost:5432/poggers_testing",
//        NoTls,
//    )?;
//    client.batch_execute("drop schema if exists public cascade ; create schema public")?;
//    client.batch_execute(query)?;
//    Ok(create(
//        "postgres://eerik:Postgrizzly@localhost:5432/poggers_testing",
//    ))
//}
//fn assert_some_edge_eq(expected: &GraphQLEdgeInfo, edges: &[Edge<GraphQLEdgeInfo>]) {
//    for edge in edges {
//        let GraphQLEdgeInfo {
//            graphql_field_name,
//            one_to_many,
//            foreign_keys,
//        } = &edge.weight;
//        if graphql_field_name == &expected.graphql_field_name
//            && one_to_many == &expected.one_to_many
//            && foreign_keys.len() == expected.foreign_keys.len()
//            && foreign_keys
//                .iter()
//                .zip(&expected.foreign_keys)
//                .all(|(a, b)| a.0 == b.0 && a.1 == b.1)
//        {
//            return;
//        }
//    }
//    panic!("No edge found:\n\n {:?}\n\n", expected);
//}
//
//#[test]
//fn check_single_connection() {
//    let Poggers {g, local_id: _, query_to_type: _, query_builder: _} = connect_create_schema(
//    "
//        create table parent_table(id integer primary key generated always as identity);
//        grant all on parent_table to public;
//        create table foreign_table(
//          id integer primary key generated always as identity,
//          parent_table_id integer not null references parent_table(id) on delete cascade
//        );
//        create index if not exists foreign_table_parent_table_idx on \"foreign_table\"(parent_table_id);
//        grant all on foreign_table to public;
//    ",
//    ).unwrap();
//
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
