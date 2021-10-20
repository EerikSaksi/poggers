use postgres::{Client, NoTls};
pub struct TableQueryInfo {
    num_selected_cols: u16,
    num_primary_keys: u16,
}


//pub fn convert(query: &str, table_query_info: Vec<TableQueryInfo>) {
//    let mut client =
//        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls).unwrap();
//
//    let rows = client.query(query, &[]).unwrap();
//    let mut i = 0;
//    let mut to_return = String::new();
//    for row in rows {
//
//    }
//}
