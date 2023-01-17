mod build_schema;
mod generate_sql;
mod state_machine_builder;
use crate::generate_sql::GraphQLSchema;
use tokio_postgres::NoTls;

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() {
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect("postgres://postgres:postgres@127.0.0.1:5432/pets", NoTls)
            .await
            .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let schema: GraphQLSchema = build_schema::create(&client).await;
    let gql_query = "
        query{
            siteUsers{
                id
                reputation
                views
                upvotes
                downvotes
            }
        }
    ";
    let ctx = schema.parse_graphql(gql_query).unwrap();
    let rows = client.query(&ctx.sql_query, &[]).await.unwrap();
    let mut builder = state_machine_builder::JsonBuilder::new(rows.iter(), ctx.table_metadata, &ctx.root_key_name);
    builder.exec_until_state_change();
    builder.exec_until_state_change();
    println!("{}", builder.s);
}
