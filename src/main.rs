mod build_schema;
mod handle_query;
mod server_side_json_builder;
use build_schema::internal_schema_info;
use handle_query::{postgres_query_builder::PostgresBuilder, Poggers};
use postgres::{Client, NoTls}; // 0.19.2, features = ["with-chrono-0_4"]

fn main() {
    panic!("");
    println!("{}", 0);
}
