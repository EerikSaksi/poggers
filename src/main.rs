mod build_schema;
mod handle_query;
mod server_side_json_builder;
use crate::internal_schema_info::create;
use crate::server_side_json_builder::JsonBuilder;
use build_schema::internal_schema_info;
use postgres::Client;
use postgres::NoTls;
fn main() {
    use std::time::Instant;
    let before = Instant::now();

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

    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let mut total_millis = 0;
    for _ in 0..10000 {
        let (sql_query, table_query_infos, root_key_name) = pogg.build_root(gql_query).unwrap();
        let rows = client.query(&*[&sql_query, ""].concat(), &[]).unwrap();
        let before = Instant::now();
        JsonBuilder::new(table_query_infos, root_key_name).convert(rows);
        total_millis += before.elapsed().as_millis();
    }
    println!("Without null check for first join {}", total_millis / 10000);
    println!("Elapsed time: {:.2?}", before.elapsed());
}
