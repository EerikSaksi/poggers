use postgres::{Client, NoTls};
mod generate_sql;

pub struct TableQueryInfo {
    graphql_fields: Vec<String>,
    parent_key_name: String,
}

pub fn convert(query: &str, table_query_info: Vec<TableQueryInfo>) {
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/pets", NoTls).unwrap();

    let rows = client.query(query, &[]).unwrap();

    let mut s = String::new();
    let mut rows_iter = rows.iter().peekable();
    while let Some(row) = rows_iter.next() {
        let pk_col_name = "__t0_pk0__";
        let pk: i32 = row.get(&*pk_col_name);
        if let Some(next_row) = rows_iter.peek() {
            let next_pk: i32 = next_row.get(&*pk_col_name);
            //parent changed
            if pk != next_pk {
                s.push_str(&["\n]\n},\n{,\n"].concat());
                let mut i = 0;
                for gql_field in &table_query_info.get(0).unwrap().graphql_fields {
                    let col_name = ["__t0_c", &i.to_string(), "__"].concat();
                    let col_val: i32 = row.get(&*col_name);
                    s.push_str(&[&gql_field, ":", &col_val.to_string(), "\n[\n"].concat());
                    i += 1;
                }
            }
        }
        let mut i = 0;
        for gql_field in &table_query_info.get(1).unwrap().graphql_fields {
            let col_name = ["__t1_c", &i.to_string(), "__"].concat();
            let col_val: i32 = row.get(&*col_name);
            s.push_str(&[&gql_field, ":", &col_val.to_string(), "\n[\n"].concat());
            i += 1
        }
    }
}
