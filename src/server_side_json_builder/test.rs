use super::*;
extern crate test;
use crate::build_schema::create;
use postgres::{Client, NoTls}; // 0.19.2, features = ["with-chrono-0_4"]
use serde_json::{Error, Value};
use std::collections::HashSet;
use test::Bencher;
fn convert_gql(gql_query: &str, write_to_file: bool) -> Value {
    let pogg = create();
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    println!("\n{}\n", sql);
    let rows = client.query(&*[sql, ""].concat(), &[]).unwrap();
    let res = JsonBuilder::new(ctx).convert(&rows);
    if write_to_file {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create("foo.json").unwrap();
        file.write_all(res.as_bytes()).unwrap();
    }
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
    let values = (0..100)
        .map(|i| format!("({}, '{}', {})", i, i, (i / 10)))
        .collect::<Vec<String>>()
        .join(", ");
    client
        .query(
            &format!(
                "insert into mutation_test_child(id, name, mutation_test_id) values {}",
                values
            ),
            &[],
        )
        .unwrap();
    client
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
            postsByOwneruserid{
              id
              posttypeid
            }
          }
        }";

    let p = convert_gql(gql_query, true);

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
    assert_eq!(
        user.get("postsByOwneruserid")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        535
    );

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
    assert_eq!(
        user.get("postsByOwneruserid")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        535
    );
    let mut post_set = HashSet::new();
    for post in user.get("postsByOwneruserid").unwrap().as_array().unwrap() {
        let id = post.get("id").unwrap().as_i64().unwrap();
        assert!(!post_set.insert(id), "Post with id {} was already in", id);
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
            postsByOwneruserid{
              id
              posttypeid
            }
          }
        }";
    let p = convert_gql(gql_query, false);
    let mut num_users = 0;
    let site_users = p.get("siteUsers").unwrap();
    let num_posts = site_users.as_array().unwrap().iter().fold(0, |cumm, user| {
        num_users += 1;
        let obj = user.as_object().unwrap();
        let posts = obj.get("postsByOwneruserid").unwrap().as_array().unwrap();
        cumm + posts.len()
    });
    assert_eq!(
        num_users,
        16429,
        "Missing {} users. 10512 users don't have posts (did you left join)",
        16429 - num_users
    );

    //select count(*) from post where post.owneruserid is not null;
    assert_eq!(num_posts, 17575);
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
            postsByOwneruserid{
              id
              posttypeid
              owneruserid
            }
          }
        }";
    let p = convert_gql(gql_query, false);
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        let obj = user.as_object().unwrap();
        let user_id = obj.get("id").unwrap().as_i64();
        let posts = obj.get("postsByOwneruserid").unwrap().as_array().unwrap();
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
    let p = convert_gql(gql_query, false);
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
            postsByOwneruserid{
              id
              posttypeid
              owneruserid
              commentsByPostid{
                id
                score
                postid
                text
              }
            }
          }
        }";
    let p = convert_gql(gql_query, false);
    let mut num_users = 0;
    let mut num_posts = 0;
    let mut num_comments = 0;
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        num_users += 1;
        user.get("postsByOwneruserid")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .for_each(|post| {
                num_posts += 1;
                post.get("commentsByPostid")
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
            postsByOwneruserid{
              id
              owneruserid
            }
            views
          }
        }";
    let p = convert_gql(gql_query, false);
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        let obj = user.as_object().unwrap();
        obj.get("views").unwrap().as_i64().unwrap();
        let posts = obj.get("postsByOwneruserid").unwrap().as_array().unwrap();
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
    convert_gql(gql_query, false);
}

#[test]
fn child_to_parent() {
    let gql_query = "
        query{
            posts{
                id
                score
                owneruserid
                siteUserByOwneruserid{
                    displayname
                    id
                }
            }
        }";
    let p = convert_gql(gql_query, false);
    let posts = p.get("posts").unwrap().as_array().unwrap();
    for post in posts {
        let user = post.get("siteUserByOwneruserid").unwrap();
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
                childTablesByParentId1AndParentId2{
                  parentId1
                  parentId2
                }
              }
            }";
    let p = convert_gql(gql_query, false);
    for parent in p.get("parentTables").unwrap().as_array().unwrap() {
        let id1 = parent.get("id1").unwrap().as_i64();
        let id2 = parent.get("id2").unwrap().as_i64();
        for child in parent
            .get("childTablesByParentId1AndParentId2")
            .unwrap()
            .as_array()
            .unwrap()
        {
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
                postsByOwneruserid{
                    title
                }
            }
        }";
    let p = convert_gql(gql_query, false);
    p.get("siteUser").unwrap().as_object().unwrap();
}

#[test]
fn invalid_id() {
    let gql_query = "
        query{
            siteUser(id: -1000) {
                displayname
                postsByOwneruserid{
                    title
                }
            }
        }";
    let p = convert_gql(gql_query, false);
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
    let pogg = create();
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    let rows = client.query(&*[sql, " limit 0"].concat(), &[]).unwrap();
    let res = JsonBuilder::new(ctx).convert(&rows);
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
    let p = convert_gql(gql_query, false);
    let parent_table = p.get("parentTable").unwrap().as_object().unwrap();
    assert_eq!(parent_table.get("id1").unwrap().as_i64().unwrap(), 0);
    assert_eq!(parent_table.get("id2").unwrap().as_i64().unwrap(), 10);
}

