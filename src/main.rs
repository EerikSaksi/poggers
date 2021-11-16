mod build_schema;
mod postgraphile_introspection;
mod server_side_json_builder;
use postgraphile_introspection::make_instrospection_query;
use postgres::{Client, NoTls};
use serde_json::Value;

fn main() {
    let mut client =
        Client::connect("postgres://postgres:postgres@localhost:5432/pets", NoTls).unwrap();
    let rows = client
        .query(&make_instrospection_query(99999999, false, false), &[])
        .unwrap();
    for row in rows {
        let json: Value = row.get(0);
        if let Value::Object(obj) = json {
            println!(
                "{:?}",
                obj.keys()
                    .fold(String::new(), |cumm, k| format!("{}, {}", cumm, k))
            )
        }
    }
}
