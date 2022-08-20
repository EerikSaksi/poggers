use crate::server_side_json_builder::TableQueryInfo;
use chrono::{DateTime, Utc};
use core::slice::Iter;
use tokio_postgres::Row;

pub trait State {}

pub struct JsonBuilder<'a, S: State> {
    json: String,
    row_iter: Iter<'a, Row>,
    state: S,
    closures: Vec<dyn Fn(&Row, usize) -> String>>,
    table_query_infos: Vec<TableQueryInfo>,
}
impl<'a> JsonBuilder<'a, ParentState> {
    pub fn new(row_iter: Iter<'a, Row>, table_query_infos: Vec<TableQueryInfo>) -> Self {
        JsonBuilder {
            row_iter,
            state: ParentState {},
            json: "{[".to_owned(),
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
        }
    }
    pub fn process(&mut self) {
        while let Some(row) = self.row_iter.next() {

        }
    }
}

pub struct ParentState {}
impl State for ParentState {}
