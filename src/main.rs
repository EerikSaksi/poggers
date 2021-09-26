use crate::handle_query::SqlOperation;
use handle_query::Poggers;
use std::collections::HashMap;
mod async_implementation;
mod build_schema;
mod handle_query;

#[tokio::main]
async fn main() {
    build_schema::create_schema();
}
