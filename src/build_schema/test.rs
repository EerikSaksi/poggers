use super::*;
use crate::build_schema::get_pogg_and_client;
use petgraph::graph::Edge;
fn assert_some_edge_eq(
    field_names: (&str, &str),
    incoming_node_cols: Vec<&str>,
    edges: &[Edge<GraphQLEdgeInfo>],
) {
    let expected = GraphQLEdgeInfo {
        graphql_field_name: GraphQLFieldNames {
            incoming: field_names.0.to_string(),
            outgoing: field_names.1.to_string(),
        },
        incoming_node_cols: incoming_node_cols.iter().map(|s| s.to_string()).collect(),
        outgoing_node_cols: vec![],
    };

    let mut cumm = String::new();
    for edge in edges {
        let GraphQLEdgeInfo {
            graphql_field_name,
            incoming_node_cols,
            outgoing_node_cols: _,
        } = &edge.weight;
        cumm.push_str(&format!("{:?}\n", &edge.weight));
        if graphql_field_name.incoming == expected.graphql_field_name.incoming
            && graphql_field_name.outgoing == expected.graphql_field_name.outgoing
            && incoming_node_cols.len() == expected.incoming_node_cols.len()
            && incoming_node_cols
                .iter()
                .zip(&expected.incoming_node_cols)
                .all(|(a, b)| a == b)
        {
            return;
        }
    }
    panic!(
        "No edge found: {:?} {:?}\n\n\n{}",
        field_names, incoming_node_cols, cumm
    );
}


#[actix_rt::test]
async fn test_one_to_many() {
    let (pogg, _) = get_pogg_and_client().await;
    let g = pogg.g;
    assert_some_edge_eq(
        ("postsByOwneruserid", "siteUserByOwneruserid"),
        vec!["owneruserid"],
        g.raw_edges(),
    );
    assert_some_edge_eq(
        ("badgesByUserid", "siteUserByUserid"),
        vec!["userid"],
        g.raw_edges(),
    );
    assert_some_edge_eq(
        ("commentsByPostid", "postByPostid"),
        vec!["postid"],
        g.raw_edges(),
    );
}

#[actix_rt::test]
async fn test_composite_primary_keys() {
    let (pogg, _) = get_pogg_and_client().await;
    let g = pogg.g;
    assert_some_edge_eq(
        (
            "compoundChildTablesByParentId1AndParentId2",
            "compoundTableByParentId1AndParentId2",
        ),
        vec!["parent_id1", "parent_id2"],
        g.raw_edges(),
    );
    let child = g
        .node_weights()
        .find(|n| n.table_name == "compound_child_table")
        .unwrap();
    for field in ["parentId1", "parentId2"] {
        assert!(
            child.field_to_types.contains_key(field),
            "{} doesn't have key, actually has {:?}",
            field,
            &child.field_to_types.keys()
        );
    }
}

#[actix_rt::test]
async fn check_id_primary_keys() {
    let (pogg, _) = get_pogg_and_client().await;
    let g = pogg.g;
    for weight in g.node_weights() {
        //every table but the parent table one has primary key as id
        if !["compound_table", "compound_child_table", "foreign_primary_key"].contains(&&*weight.table_name) {
            assert_eq!(weight.primary_keys, vec!["id"], "{}", weight.table_name);
        }
    }
}

#[actix_rt::test]
async fn foreign_primary_key() {
    let (pogg, _) = get_pogg_and_client().await;
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
    assert_eq!(g[edge].incoming_node_cols, vec!["post_id"]);
}

#[actix_rt::test]
async fn field_to_operation() {
    let (pogg, _) = get_pogg_and_client().await;
    let field_to_operation = pogg.field_to_operation;
    assert!(
        field_to_operation.contains_key("siteUsers"),
        "{}",
        field_to_operation
            .keys()
            .into_iter()
            .fold(String::new(), |a, b| format!("{}\n{}", a, b))
    )
}

#[actix_rt::test]
async fn post_has_owneruserid() {
    let (pogg, _) = get_pogg_and_client().await;
    let g = pogg.g;
    let post_node = g.node_weights().find(|n| n.table_name == "post").unwrap();
    assert!(
        post_node.field_to_types.contains_key("owneruserid"),
        "No owneruserid, only {:?}",
        post_node.field_to_types.keys()
    );
}

#[actix_rt::test]
async fn post_has_correct_num_fields() {
    let (pogg, _) = get_pogg_and_client().await;
    let g = pogg.g;
    let post_node = g.node_weights().find(|n| n.table_name == "post").unwrap();
    assert_eq!(
        post_node.field_to_types.len(),
        21,
        "Found extra fields: {:?}",
        post_node.field_to_types.keys()
    );
}

#[actix_rt::test]
async fn check_nullability() {
    let (pogg, _) = get_pogg_and_client().await;
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
        ("lastaccessdate", POG_TIMESTAMP + POG_NULLABLE_INT),
        ("websiteurl", POG_STR + POG_NULLABLE_INT),
        ("location", POG_STR + POG_NULLABLE_INT),
        ("aboutme", POG_STR + POG_NULLABLE_INT),
        ("views", POG_INT),
        ("upvotes", POG_INT),
        ("downvotes", POG_INT),
        ("profileimageurl", POG_STR + POG_NULLABLE_INT),
        ("age", POG_INT + POG_NULLABLE_INT),
        ("accountid", POG_INT + POG_NULLABLE_INT),
        ("jsonfield", POG_JSON + POG_NULLABLE_INT),
    ];
    for (key, expected) in expected_column_types {
        assert_eq!(user_node.field_to_types.get(key).unwrap().1, expected);
    }
}

#[actix_rt::test]
async fn test_delete_comment_mutation() {
    let (pogg, _) = get_pogg_and_client().await;
    let field_to_operation = pogg.field_to_operation;
    assert!(
        field_to_operation.contains_key("deleteComment"),
        "{:?} actually contains keys",
        field_to_operation.keys()
    )
}

//#[actix_rt::test]
//async fn test_by_fk() {
//    let g = create().g;
//    let post_node = g
//        .node_indices()
//        .find(|n| g[*n].table_name == "post")
//        .unwrap();
//    let site_user_node = g
//        .node_indices()
//        .find(|n| g[*n].table_name == "site_user")
//        .unwrap();
//    assert_some
//}
