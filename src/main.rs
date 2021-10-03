use build_schema::internal_schema_info;
use handle_query::Poggers;
mod async_implementation;
mod build_schema;
mod handle_query;

fn main() {
    let (type_graph, query_to_type) = build_schema::internal_schema_info::create();
    let pog = Poggers {
        type_graph,
        query_to_type,
    };
    println!(
        "{}",
        pog.build_root(
            "
              query {
                exercises {
                  bodyPart
                }
              }
            ",
        )
        .unwrap()
    )
}
