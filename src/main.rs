mod build_schema;
mod server_side_json_builder;
mod state_machine_builder;
use tokio_postgres::{Error, NoTls};

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() {
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect("postgres://postgres:postgres@127.0.0.1:5432/pets", NoTls)
            .await
            .unwrap();
    gener
}
