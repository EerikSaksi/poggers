#[cfg(test)]
#[path = "./test.rs"]
mod test;

use postgres::Row;

pub use self::generate_sql::ServerSidePoggers;
use chrono::{DateTime, Utc};
pub mod generate_sql;

pub enum ColumnInfo {
    Foreign(String),
    ForeignSingular(String),
    Terminal(String, usize),
}

pub struct TableQueryInfo {
    graphql_fields: Vec<ColumnInfo>,
    column_offset: usize,
}

pub struct JsonBuilder {
    closures: Vec<Box<dyn Fn(&Row, usize) -> String>>,
    table_query_infos: Vec<TableQueryInfo>,
    root_key_name: String,
}

#[allow(dead_code)]
impl JsonBuilder {
    pub fn new(table_query_infos: Vec<TableQueryInfo>, root_key_name: String) -> Self {
        JsonBuilder {
            closures: vec![
                Box::new(|row, index| {
                    let col_val: i32 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: &str = row.get(index);
                    serde_json::to_string(col_val).unwrap()
                }),
                Box::new(|row, index| {
                    let col_val: f64 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: chrono::NaiveDateTime = row.get(index);
                    ["\"", &col_val.to_string(), "\""].concat()
                }),
                Box::new(|row, index| {
                    let col_val: DateTime<Utc> = row.get(index);
                    ["\"", &col_val.to_string(), "\""].concat()
                }),
                Box::new(|row, index| {
                    let col_val: bool = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: serde_json::Value = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: Option<i32> = row.get(index);
                    match col_val {
                        Some(val) => val.to_string(),
                        None => String::from("null"),
                    }
                }),
                Box::new(|row, index| {
                    let col_val: Option<&str> = row.get(index);
                    match col_val {
                        Some(val) => serde_json::to_string(val).unwrap(),
                        None => String::from("null"),
                    }
                }),
                Box::new(|row, index| {
                    let col_val: Option<f64> = row.get(index);
                    match col_val {
                        Some(val) => val.to_string(),
                        None => String::from("null"),
                    }
                }),
                Box::new(|row, index| {
                    let col_val: Option<chrono::NaiveDateTime> = row.get(index);
                    match col_val {
                        Some(val) => ["\"", &val.to_string(), "\""].concat(),
                        None => String::from("null"),
                    }
                }),
                Box::new(|row, index| {
                    let col_val: Option<DateTime<Utc>> = row.get(index);
                    match col_val {
                        Some(val) => ["\"", &val.to_string(), "\""].concat(),
                        None => String::from("null"),
                    }
                }),
                Box::new(|row, index| {
                    let col_val: Option<bool> = row.get(index);
                    match col_val {
                        Some(val) => val.to_string(),
                        None => String::from("null"),
                    }
                }),
                Box::new(|row, index| {
                    let col_val: Option<serde_json::Value> = row.get(index);
                    match col_val {
                        Some(val) => val.to_string(),
                        None => String::from("null"),
                    }
                }),
            ],
            table_query_infos,
            root_key_name,
        }
    }

