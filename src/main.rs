mod build_schema;
mod handle_query;
mod server_side_json_builder;
use build_schema::internal_schema_info;

fn main() {
    server_side_json_builder::build_json_server_side().unwrap();
    server_side_json_builder::postgraphile_query().unwrap();
}
