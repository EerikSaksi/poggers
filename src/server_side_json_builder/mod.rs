#[cfg(test)]
#[path = "./test.rs"]
mod test;
use postgres::{Client, NoTls};

pub use self::generate_sql::ServerSidePoggers;
pub mod generate_sql;

#[derive(PartialEq, Debug)]
pub struct TableQueryInfo {
    graphql_fields: Vec<String>,
    parent_key_name: String,
}

pub fn convert(gql_query: &str, pogg: &mut ServerSidePoggers) -> String {
    let client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();
    let (sql_query, table_query_infos) = pogg.build_root(gql_query).unwrap();


    //let rows = client.query(&*gql_query, &[]).unwrap();

    //let mut s = String::new();
    //let mut rows_iter = rows.iter().peekable();
    //while let Some(row) = rows_iter.next() {
    //    let pk_col_name = "__t0_pk0__";
    //    let pk: i32 = row.get(&*pk_col_name);
    //    if let Some(next_row) = rows_iter.peek() {
    //        let next_pk: i32 = next_row.get(&*pk_col_name);
    //        //parent changed
    //        if pk != next_pk {
    //            s.push_str(&["\n]\n},\n{,\n"].concat());
    //            let mut i = 0;
    //            for gql_field in &table_query_infos.get(0).unwrap().graphql_fields {
    //                let col_name = ["__t0_c", &i.to_string(), "__"].concat();
    //                let col_val: i32 = row.get(&*col_name);
    //                s.push_str(&[&gql_field, ":", &col_val.to_string(), "\n[\n"].concat());
    //                i += 1;
    //            }
    //        }
    //    }
    //    let mut i = 0;
    //    for gql_field in &table_query_infos.get(1).unwrap().graphql_fields {
    //        let col_name = ["__t1_c", &i.to_string(), "__"].concat();
    //        let col_val: i32 = row.get(&*col_name);
    //        s.push_str(&[&gql_field, ":", &col_val.to_string(), "\n[\n"].concat());
    //        i += 1
    //    }
    //}
    "hello ".to_string()
}
