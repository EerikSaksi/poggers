mod handle_query;
mod build_schema;
fn main() {
    //println!("{}", handle_query::build_root());
    println!("{:?}", build_schema::client_connect());
}
