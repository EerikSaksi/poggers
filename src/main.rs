mod build_schema;
mod handle_query;
mod postgres_client;
use build_schema::internal_schema_info;
use postgres::{Client, NoTls, Row};
fn main() {
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls).unwrap();
    //https://stackoverflow.com/questions/1152260/how-to-list-table-foreign-keys
    let query_res = client.query(
        "
",
        &[],
    ).unwrap();
}
