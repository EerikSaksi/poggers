use std::collections::HashSet;
use super::*;
use tokio_postgres::Client;
use serde_json::Value;
use crate::build_schema::get_schema_and_client;
async fn convert_gql(gql_query: &str) -> (GraphQLSchema, Client, Value) {
    let (schema, client) = get_schema_and_client().await;
    let ctx = schema.parse_graphql(gql_query).unwrap();
    println!("{}", ctx.sql_query);
    let rows = client.query(&ctx.sql_query, &[]).await.unwrap();
    let mut builder = JsonBuilder::new(rows.iter(), ctx.table_metadata, &ctx.root_key_name);
    builder.exec_until_state_change();
    (schema, client, serde_json::from_str(&*&builder.s).unwrap())
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
    let (_, _, p) = convert_gql(gql_query).await;
    let site_users = p.get("siteUsers").unwrap();
    site_users.as_array().unwrap().iter().for_each(|user| {
        //test non nullable fields defined for all users
        let obj = user.as_object().unwrap();
        obj.get("id").unwrap().as_i64().unwrap();
        obj.get("displayname").unwrap().as_str().unwrap();
    });
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

    let (_, client, p) = convert_gql(gql_query).await;

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
