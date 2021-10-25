use super::*;
use crate::build_schema::internal_schema_info::create;
use serde_json::{Error, Value};
use std::fs::File;
use std::io::prelude::*;

#[test]
fn select_integer_fields() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            reputation
            views
            upvotes
            downvotes
            posts{
              id
              posttypeid
            }
          }
        }";
    let res = convert(query, &mut pogg);
    //let p: Result<Value, Error> = serde_json::from_str(&*res);
    //let mut file = File::create("foo.json").unwrap();
    //file.write_all(res.as_bytes()).unwrap();
    //match p {
    //    Ok(p) => println!("{}", p),
    //    Err(e) => panic!("{}", e),
    //}
    panic!();
}