    pub fn convert(&self, rows: Vec<Row>) -> String {
        //the first row requires special treatment, we need to add the initial braces, and we need
        //there is no previous pks to determine if this row should be
        let mut s = ["{", &JsonBuilder::stringify(&self.root_key_name), ":["].concat();

        let mut row_iter = rows.iter().peekable();
        let first_row = row_iter.next().unwrap();
        s.push('{');
        self.build_one_root_parent(&mut s, &first_row, &mut row_iter);

        let mut last_pk: i32 = first_row.get(0);
        while let Some(row) = row_iter.next() {
            //one left of the start of the next tables cols is primary key
            let pk: i32 = row.get(0);
            if pk != last_pk {
                //parent changed
                s.drain(s.len() - 1..s.len());
                s.push_str(&["},{"].concat());
                self.build_one_root_parent(&mut s, &row, &mut row_iter)
            }
            last_pk = pk;
        }

        //drop trailing comma (not allowed in some JSON parsers)
        s.drain(s.len() - 1..s.len());
        s.push_str("}]}");
        s
    }
    fn build_one_root_parent<'a, I>(
        &self,
        s: &mut String,
        row: &'a postgres::Row,
        row_iter: &mut std::iter::Peekable<I>,
    ) where
        I: std::iter::Iterator<Item = &'a Row>,
    {
        let mut col_offset = self.table_query_infos.get(0).unwrap().column_offset + 1;
        for col_info in &self.table_query_infos.get(0).unwrap().graphql_fields {
            col_offset = self.build_col_info(s, col_info, row, 0, row_iter, col_offset);
        }
    }
    fn build_child<'a, I>(
        &self,
        s: &mut String,
        parent_pk_index: usize,
        mut row: &'a Row,
        row_iter: &mut std::iter::Peekable<I>,
        table_index: usize,
    ) where
        I: std::iter::Iterator<Item = &'a Row>,
    {
        let mut parent_pk: i32 = row.get(parent_pk_index);
        let col_offset_start = self
            .table_query_infos
            .get(table_index)
            .unwrap()
            .column_offset
            + 1;

        loop {
            let mut col_offset = col_offset_start;
            s.push('{');
            for col_info in &self
                .table_query_infos
                .get(table_index)
                .unwrap()
                .graphql_fields
            {
                col_offset =
                    self.build_col_info(s, col_info, row, table_index, row_iter, col_offset);
            }
            s.drain(s.len() - 1..s.len());
            s.push_str("},");
            match row_iter.peek() {
                Some(next_row) => {
                    let next_pk_opt: Option<i32> = next_row.get(parent_pk_index);
                    match next_pk_opt {
                        Some(next_pk) => {
                            if next_pk != parent_pk {
                                break;
                            };
                            parent_pk = next_pk
                        }
                        None => break,
                    }
                }
                None => break,
            }

            //can unwrap as this does not run if peek fails
            row = row_iter.next().unwrap();
        }
    }
    fn stringify(field: &str) -> String {
        ["\"", field, "\""].concat()
    }
    fn build_col_info<'a, I>(
        &self,
        s: &mut String,
        col_info: &ColumnInfo,
        row: &'a postgres::Row,
        table_index: usize,
        row_iter: &mut std::iter::Peekable<I>,
        col_offset: usize,
    ) -> usize
    where
        I: std::iter::Iterator<Item = &'a Row>,
    {
        match col_info {
            ColumnInfo::ForeignSingular(key) => {
                let child_pk: Option<i32> = row.get(
                    self.table_query_infos
                        .get(table_index + 1)
                        .unwrap()
                        .column_offset,
                );

                s.push_str(&[&JsonBuilder::stringify(&key), ":["].concat());
                if child_pk.is_some() {
                    self.build_child(s, col_offset, row, row_iter, table_index + 1);
                    s.drain(s.len() - 1..s.len());
                }
                s.push_str("]}");
                col_offset
            }
            ColumnInfo::Foreign(key) => {
                let child_pk: Option<i32> = row.get(
                    self.table_query_infos
                        .get(table_index + 1)
                        .unwrap()
                        .column_offset,
                );

                s.push_str(&[&JsonBuilder::stringify(&key), ":["].concat());
                if child_pk.is_some() {
                    let parent_pk_index = self
                        .table_query_infos
                        .get(table_index)
                        .unwrap()
                        .column_offset;
                    self.build_child(s, parent_pk_index, row, row_iter, table_index + 1);
                    s.drain(s.len() - 1..s.len());
                }
                s.push_str("],");
                col_offset
            }
            ColumnInfo::Terminal(key, closure_index) => {
                let col_val = self.closures[*closure_index](&row, col_offset);
                s.push_str(
                    &[
                        &JsonBuilder::stringify(&key),
                        ":",
                        &col_val.to_string(),
                        ",",
                    ]
                    .concat(),
                );
                col_offset + 1
            }
        }
    }
}
