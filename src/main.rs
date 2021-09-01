mod build_schema;
mod handle_query;
mod async_implementation;

use async_graphql::*;

struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

fn main(){
    build_schema::client_connect();
}
