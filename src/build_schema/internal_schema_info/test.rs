use super::*;
use petgraph::graph::Edge;
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
                .all(|(a, b)| a == b)
        {
            return;
        }
    }
    panic!("No edge found:\n\n {:?}\n\n", expected);
}

#[test]
fn check_many_to_one() {
    let Poggers {
        g,
        local_id: _,
        query_to_type: _,
        query_builder: _,
    } = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    assert_some_edge_eq(
        &GraphQLEdgeInfo {
            foreign_keys: vec!["owneruserid".to_string()],
            graphql_field_name: ("posts".to_string(), "siteUser".to_string()),
        },
        g.raw_edges(),
    );
}

#[test]
fn test_correct_num_edges() {
    let Poggers {
        g,
        local_id: _,
        query_to_type: _,
        query_builder: _,
    } = create("postgres://eerik:Postgrizzly@localhost:5432/pets");

    //when running the subquery in read_database 10 foreign keys are shown (also manually validated)
    assert_eq!(g.raw_edges().len(), 10);
}


#[test]
fn check_id_primary_keys() {
    let Poggers {
        g,
        local_id: _,
        query_to_type: _,
        query_builder: _,
    } = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    for weight in g.node_weights() {
        //every table but the parent table one has primary key as id
        if weight.table_name != "parent_table" {
            assert_eq!(weight.primary_keys, vec!["id"]);
        }
    }

    //when running the subquery in read_database 10 foreign keys are shown (also manually validated)
    assert_eq!(g.raw_edges().len(), 10);
}

