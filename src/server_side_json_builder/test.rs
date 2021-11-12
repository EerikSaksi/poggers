use super::*;
use crate::build_schema::create;
use postgres::{Client, NoTls}; // 0.19.2, features = ["with-chrono-0_4"]
use serde_json::{Error, Value};

fn convert_gql(gql_query: &str) -> Value {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    println!("\n{}\n", sql);
    let rows = client.query(&*[sql, ""].concat(), &[]).unwrap();
    let res = JsonBuilder::new(ctx).convert(rows);
    serde_json::from_str(&*res).unwrap()
}
fn mutation_test_fixtures() -> Client {
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    client.query("delete from mutation_test", &[]).unwrap();
    let post_ids = client.query("select id from post limit 100", &[]).unwrap();
    let values = (0..100)
        .map(|i| {
            let post_id: i32 = post_ids.get(i).unwrap().get(0);
            format!("({}, '{}', {}.5, {})", i, i, i, post_id)
        })
        .collect::<Vec<String>>()
        .join(", ");

    client
        .query(
            &format!(
                "insert into mutation_test(id, non_nullable_str, nullable_float, post_id) values {}",
                values
            ),
            &[],
        )
        .unwrap();
    client
}
#[allow(dead_code)]
fn write_json_to_file(res: &str) {
    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("foo.json").unwrap();
    file.write_all(res.as_bytes()).unwrap();
}

#[test]
fn test_random_user() {
    let gql_query = "
        query{
          siteUsers{
            id
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

    let p = convert_gql(gql_query);

    let site_users = p.get("siteUsers").unwrap();
    //test specific user sampled at random
    let user = site_users
        .as_array()
        .unwrap()
        .iter()
        .find(|user| user.get("id").unwrap() == 13)
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(user.get("reputation").unwrap(), 28971);
    assert_eq!(user.get("views").unwrap(), 3534);
    assert_eq!(user.get("upvotes").unwrap(), 4879);
    assert_eq!(user.get("downvotes").unwrap(), 207);
    assert_eq!(user.get("posts").unwrap().as_array().unwrap().len(), 535);
}

#[test]
fn all_posts_fetched() {
    let gql_query = "
        query{
          siteUsers{
            id
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
    let p = convert_gql(gql_query);
    let mut num_users = 0;
    let site_users = p.get("siteUsers").unwrap();
    let num_posts = site_users.as_array().unwrap().iter().fold(0, |cumm, user| {
        num_users += 1;
        let obj = user.as_object().unwrap();
        let posts = obj.get("posts").unwrap().as_array().unwrap();
        cumm + posts.len()
    });

    //select count(*) from post where post.owneruserid is not null;
    assert_eq!(num_posts, 17575);
    assert_eq!(
        num_users,
        16429,
        "Missing {} users. 10512 users don't have posts (did you left join)",
        16429 - num_users
    );
}

#[test]
fn all_posts_belong_to_parent() {
    let gql_query = "
        query {
          siteUsers{
            id
            reputation
            views
            upvotes
            downvotes
            posts{
              id
              posttypeid
              owneruserid
            }
          }
        }";
    let p = convert_gql(gql_query);
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        let obj = user.as_object().unwrap();
        let user_id = obj.get("id").unwrap().as_i64();
        let posts = obj.get("posts").unwrap().as_array().unwrap();
        assert!(posts
            .iter()
            .all(|post| post.get("owneruserid").unwrap().as_i64() == user_id))
    });
}

#[test]
fn non_nullable_string_fields() {
    let gql_query = "
        query {
          siteUsers{
            id
            displayname
          }
        }";
    let p = convert_gql(gql_query);
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        //test non nullable fields defined for all users
        let obj = user.as_object().unwrap();
        obj.get("id").unwrap().as_i64().unwrap();
        obj.get("displayname").unwrap().as_str().unwrap();
    });
}

#[test]
fn three_way_join() {
    let gql_query = "
        query {
          siteUsers{
            id
            reputation
            views
            upvotes
            downvotes
            posts{
              id
              posttypeid
              owneruserid
              comments{
                id
                score
                postid
                text
              }
            }
          }
        }";
    let p = convert_gql(gql_query);
    let mut num_users = 0;
    let mut num_posts = 0;
    let mut num_comments = 0;
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        num_users += 1;
        user.get("posts")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .for_each(|post| {
                num_posts += 1;
                post.get("comments")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .for_each(|comment| {
                        num_comments += 1;
                        assert_eq!(
                            comment.get("postid").unwrap().as_i64(),
                            post.get("id").unwrap().as_i64()
                        )
                    })
            });
    });
    //select count(*) from site_user
    assert_eq!(num_users, 16429, "Mismatched user count");

    //select count(*) from post where owneruserid is not null;
    assert_eq!(num_posts, 17575, "Mismatched post count");

    //select count(*) from post join comment on post.id = comment.postid where owneruserid is not null;
    assert_eq!(num_comments, 21630);
}

