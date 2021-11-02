use super::*;
use crate::build_schema::internal_schema_info::create;
use postgres::Client;
use postgres::NoTls;
use serde_json::{Error, Value};

fn convert_gql(gql_query: &str) -> String {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let (sql_query, table_query_infos, root_key_name) = pogg.build_root(gql_query).unwrap();
    let rows = client.query(&*[&sql_query, ""].concat(), &[]).unwrap();
    let to_return = JsonBuilder::new(table_query_infos, root_key_name).convert(rows);
    to_return
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

    let res = convert_gql(gql_query);

    let p: Result<Value, Error> = serde_json::from_str(&*res);
    match p {
        Ok(p) => {
            let site_users = p.get("siteUsers").unwrap();
            //test specific user sampled at random
            assert!(site_users.as_array().unwrap().iter().any(|user| {
                let obj = user.as_object().unwrap();
                obj.get("reputation").unwrap() == 28971
                    && obj.get("views").unwrap() == 3534
                    && obj.get("upvotes").unwrap() == 4879
                    && obj.get("downvotes").unwrap() == 207
            }));
        }
        Err(e) => panic!("{}", e),
    }
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
    let res = convert_gql(gql_query);
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    match p {
        Ok(p) => {
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
        Err(e) => panic!("{}", e),
    }
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
    let res = convert_gql(gql_query);
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    match p {
        Ok(p) => {
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
        Err(e) => panic!("{}", e),
    }
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
    let res = convert_gql(gql_query);
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    match p {
        Ok(p) => {
            let site_users = p.get("siteUsers").unwrap();
            site_users.as_array().unwrap().iter().for_each(|user| {
                //test non nullable fields defined for all users
                let obj = user.as_object().unwrap();
                obj.get("id").unwrap().as_i64().unwrap();
                obj.get("displayname").unwrap().as_str().unwrap();
            });
        }
        Err(e) => panic!("{}", e),
    }
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
    let res = convert_gql(gql_query);
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    let mut num_users = 0;
    let mut num_posts = 0;
    let mut num_comments = 0;
    match p {
        Ok(p) => {
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
        }
        Err(e) => panic!("{}", e),
    }
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
              owneruserid
            }
            views
          }
        }";
    let res = convert_gql(gql_query);
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    assert!(p.is_ok(), "{}", p.unwrap_err());
}
