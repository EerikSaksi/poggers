use crate::{
    build_schema::get_pogg_and_client, server_side_json_builder::generate_sql::JsonBuilderContext,
};

#[actix_rt::test]
async fn column_offsets() {
    let (pogg, _) = get_pogg_and_client();
    let query = "
        query{
          siteUsers{
            reputation
            views
            upvotes
            downvotes
            postsByOwneruserid{
              id
              posttypeid
            }
          }
        }";
    let JsonBuilderContext {
        sql_query: _,
        table_query_infos,
        root_key_name: _,
        root_query_is_many: _,
    } = pogg.build_root(query).unwrap();
    assert_eq!(table_query_infos.get(0).unwrap().primary_key_range.start, 0);
    assert_eq!(table_query_infos.get(1).unwrap().primary_key_range.start, 5);
}

#[actix_rt::test]
async fn test_invalid_root_query() {
    let (pogg, _) = get_pogg_and_client();
    let query = "
        query{
          commentos {
              id
          }
        }";
    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(err.as_str(), "No operation named \"commentos\"");
}
#[actix_rt::test]
async fn test_invalid_syntax() {
    let (pogg, _) = get_pogg_and_client();
    let query = "
        query{
          comments {
              id
            }
          ";
    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(
        err.as_str(),
        " --> 6:11\n  |\n6 |           \n  |           ^---\n  |\n  = expected selection"
    );
}

#[actix_rt::test]
async fn test_invalid_subchild() {
    let (pogg, _) = get_pogg_and_client();
    let query = "
        query{
          posts {
              nonExistentChild
          }
        }";

    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(
        err.as_str(),
        "Post does not have selection nonExistentChild"
    );
}

#[actix_rt::test]
async fn test_error_propagation() {
    let (pogg, _) = get_pogg_and_client();
    let query = "
        query{
            siteUsers{
                id
                displayname
                postsByOwneruserid{
                    id
                    title
                    commentsByPostid{
                        nonExistentChild
                    }
                }
            }
        }";

    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(
        err.as_str(),
        "Comment does not have selection nonExistentChild"
    );
}

#[actix_rt::test]
async fn test_no_root() {
    let (pogg, _) = get_pogg_and_client();
    let query = "
        query{
        }";
    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(
        err.as_str(),
        " --> 3:9\n  |\n3 |         }\n  |         ^---\n  |\n  = expected selection"
    );
}

#[actix_rt::test]
async fn delete_mutation() {
    let (pogg, _) = get_pogg_and_client();
    let gql_query = "
        mutation{
          deleteMutationTest(id: 1){
            nullableFloat 
          }
        }
        ";
    let ctx = pogg.build_root(gql_query).unwrap();
    assert_eq!(ctx.sql_query, "WITH __table_0__ AS ( DELETE FROM mutation_test AS __table_0__ WHERE __table_0__.id = 1 RETURNING *) SELECT __table_0__.id AS __t0_pk0__, __table_0__.nullable_float AS __t0_c0__ FROM __table_0__");
}

#[actix_rt::test]
async fn handle_named_operation() {
    let (pogg, _) = get_pogg_and_client();
    let gql_query = "
        query named_operation {
            siteUsers{
                id
                displayname
            }
        }
        ";
    pogg.build_root(gql_query).unwrap();
}