//kinda janky but these need to run sequentially
#[test]
fn mutation_tests() {
    let mut client = mutation_test_fixtures();
    let gql_query = "
        mutation{
          deleteMutationTest(id: 1){
            nonNullableStr
          }
        }
        ";
    let p = convert_gql(gql_query, false);
    if let Some(row) = client
        .query(
            "select non_nullable_str from mutation_test where id = 1",
            &[],
        )
        .unwrap()
        .get(0)
    {
        panic!("Got row {:?}, expected row to be deleted", row)
    }

    assert_eq!(
        p.get("deleteMutationTest")
            .unwrap()
            .as_object()
            .unwrap()
            .get("nonNullableStr")
            .unwrap()
            .as_str()
            .unwrap(),
        "1"
    );

    let gql_query = "
        mutation{
          deleteMutationTest(id: 2){
              id 
              postByPostId{
                  title
              }
          }
        }
        ";
    let p = convert_gql(gql_query, false);
    p.get("deleteMutationTest")
        .unwrap()
        .get("postByPostId")
        .unwrap()
        .as_object()
        .unwrap()
        .get("title")
        .unwrap()
        .as_str();

    assert_eq!(
        p.get("deleteMutationTest")
            .unwrap()
            .get("id")
            .unwrap()
            .as_i64()
            .unwrap(),
        2
    );

    let mut client = mutation_test_fixtures();
    let gql_query = "
        mutation{
          updateMutationTest(id: 3, patch: {nullableFloat: 1.23}){
            nullableFloat
          }
        }
    ";
    convert_gql(gql_query, false);
    let rows = client
        .query("select nullable_float from mutation_test where id = 3", &[])
        .unwrap();
    let nullable_float: f64 = rows.get(0).unwrap().get(0);
    assert_eq!(nullable_float, 1.23);

    let gql_query = "
        mutation{
          updateMutationTest(id: 4, patch: {nonNullableStr: \"newValue\"}){
            nonNullableStr
          }
        }
    ";
    convert_gql(gql_query, false);
    let rows = client
        .query(
            "select non_nullable_str from mutation_test where id = 4",
            &[],
        )
        .unwrap();
    let new_value: &str = rows.get(0).unwrap().get(0);
    assert_eq!(new_value, "newValue");

    let gql_query = "
        mutation{
          updateMutationTest(id: 5, patch: {nonNullableStr: \"'newValue'\"}){
            nonNullableStr
          }
        }
    ";
    convert_gql(gql_query, false);
    let rows = client
        .query(
            "select non_nullable_str from mutation_test where id = 5",
            &[],
        )
        .unwrap();
    let new_value: &str = rows.get(0).unwrap().get(0);
    assert_eq!(new_value, "'newValue'");

    let gql_query = "
        mutation{
          updateMutationTest(id: 6, patch: {nonNullableStr: \"'asdf'\", nullableFloat: 6.89}){
              nonNullableStr
              nullableFloat
          }
        }
    ";
    match convert_gql(gql_query, false)
        .get("updateMutationTest")
        .unwrap()
        .as_object()
    {
        Some(update_mutation_test) => {
            assert_eq!(
                update_mutation_test
                    .get("nonNullableStr")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                "'asdf'"
            );
            assert_eq!(
                update_mutation_test
                    .get("nullableFloat")
                    .unwrap()
                    .as_f64()
                    .unwrap(),
                6.89
            );
        }
        None => panic!("update_mutation_test failed"),
    }
    let rows = client
        .query(
            "select non_nullable_str, nullable_float from mutation_test where id = 6",
            &[],
        )
        .unwrap();
    let new_str: &str = rows.get(0).unwrap().get("non_nullable_str");
    assert_eq!(new_str, "'asdf'");
    let float: f64 = rows.get(0).unwrap().get("nullable_float");
    assert_eq!(float, 6.89);

    let gql_query = "
        mutation{
          insertMutationTest(id: 102, nonNullableStr: \"inserted\", nullableFloat: 432.10){
              nonNullableStr
              nullableFloat
          }
        }
    ";
    convert_gql(gql_query, false);
    let rows = client
        .query(
            "select non_nullable_str, nullable_float from mutation_test where id = 102",
            &[],
        )
        .unwrap();
    let name: &str = rows.get(0).unwrap().get("non_nullable_str");
    assert_eq!(name, "inserted");

    let gql_query = "
        mutation{
          deleteMutationTest(id: 5){
            id
            mutationTestChildsByMutationTestId {
                mutationTestId
                name
            }
          }
        }
    ";
    if let Some(res) = convert_gql(gql_query, false).get("deleteMutationTest") {
        let id = res.get("id").unwrap().as_i64().unwrap();
        for child in res
            .get("mutationTestChildsByMutationTestId")
            .unwrap()
            .as_array()
            .unwrap()
        {
            assert_eq!(id, child.get("mutationTestId").unwrap().as_i64().unwrap());
        }
    } else {
        panic!("Couldn't parse deleteMutationTest");
    }
}
#[bench]
fn bench_safe_builder(b: &mut Bencher) {
    let pogg = create();
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();

    let gql_query = "
        query {
          siteUsers{
            id
            reputation
            views
            upvotes
            downvotes
            postsByOwneruserid{
              id
              posttypeid
              owneruserid
              commentsByPostid{
                id
                score
                postid
                text
              }
            }
          }
        }";
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    println!("\n{}\n", sql);
    let rows = client.query(&*[sql, ""].concat(), &[]).unwrap();
    let builder = JsonBuilder::new(ctx);
    b.iter(|| {
        let _ = test::black_box(builder.convert(&rows));
    });
}

#[test]
fn many_where_clause() {
    let gql_query = "
        query{
          posts(where: {score: 1, commentcount:2, tags: \"<cats>\"}){
            id
          }
        }
        ";
    let p = convert_gql(gql_query, false);
    assert_eq!(p.get("posts").unwrap().as_array().unwrap().len(), 11);
}

#[test]
fn where_string_escape() {
    let gql_query = "
        query{
          posts(where: {tags: \"''\"}){
            id
          }
        }
        ";
    convert_gql(gql_query, false);
}
