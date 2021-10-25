mod build_schema;
mod handle_query;
use build_schema::internal_schema_info;
use handle_query::{postgres_query_builder::PostgresBuilder, Poggers};
use postgres::{Client, NoTls};
mod server_side_json_builder;

fn main() {
    let serverside_pogg = build_schema::internal_schema_info::create(
        "postgres://eerik:Postgrizzly@localhost:5432/pets",
    );
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    use std::time::Instant;
    let before = Instant::now();
    let mut pogg = Poggers {
        g: serverside_pogg.g,
        local_id: 0,
        query_to_type: serverside_pogg.query_to_type,
        query_builder: PostgresBuilder {},
    };

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

    let rows = client.query(&*sql, &[]).unwrap();
    println!("Elapsed time: {:.2?}", before.elapsed());
    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::create("just_use_it.txt").unwrap();
    file.write_all(&format!("{:?}", rows.get(0).unwrap()).as_bytes())
        .unwrap();
}
