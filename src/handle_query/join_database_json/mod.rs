//use postgres::Row;
//pub fn main(query_res: Vec<Row>) -> String {
//    let mut s = String::new();
//    if let Some(row) = query_res.get(1) {
//        s.push_str("{{");
//        for col in row.columns() {
//            let val: serde_json::Value = row.get(col.name());
//            s.push_str(&format!("\t{}: {},", col.name(), val.to_string()));
//        }
//        s.push_str("\n}}");
//    }
//    s
//}
