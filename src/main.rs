mod async_implementation;
mod build_schema;
mod handle_query;

#[tokio::main]
async fn main() {
    println!("{}", async_implementation::execute_add().await);
}
