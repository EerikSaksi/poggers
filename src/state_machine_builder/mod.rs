use crate::generate_sql::*;
use core::slice::Iter;
use tokio_postgres::Row;
mod column_converter;

#[cfg(test)]
#[path = "./test.rs"]
mod test;

#[derive(Debug)]
pub enum State {
    Parent,
    ForeignField(usize),
}

pub struct JsonBuilder<'a> {
    pub s: String,
    row_iter: Iter<'a, Row>,
    state: State,
    table_metadata: Vec<TableMetadata>,
    root_key_name: &'a str,
}
impl<'a> JsonBuilder<'a> {
    pub fn new(
        row_iter: Iter<'a, Row>,
        table_metadata: Vec<TableMetadata>,
        root_key_name: &'a str,
    ) -> Self {
        JsonBuilder {
            s: String::new(),
            row_iter,
            state: State::Parent,
            table_metadata,
            root_key_name,
        }
    }

    pub fn exec_until_state_change(&mut self) {
        self.s.push_str(&["{\"", self.root_key_name, "\":["].concat());
        for row in self.row_iter.by_ref() {
            match self.state {
                State::Parent => {
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
                            ColumnInfo::Foreign(field_name) => {
                                self.s.push_str(&[field_name, ":{"].concat());
                                self.state = State::ForeignField(1)
                            }
                            ColumnInfo::ForeignSingular(_) => unimplemented!(),
                        }
                    }
                    self.s.pop();
                    self.s.push_str("},");
                }
                State::ForeignField(_) => unimplemented!(),
            }
        }
        self.s.pop();
        self.s.push_str("]}");
    }
}
