mod build_schema;
mod handle_query;
mod server_side_json_builder;
use crate::handle_query::postgres_query_builder::PostgresBuilder;
use crate::handle_query::Poggers;
use crate::internal_schema_info::create;
use crate::server_side_json_builder::JsonBuilder;
use build_schema::internal_schema_info;
use postgres::Client;
use postgres::NoTls;
fn main() {
    let serverside_pogg = build_schema::internal_schema_info::create(
        "postgres://eerik:Postgrizzly@localhost:5432/pets",
    );
    let mut pogg = Poggers {
        g: serverside_pogg.g,
        local_id: 0,
        query_to_type: serverside_pogg.query_to_type,
        query_builder: PostgresBuilder {},
    };
    use std::time::Instant;
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let before = Instant::now();

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
    let sql = pogg.build_root(query).unwrap();
    let res = client.query(&*sql, &[]).unwrap();
    println!("Elapsed time: {:.2?}", before.elapsed());
}
