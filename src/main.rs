mod build_schema;
mod handle_query;
use build_schema::internal_schema_info;
use handle_query::{postgres_query_builder::PostgresBuilder, Poggers};
use postgres::{Client, NoTls};
mod server_side_json_builder;
use std::time::Instant;

fn main() {
    let mut serverside_pogg = build_schema::internal_schema_info::create(
        "postgres://eerik:Postgrizzly@localhost:5432/pets",
    );
    let mut poggg = build_schema::internal_schema_info::create(
        "postgres://eerik:Postgrizzly@localhost:5432/pets",
    );

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

    let mut database_side_times: Vec<u128> = Vec::new();

    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    {
        let mut pogg = Poggers {
            query_builder: PostgresBuilder {},
            local_id: 0,
            query_to_type: poggg.query_to_type,
            g: poggg.g,
        };
        let sql = "select
  to_json(json_build_array(__local_0__.\"id\"))::text as \"__identifiers\",
  to_json((__local_0__.\"id\"))::text as \"id\",
  to_json((__local_0__.\"reputation\"))::text as \"reputation\",
  to_json((__local_0__.\"views\"))::text as \"views\",
  to_json((__local_0__.\"upvotes\"))::text as \"upvotes\",
  to_json((__local_0__.\"downvotes\"))::text  as \"downvotes\",
  to_json(
    (
      select
        coalesce(
          (
            select
              json_agg(__local_1__.\"object\")
            from
              (
                select
                  json_build_object(
                    '__identifiers':: text,
                    json_build_array(__local_2__.\"id\"),
                    'id':: text,
                    (__local_2__.\"id\"),
                    'posttypeid':: text,
                    (__local_2__.\"posttypeid\")
                  ) as object
                from
                  (
                    select
                      __local_2__.*
                    from
                      \"public\".\"post\" as __local_2__
                    where
                      (__local_2__.\"owneruserid\" = __local_0__.\"id\")
                    order by
                      __local_2__.\"id\" ASC
                  ) __local_2__
              ) as __local_1__
          ),
          '[]':: json
        )
    )
  ) as \"@posts\"
from
  (
    select
      __local_0__.*
    from
      \"public\".\"site_user\" as __local_0__
    order by
      __local_0__.\"id\" ASC
  ) __local_0__";
        for _ in 0..1000{
            let before = Instant::now();
            pogg.local_id = 0;
            client.query(&*sql, &[]).unwrap();
            database_side_times.push(before.elapsed().as_micros());
        }
    }

    println!("Database side times {:?}", database_side_times);

    let (sql_query, table_query_infos) = serverside_pogg.build_root(query).unwrap();

    let mut single_threaded_times: Vec<u128> = Vec::new();
    for _ in 0..1000{
        let before = Instant::now();
        serverside_pogg.local_id = 0;
        let rows = client.query(&*sql_query, &[]).unwrap();
        server_side_json_builder::convert(rows, &table_query_infos);
        single_threaded_times.push(before.elapsed().as_micros());
    }
    println!("Single threaded times {:?}", single_threaded_times);

    serverside_pogg.local_id = 0;
    serverside_pogg.num_select_cols = 0;
    server_side_json_builder::run_multithreaded(query, &mut serverside_pogg);
}
