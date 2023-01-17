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
