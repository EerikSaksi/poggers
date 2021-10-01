use handle_query::Poggers;
use std::collections::HashMap;
use build_schema::internal_schema_info;
mod async_implementation;
mod build_schema;
mod handle_query;

fn main() {
    let g = internal_schema_info::create();
    println!("hello");
}
