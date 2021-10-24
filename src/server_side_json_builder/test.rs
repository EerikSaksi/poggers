use super::*;
use crate::build_schema::internal_schema_info::create;

#[test]
fn select_post_ids() {
    let pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            displayname
            aboutme
            postsByOwneruserid{
              body
              title
            }
          }
        }";
    convert(query, &mut pogg);
}
