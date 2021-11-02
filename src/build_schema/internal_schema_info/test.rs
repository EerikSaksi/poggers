use super::*;
use petgraph::graph::Edge;
fn assert_some_edge_eq(
    field_names: (&str, &str),
    foreign_keys: Vec<&str>,
    edges: &[Edge<GraphQLEdgeInfo>],
) {
    let expected = GraphQLEdgeInfo {
        graphql_field_name: (field_names.0.to_string(), field_names.1.to_string()),
        foreign_keys: foreign_keys
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    };

    let mut cumm = String::new();
    for edge in edges {
        let GraphQLEdgeInfo {
            graphql_field_name,
            foreign_keys,
        } = &edge.weight;
        cumm.push_str(&format!("{:?}\n", &edge.weight));
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
    panic!("No edge found:\n\n {:?}\n{}\n", expected, cumm);
}

#[test]
fn test_one_to_many() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let g = pogg.g;
    assert_some_edge_eq(("posts", "siteUser"), vec!["owneruserid"], g.raw_edges());
    assert_some_edge_eq(("badges", "siteUser"), vec!["userid"], g.raw_edges());
    assert_some_edge_eq(("comments", "post"), vec!["postid"], g.raw_edges());
}

#[test]
fn test_composite_primary_keys() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let g = pogg.g;
    assert_some_edge_eq(
        ("childTables", "parentTable"),
        vec!["parent_id1", "parent_id2"],
        g.raw_edges(),
    );
}

#[test]
fn check_id_primary_keys() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let g = pogg.g;
    for weight in g.node_weights() {
        //every table but the parent table one has primary key as id
        if !["parent_table", "child_table", "foreign_primary_key"].contains(&&*weight.table_name) {
            assert_eq!(weight.primary_keys, vec!["id"], "{}", weight.table_name);
        }
    }
}

#[test]
fn foreign_primary_key() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let g = pogg.g;
    let node = g
        .node_indices()
        .find(|node| g[*node].table_name == "foreign_primary_key")
        .unwrap();
    assert_eq!(g[node].primary_keys, vec!["post_id"]);
    let foreign_node = g
        .node_indices()
        .find(|node| g[*node].table_name == "post")
        .unwrap();
    let edge = g.find_edge(node, foreign_node).unwrap();
    assert_eq!(g[edge].foreign_keys, vec!["post_id"]);
}

#[test]
fn query_to_type() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query_to_type = pogg.query_to_type;
    assert!(
        query_to_type.contains_key("siteUsers"),
        "{}",
        query_to_type
            .keys()
            .into_iter()
            .fold(String::new(), |a, b| format!("{}\n{}", a, b))
    )
}

#[test]
fn post_has_owneruserid() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let g = pogg.g;
    let post_node = g.node_weights().find(|n| n.table_name == "post").unwrap();
    assert!(
        post_node.field_to_types.contains_key("owneruserid"),
        "No owneruserid, only {:?}",
        post_node.field_to_types.keys()
    );
}

#[test]
fn post_has_correct_num_fields() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let g = pogg.g;
    let post_node = g.node_weights().find(|n| n.table_name == "post").unwrap();
    assert_eq!(
        post_node.field_to_types.len(),
        21,
        "Found extra fields: {:?}",
        post_node.field_to_types.keys()
    );
}

#[test]
fn check_nullability() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let g = pogg.g;
    let user_node = g
        .node_weights()
        .find(|n| n.table_name == "site_user")
        .unwrap();
    let expected_column_types = vec![
        ("id", POG_INT),
        ("reputation", POG_INT),
        ("creationdate", POG_TIMESTAMP),
        ("displayname", POG_STR),
        ("lastaccessdate", POG_NULLABLE_TIMESTAMP),
        ("websiteurl", POG_NULLABLE_STR),
        ("location", POG_NULLABLE_STR),
        ("aboutme", POG_NULLABLE_STR),
        ("views", POG_INT),
        ("upvotes", POG_INT),
        ("downvotes", POG_INT),
        ("profileimageurl", POG_NULLABLE_STR),
        ("age", POG_NULLABLE_INT),
        ("accountid", POG_NULLABLE_INT),
        ("jsonfield", POG_NULLABLE_JSON),
    ];
    for (key, expected) in expected_column_types {
        assert_eq!(*user_node.field_to_types.get(key).unwrap(), expected);
    }
}
