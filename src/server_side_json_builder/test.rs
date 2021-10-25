use super::*;
use crate::build_schema::internal_schema_info::create;
use serde_json::{Error, Value};

#[test]
fn test_random_user() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            id
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
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    match p {
        Ok(p) => {
            let site_users = p.get("siteUsers").unwrap();

            //test specific user sampled at random
            assert!(site_users.as_array().unwrap().iter().any(|user| {
                let obj = user.as_object().unwrap();
                obj.get("reputation").unwrap() == 28971
                    && obj.get("views").unwrap() == 3534
                    && obj.get("upvotes").unwrap() == 4879
                    && obj.get("downvotes").unwrap() == 207
            }));
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn all_posts_fetched() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            id
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
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    match p {
        Ok(p) => {
            let site_users = p.get("siteUsers").unwrap();
            let num_posts = site_users.as_array().unwrap().iter().fold(0, |cumm, user| {
                let obj = user.as_object().unwrap();
                let posts = obj.get("posts").unwrap().as_array().unwrap();
                cumm + posts.len()
            });

            //select count(*) from post where post.owneruserid is not null;
            assert_eq!(num_posts, 17575);
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn all_posts_belong_to_parent() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            id
            reputation
            views
            upvotes
            downvotes
            posts{
              id
              posttypeid
              owneruserid
            }
          }
        }";
    let res = convert(query, &mut pogg);
    let p: Result<Value, Error> = serde_json::from_str(&*res);
    match p {
        Ok(p) => {
            let site_users = p.get("siteUsers").unwrap();
            site_users.as_array().unwrap().iter().for_each(|user| {
                let obj = user.as_object().unwrap();
                let user_id = obj.get("id").unwrap().as_i64();
                let posts = obj.get("posts").unwrap().as_array().unwrap();
                assert!(posts
                    .iter()
                    .all(|post| post.get("owneruserid").unwrap().as_i64() == user_id))
            });
        }
        Err(e) => panic!("{}", e),
    }
}
