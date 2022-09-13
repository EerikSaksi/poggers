use crate::generate_sql::*;
use chrono::{DateTime, Utc};
use core::slice::Iter;
use tokio_postgres::Row;

pub trait State {}

pub struct JsonBuilder<'a, S: State> {
    json: String,
    row_iter: Iter<'a, Row>,
    state: S,
    table_query_infos: Vec<TableQueryInfo>,
}
impl<'a> JsonBuilder<'a, ParentState> {
    pub fn new(row_iter: Iter<'a, Row>, table_query_infos: Vec<TableQueryInfo>) -> Self {
        JsonBuilder {
            row_iter,
            state: ParentState {},
            json: "{[".to_owned(),
            table_query_infos,
        }
    }
    pub fn process(&mut self) {
        while let Some(row) = self.row_iter.next() {}
    }
}
    
pub struct ParentState {}
impl State for ParentState {}
