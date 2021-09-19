mod build_schema;
mod handle_query;

fn main() {
    let expected = "
            query {
                exercise(id: 1) {
                    bodyPart
                }
            }
";
    println!("{}", handle_query::build_root(expected).unwrap());
}
