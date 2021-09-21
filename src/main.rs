mod build_schema;
mod handle_query;
use std::time::Instant;

fn main() {
    let expected = "
            query {
                exercises {
                    bodyPart
                }
            }
";
    let before = Instant::now();
    match handle_query::build_root(expected) {
        Ok(val) => println!("{}", val),
        Err(e) => println!("{}", e),
    }
    println!("Elapsed time: {:.2?}", before.elapsed());
}
