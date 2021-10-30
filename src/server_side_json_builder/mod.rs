#[cfg(test)]
#[path = "./test.rs"]
mod test;
use postgres::Row;

pub use self::generate_sql::ServerSidePoggers;
use chrono::{DateTime, Utc};
pub mod generate_sql;

pub enum ColumnInfo {
    Foreign(String, usize),
    Terminal(String, usize),
}

pub struct TableQueryInfo {
    graphql_fields: Vec<ColumnInfo>,
    parent_key_name: String,
    column_offset: usize,
}

pub struct JsonBuilder {
    closures: Vec<Box<dyn Fn(&Row, usize) -> String>>,
}

#[allow(dead_code)]
impl JsonBuilder {
    pub fn new() -> Self {
        JsonBuilder {
            closures: vec![
                Box::new(|row, index| {
                    let col_val: i32 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: &str = row.get(index);
                    ["\"", col_val, "\""].concat()
                }),
                Box::new(|row, index| {
                    let col_val: f64 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: DateTime<Utc> = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: bool = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: serde_json::Value = row.get(index);
                    col_val.to_string()
                }),
            ],
        }
    }

    pub fn convert(&self, rows: Vec<Row>, table_query_infos: &Vec<TableQueryInfo>) -> String {
        let mut s = [
            "{",
            &JsonBuilder::stringify(&table_query_infos.get(0).unwrap().parent_key_name),
            ":[",
        ]
        .concat();

        let mut row_iter = rows.iter();
        let first_row = row_iter.next().unwrap();
        s.push('{');
        self.build_field(&mut s, &table_query_infos, 0, &first_row);

        let mut last_pk: i32 = first_row.get(0);
        while let Some(row) = row_iter.next() {
            //one left of the start of the next tables cols is primary key
            let pk: i32 = row.get(0);
            if pk != last_pk {
                //parent changed
                s.drain(s.len() - 1..s.len());
                s.push_str(&["},{"].concat());
                self.build_field(&mut s, &table_query_infos, 0, &row)
            }
            last_pk = pk;
        }

        //drop trailing comma (not allowed in some JSON parsers)
        s.drain(s.len() - 1..s.len());

        s.push_str("}]}");
        s
    }
    fn build_field(
        &self,
        s: &mut String,
        table_query_infos: &Vec<TableQueryInfo>,
        table_index: usize,
        row: &Row,
    ) {
        let col_offset = table_query_infos.get(0).unwrap().column_offset;
        for (i, col_info) in table_query_infos
            .get(0)
            .unwrap()
            .graphql_fields
            .iter()
            .enumerate()
        {
            match col_info {
                ColumnInfo::Foreign(key, child_index) => {
                    s.push_str(&[&JsonBuilder::stringify(key), ": ["].concat());
                    self.build_field(s, table_query_infos, *child_index, row);
                    s.push(']');
                }
                ColumnInfo::Terminal(key, closure_index) => {
                    let col_val = self.closures[*closure_index](row, col_offset + i + 1);
                    s.push_str(
                        &[&JsonBuilder::stringify(key), ":", &col_val.to_string(), ","].concat(),
                    );
                }
            };
        }

        if let Some(TableQueryInfo {
            graphql_fields: _,
            parent_key_name,
            column_offset: _,
        }) = table_query_infos.get(table_index + 1)
        {
            s.push_str(&[&JsonBuilder::stringify(&parent_key_name), ":["].concat())
        }
    }

    fn stringify(field: &str) -> String {
        ["\"", field, "\""].concat()
    }
}

//pub fn run_multithreaded(gql_query: &str, pogg: &mut ServerSidePoggers) {
//    let mut handles = vec![];
//
//    let times: Vec<u128> = vec![];
//    let (query, table_query_infos) = pogg.build_root(gql_query).unwrap();
//    let mut thread_infos = (0..8).map(|_| (query.to_string(), table_query_infos.to_vec()));
//
//    let client = Arc::new(Mutex::new((Client::connect(
//        "postgres://eerik:Postgrizzly@localhost:5432/pets",
//        NoTls,
//    )
//    .unwrap(),)));
//
//    let runtime_infos = Arc::new(Mutex::new((times, Instant::now())));
//    for _ in 0..8 {
//        let client = Arc::clone(&client);
//        let runtime_infos = Arc::clone(&runtime_infos);
//        let threads_metadata = thread_infos.next().unwrap();
//        let query = threads_metadata.0;
//        let table_query_infos = threads_metadata.1;
//        let handle = thread::spawn(move || loop {
//            let rows: Vec<Row>;
//            {
//                let mut locked_client = client.lock().unwrap();
//                rows = locked_client.0.query(&*query, &[]).unwrap();
//            }
//            print!("{}", convert(rows, &table_query_infos));
//            let mut locked_runtime_infos = runtime_infos.lock().unwrap();
//            if 1000 <= locked_runtime_infos.0.len() {
//                return;
//            }
//            let elapsed = locked_runtime_infos.1.elapsed().as_micros();
//            locked_runtime_infos.0.push(elapsed);
//            locked_runtime_infos.1 = Instant::now();
//        });
//        handles.push(handle);
//    }
//
//    for handle in handles {
//        handle.join().unwrap();
//    }
//    println!("Multithreaded times {:?}", runtime_infos.lock().unwrap().0);
//}
