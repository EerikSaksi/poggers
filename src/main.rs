mod build_schema;
mod handle_query;
mod server_side_json_builder;
use async_graphql::check_rules;
use async_graphql::parser::parse_query;

fn main() {
    let pogg = build_schema::create(
        "postgres://postgres:postgres@localhost:5432/pets",
    );
    //let registry = build_schema::internal_to_async(pogg);
    //let query = "
    //    query {
    //        users{
    //            id
    //        }
    //    }";
    //let doc = parse_query(query).unwrap();
    //match check_rules(&registry, &doc, None, async_graphql::ValidationMode::Strict) {
    //    Ok(_) => println!("No errors"),
    //    Err(e) => println!("Got error {:?}", e),
    //}
}
