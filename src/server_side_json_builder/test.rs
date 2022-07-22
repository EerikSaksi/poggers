use super::*;
use crate::build_schema::get_pogg_and_client;
use deadpool_postgres::tokio_postgres::Client;
use serde_json::{Error, Value};
use std::collections::HashSet;

//quite a few queries need to run a graphql query. We made this function for those cases to
//provide the parsed json, in addition to the client and poggers
async fn convert_gql(gql_query: &str, write_to_file: bool) -> (ServerSidePoggers, Client, Value) {
    let (pogg, client) = get_pogg_and_client().await;
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    println!("\n{}\n", sql);
    let rows = client.query(sql, &[]).await.unwrap();
    let res = JsonBuilder::new(ctx).convert(rows);

    if write_to_file {
        use std::fs::File;
        use std::io::prelude::*;
        let mut file = File::create("foo.json").unwrap();
        file.write_all(res.as_bytes()).unwrap();
    }
    (pogg, client, serde_json::from_str(&*res).unwrap())
}

async fn mutation_test_fixtures() -> Client {
    let (_, client) = get_pogg_and_client().await;

    //create the mutation test table which is used to ensure that mutations work properly
    client
        .query(
            "
            create or replace table mutation_test(
              id integer primary key generated always as identity,
              non_nullable_str varchar not null, 
              nullable_float float,
              post_id integer references post(id))
            )
        ",
            &[],
        )
        .await
        .unwrap();

    //get post_ids, allowing us to insert values in the post_id column whilst
    //adhering to the foreign id constraint
    let post_ids = client
        .query("select id from post limit 100", &[])
        .await
        .unwrap();

    let values = (0..100)
        .map(|i| {
            let post_id: i32 = post_ids.get(i).unwrap().get(0);
            format!("({}, '{}', {}.5, {})", i, i, i, post_id)
        })
        .collect::<Vec<String>>()
        .join(", ");

    client
        .query(
            &*format!(
                "insert into mutation_test(id, non_nullable_str, nullable_float, post_id) values {}",
                values
            ),
            &[],
        )
        .await
        .unwrap();
    let values = (0..100)
        .map(|i| format!("({}, '{}', {})", i, i, (i / 10)))
        .collect::<Vec<String>>()
        .join(", ");

    client
        .query(
            "
            create or replace table mutation_test_child(
              id integer primary key generated always as identity,
              name varchar,
              mutation_test_id integer references mutation_test(id)
            )
        ",
            &[],
        )
        .await
        .unwrap();

    client
        .query(
            &*format!(
                "insert into mutation_test_child(id, name, mutation_test_id) values {}",
                values
            ),
            &[],
        )
        .await
        .unwrap();
    client
}

#[actix_rt::test]
async fn test_random_user() {
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

    let (_, client, p) = convert_gql(gql_query, false).await;

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

    match client
        .query(
            "SELECT reputation, views, upvotes, downvotes FROM site_user where id = 13",
            &[],
        )
        .await
    {
        Ok(user_query) => {
            let user_row = user_query.get(0).unwrap();

            assert_eq!(
                user.get("reputation").unwrap(),
                user_row.get::<usize, i32>(0)
            );
            assert_eq!(user.get("views").unwrap(), user_row.get::<usize, i32>(1));
            assert_eq!(user.get("upvotes").unwrap(), user_row.get::<usize, i32>(2));
            assert_eq!(
                user.get("downvotes").unwrap(),
                user_row.get::<usize, i32>(3)
            );
        }
        Err(e) => panic!("{}", e),
    }

    let count: i64 = client
        .query(
            "SELECT count(*) FROM site_user join post on post.owneruserid = site_user.id where site_user.id = 13",
            &[],
        )
        .await
        .unwrap()
        .get(0)
        .unwrap()
        .get(0);
    assert_eq!(
        user.get("postsByOwneruserid")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        count as usize
    );

    //check no duplicates in posts
    let mut post_set = HashSet::new();
    for post in user.get("postsByOwneruserid").unwrap().as_array().unwrap() {
        let post_id = post.get("id").unwrap().as_i64().unwrap();
        assert!(
            post_set.insert(post_id),
            "Post with id {} was already in",
            post_id
        );
    }
}

#[actix_rt::test]
async fn all_posts_fetched() {
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
    let (_, client, p) = convert_gql(gql_query, false).await;
    let site_users = p.get("siteUsers").unwrap();
    let num_users = site_users.as_array().unwrap().len();
    let num_posts = site_users.as_array().unwrap().iter().fold(0, |cumm, user| {
        let posts = user
            .as_object()
            .unwrap()
            .get("postsByOwneruserid")
            .unwrap()
            .as_array()
            .unwrap();
        cumm + posts.len()
    });
    let real_count: i64 = client
        .query("SELECT count(*) from site_user", &[])
        .await
        .unwrap()
        .get(0)
        .unwrap()
        .get(0);
    assert_eq!(num_users, real_count as usize);

    let real_count: i64 = client
        //we add the is not null as posts without owneruserid would not be included in the GraphQL
        //query
        .query("SELECT count(*) from post where owneruserid is not null", &[])
        .await
        .unwrap()
        .get(0)
        .unwrap()
        .get(0);
    assert_eq!(num_posts, real_count as usize);
}

#[actix_rt::test]
async fn all_posts_belong_to_parent() {
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
    let (_, _, p) = convert_gql(gql_query, false).await;
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

#[actix_rt::test]
async fn non_nullable_string_fields() {
    let gql_query = "
        query {
          siteUsers{
            id
            displayname
          }
        }";
    let (_, _, p) = convert_gql(gql_query, false).await;
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        //test non nullable fields defined for all users
        let obj = user.as_object().unwrap();
        obj.get("id").unwrap().as_i64().unwrap();
        obj.get("displayname").unwrap().as_str().unwrap();
    });
}

