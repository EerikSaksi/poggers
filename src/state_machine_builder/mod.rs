use crate::generate_sql::*;
use core::slice::Iter;
use tokio_postgres::Row;
use std::io::Write;
mod column_converter;

#[cfg(test)]
#[path = "./test.rs"]
mod test;

#[derive(Debug)]
pub enum State {
    Init,
    Parent,
    Done,
}

pub struct JsonBuilder<'a> {
    pub s: String,
    row_iter: Iter<'a, Row>,
    state: State,
    table_metadata: Vec<TableMetadata>,
    root_key_name: &'a str
}
impl<'a> JsonBuilder<'a> {
    pub fn new(row_iter: Iter<'a, Row>, table_metadata: Vec<TableMetadata>, root_key_name: &'a str) -> Self {
        JsonBuilder {
            s: String::new(),
            row_iter,
            state: State::Init,
            table_metadata,
            root_key_name
        }
    }
    pub fn exec_until_state_change(&mut self) {
        match &self.state {
            State::Parent => {
                for row in self.row_iter.by_ref() {
                    self.s.push('{');
                    for (index, col) in self
                        .table_metadata
                        .get(0)
                        .unwrap()
                        .graphql_fields
                        .iter()
                        .enumerate()
                    {
                        match col {
                            ColumnInfo::Terminal(col_name, pg_type) => self.s.push_str(
                                &[
                                    "\"",
                                    col_name,
                                    "\":",
                                    &pg_type.stringify_column(row, index + 1),
                                    ",",
                                ]
                                .concat(),
                            ),
                            _ => unimplemented!(),
                        }
                    }
                    self.s.pop();
                    self.s.push_str("},");
                }
                self.state = State::Done;
            },
            State::Init => {
                write!(self.s, "[{}:\{", self.root_key_name);
                self.state = State::Parent;
            },
            State::Done => ()
        }
    }
}