#[test]
fn join_foreign_field_not_last() {
    let gql_query = "
        query {
          siteUsers{
            id
            posts{
              id
              owneruserid
            }
            views
          }
        }";
    let p = convert_gql(gql_query);
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        let obj = user.as_object().unwrap();
        obj.get("views").unwrap().as_i64().unwrap();
        let posts = obj.get("posts").unwrap().as_array().unwrap();
        let user_id = obj.get("id").unwrap().as_i64().unwrap();
        for post in posts {
            assert_eq!(post.get("owneruserid").unwrap().as_i64().unwrap(), user_id)
        }
    })
}

//#[test]
//fn two_children_one_parent() {
//    let gql_query = "
//        query{
//          siteUsers{
//            id
//            comments{
//              score
//            }
//            posts{
//              title
//            }
//          }
//        }";
//    let res = convert_gql(gql_query);
//    let p: Result<Value, Error> = serde_json::from_str(&*res);
//    write_json_to_file(&res);
//    match p {
//        Ok(p) => {
//            let site_users = p.get("siteUsers").unwrap();
//            site_users.as_array().unwrap().iter().for_each(|user| {
//                let obj = user.as_object().unwrap();
//                obj.get("id").unwrap().as_i64().unwrap();
//                let comments = obj.get("comments").unwrap().as_array().unwrap();
//                println!("{}", comments.len());
//                let posts = obj.get("posts").unwrap().as_array().unwrap();
//                panic!("{}", posts.len());
//            })
//        }
//        Err(e) => panic!("{}", e),
//    }
//}

#[test]
fn weird_types_and_nullability() {
    let gql_query = "
        query{
          siteUsers{
            id
            creationdate
            aboutme
            jsonfield
            age
          }
        }";
    let p = convert_gql(gql_query);
}

#[test]
fn child_to_parent() {
    let gql_query = "
        query{
            posts{
                id
                score
                owneruserid
                siteUser{
                    displayname
                    id
                }
            }
        }";
    let p = convert_gql(gql_query);
    let posts = p.get("posts").unwrap().as_array().unwrap();
    for post in posts {
        let user = post.get("siteUser").unwrap();
        if user.is_null() {
            assert!(post.get("owneruserid").unwrap().is_null());
        } else {
            let user = user.as_object().unwrap();
            user.get("displayname").unwrap().as_str().unwrap();
            assert_eq!(
                post.get("owneruserid").unwrap().as_i64().unwrap(),
                user.get("id").unwrap().as_i64().unwrap()
            );
        }
    }
}

#[test]
fn composite_join() {
    let gql_query = "
            query{
              parentTables {
                id1
                id2
                childTables{
                  parentId1
                  parentId2
                }
              }
            }";
    let p = convert_gql(gql_query);
    for parent in p.get("parentTables").unwrap().as_array().unwrap() {
        let id1 = parent.get("id1").unwrap().as_i64();
        let id2 = parent.get("id2").unwrap().as_i64();
        for child in parent.get("childTables").unwrap().as_array().unwrap() {
            assert_eq!(id1, child.get("parentId1").unwrap().as_i64());
            assert_eq!(id2, child.get("parentId2").unwrap().as_i64());
        }
    }
}

#[test]
fn with_argument() {
    let gql_query = "
        query{
            siteUser(id: 13) {
                displayname
                posts{
                    title
                }
            }
        }";
    let p = convert_gql(gql_query);
    p.get("siteUser").unwrap().as_object().unwrap();
}

#[test]
fn invalid_id() {
    let gql_query = "
        query{
            siteUser(id: -1000) {
                displayname
                posts{
                    title
                }
            }
        }";
    let p = convert_gql(gql_query);
    p.get("siteUser").unwrap().as_null().unwrap();
}

//we add limit 0 to this query to ensure an empty query set, and check if we still return an empty
//array
#[test]
fn test_empty_many_query() {
    let gql_query = "
        query{
          siteUsers{
            id
            creationdate
            aboutme
            jsonfield
            age
          }
        }";
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    let rows = client.query(&*[sql, " limit 0"].concat(), &[]).unwrap();
    let res = JsonBuilder::new(ctx).convert(rows);
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    let users_len = p
        .unwrap()
        .get("siteUsers")
        .unwrap()
        .as_array()
        .unwrap()
        .len();
    assert_eq!(users_len, 0);
}

#[test]
fn test_select_one_compound() {
    let gql_query = "
        query{
          parentTable(id1: 0, id2: 10){
            id1
            id2
          }
        }
        ";
    let p = convert_gql(gql_query);
    let parent_table = p.get("parentTable").unwrap().as_object().unwrap();
    assert_eq!(parent_table.get("id1").unwrap().as_i64().unwrap(), 0);
    assert_eq!(parent_table.get("id2").unwrap().as_i64().unwrap(), 10);
}

#[test]
fn delete_one_id() {
    let mut client = mutation_test_fixtures();
    let gql_query = "
        mutation{
          deleteMutationTest(id: 1){
            nonNullableStr
          }
        }
        ";
    convert_gql(gql_query);
    if let Some(row) = client
        .query(
            "select non_nullable_str from mutation_test where id = 1",
            &[],
        )
        .unwrap()
        .get(0)
    {
        panic!("Got row {:?}", row)
    }

    //verify no other deletions
    let count: i64 = client
        .query("select count(*) from mutation_test", &[])
        .unwrap()
        .get(0)
        .unwrap()
        .get(0);
    assert_eq!(count, 99);
}