#[actix_rt::test]
async fn three_way_join() {
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
              }
            }
          }
        }";

    let (pogg, client) = get_pogg_and_client().await;
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    println!("\n{}\n", sql);
    let rows = client.query(&*[sql, ""].concat(), &[]).await.unwrap();
    let res = JsonBuilder::new(ctx).convert(rows);
    let p: Value = serde_json::from_str(&*res).unwrap();

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

    let expected_count: i64 = client
        .query("select count(*) from site_user", &[])
        .await
        .unwrap()
        .get(0)
        .unwrap()
        .get(0);
    assert_eq!(num_users, expected_count, "Mismatched user count");

    let expected_count: i64 = client
        .query(
            "select count(*) from post where owneruserid is not null;",
            &[],
        )
        .await
        .unwrap()
        .get(0)
        .unwrap()
        .get(0);
    assert_eq!(num_posts, expected_count, "Mismatched post count");

    let expected_count: i64= client
        .query("select count(*) from post join comment on post.id = comment.postid where owneruserid is not null;", &[])
        .await
        .unwrap()
        .get(0)
        .unwrap()
        .get(0);
    assert_eq!(num_comments, expected_count);
}

#[actix_rt::test]
async fn join_foreign_field_not_last() {
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
    let (_, _, p) = convert_gql(gql_query, false).await;
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

//#[actix_rt::test]
//async fn two_children_one_parent() {
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
//    let res = convert_gql(gql_query).await;
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

#[actix_rt::test]
async fn weird_types_and_nullability() {
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
    convert_gql(gql_query, false).await;
}

#[actix_rt::test]
async fn child_to_parent() {
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
    let (_, _, p) = convert_gql(gql_query, false).await;
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

#[actix_rt::test]
async fn composite_join() {
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
    let (_, _, p) = convert_gql(gql_query, false).await;
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

#[actix_rt::test]
async fn with_argument() {
    let gql_query = "
        query{
            siteUser(id: 13) {
                displayname
                postsByOwneruserid{
                    title
                }
            }
        }";
    let (_, _, p) = convert_gql(gql_query, false).await;
    p.get("siteUser").unwrap().as_object().unwrap();
}

#[actix_rt::test]
async fn invalid_id() {
    let gql_query = "
        query{
            siteUser(id: -1000) {
                displayname
                postsByOwneruserid{
                    title
                }
            }
        }";
    let (_, _, p) = convert_gql(gql_query, false).await;
    p.get("siteUser").unwrap().as_null().unwrap();
}

//we add limit 0 to this query to ensure an empty query set, and check if we still return an empty
//array
#[actix_rt::test]
async fn test_empty_many_query() {
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
    let (pogg, client) = get_pogg_and_client().await;
    let ctx = pogg.build_root(gql_query).unwrap();
    let sql = &ctx.sql_query;
    let rows = client
        .query(&*[sql, " limit 0"].concat(), &[])
        .await
        .unwrap();
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

#[actix_rt::test]
async fn test_select_one_compound() {
    let gql_query = "
        query{
          parentTable(id1: 0, id2: 10){
            id1
            id2
          }
        }
        ";
    let (_, _, p) = convert_gql(gql_query, false).await;
    let parent_table = p.get("parentTable").unwrap().as_object().unwrap();
    assert_eq!(parent_table.get("id1").unwrap().as_i64().unwrap(), 0);
    assert_eq!(parent_table.get("id2").unwrap().as_i64().unwrap(), 10);
}

//kinda janky but these need to run sequentially
#[actix_rt::test]
async fn mutation_tests() {
    let client = mutation_test_fixtures().await;
    let gql_query = "
        mutation{
          deleteMutationTest(id: 1){
            nonNullableStr
          }A
        }
        ";
    let (_, _, p) = convert_gql(gql_query, false).await;
    if let Some(row) = client
        .query(
            "select non_nullable_str from mutation_test where id = 1",
            &[],
        )
        .await
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
    let (_, _, p) = convert_gql(gql_query, false).await;
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

    let client = mutation_test_fixtures().await;
    let gql_query = "
        mutation{
          updateMutationTest(id: 3, patch: {nullableFloat: 1.23}){
            nullableFloat
          }
        }
    ";
    convert_gql(gql_query, false).await;
    let rows = client
        .query("select nullable_float from mutation_test where id = 3", &[])
        .await
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
    convert_gql(gql_query, false).await;
    let rows = client
        .query(
            "select non_nullable_str from mutation_test where id = 4",
            &[],
        )
        .await
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
    convert_gql(gql_query, false).await;
    let rows = client
        .query(
            "select non_nullable_str from mutation_test where id = 5",
            &[],
        )
        .await
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
        .await
        .2
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
        .await
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
    convert_gql(gql_query, false).await;
    let rows = client
        .query(
            "select non_nullable_str, nullable_float from mutation_test where id = 102",
            &[],
        )
        .await
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
    if let Some(res) = convert_gql(gql_query, false)
        .await
        .2
        .get("deleteMutationTest")
    {
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

#[actix_rt::test]
async fn many_where_clause() {
    let gql_query = "
        query{
          posts(where: {score: 1, commentcount:2, tags: \"<cats>\"}){
            id
          }
        }
        ";
    let (_, _, p) = convert_gql(gql_query, false).await;
    assert_eq!(p.get("posts").unwrap().as_array().unwrap().len(), 11);
}

#[actix_rt::test]
async fn where_string_escape() {
    let gql_query = "
        query{
          posts(where: {tags: \"''\"}){
            id
          }
        }
        ";
    convert_gql(gql_query, false).await;
}
