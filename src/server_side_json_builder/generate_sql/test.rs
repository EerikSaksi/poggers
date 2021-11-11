use crate::{build_schema::create, server_side_json_builder::generate_sql::JsonBuilderContext};

#[test]
fn column_offsets() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            reputation
            views
            upvotes
            downvotes
            posts{
              id
              posttypeid
            }
          }
        }";
    let JsonBuilderContext { sql_query: _, table_query_infos, root_key_name, root_query_is_many } = pogg.build_root(query).unwrap();
    assert_eq!(table_query_infos.get(0).unwrap().primary_key_range.start, 0);
    assert_eq!(table_query_infos.get(1).unwrap().primary_key_range.start, 5);
}

#[test]
fn test_invalid_root_query() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          commentos {
              id
          }
        }";
    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(
        err.as_str(),
        "No query named \"commentos\""
    );
}
#[test]
fn test_invalid_syntax() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          comments {
              id
        }";
    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(
        err.as_str(),
        " --> 5:10\n  |\n5 |         }\n  |          ^---\n  |\n  = expected selection"
    );
}

#[test]
fn test_invalid_subchild() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
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

#[test]
fn test_error_propagation() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
            siteUsers{
                id
                displayname
                posts {
                    id
                    title
                    comments{
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

#[test]
fn test_no_root() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
        }";

    let err = pogg.build_root(query).expect_err("Wasn't Err");
    assert_eq!(
        err.as_str(),
        " --> 3:9\n  |\n3 |         }\n  |         ^---\n  |\n  = expected selection"
    );
}

