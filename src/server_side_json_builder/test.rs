use super::*;
use crate::build_schema::internal_schema_info::create;

#[test]
fn select_string_fields() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            displayname
            aboutme
            posts{
              body
              title
            }
          }
        }";
    println!("{}", convert(query, &mut pogg));
}
