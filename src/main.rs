use crate::handle_query::SqlOperation;
use handle_query::Poggers;
use std::collections::HashMap;
mod async_implementation;
mod build_schema;
mod handle_query;

#[tokio::main]
async fn main() {
    let mut graphql_query_to_operation = HashMap::new();
    graphql_query_to_operation.insert(
        String::from("exercise"),
        SqlOperation {
            is_many: false,
            table_name: "exercise",
        },
    );
    let pogg = Poggers {
        graphql_query_to_operation,
    };
    println!(
        "{}",
        pogg.build_root(
            "
            query {
              exercise(id: 123) {
                bodyPart
              } 
            }",
        ).unwrap()
    );
}
