mod build_schema;
mod handle_query;
use build_schema::internal_schema_info;
use handle_query::{postgres_query_builder::PostgresBuilder, Poggers};
use postgres::{Client, NoTls};
mod server_side_json_builder;

fn main() {
    let mut millis: u128 = 0;

    let mut serverside_pogg = build_schema::internal_schema_info::create(
        "postgres://eerik:Postgrizzly@localhost:5432/pets",
    );
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    use std::time::Instant;

    let query = "
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
    for _ in 0..100 {
        let before = Instant::now();
        let sql = server_side_json_builder::convert(query, &mut serverside_pogg);
        let rows = client.query(&*sql, &[]).unwrap();
        millis += before.elapsed().as_millis();
        println!("{:?}", rows);
        serverside_pogg.local_id = 0;
    }
    println!("Milli seconds {}", millis / 100);
}
