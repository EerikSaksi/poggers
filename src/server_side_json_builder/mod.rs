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
    table_query_infos: Vec<TableQueryInfo>,
}

#[allow(dead_code)]
impl JsonBuilder {
    pub fn new(table_query_infos: Vec<TableQueryInfo>) -> Self {
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
            table_query_infos,
        }
    }

    pub fn convert(&self, rows: Vec<Row>) -> String {
        let mut s = [
            "{",
            &JsonBuilder::stringify(&self.table_query_infos.get(0).unwrap().parent_key_name),
            ":[",
        ]
        .concat();

        let mut row_iter = rows.iter().peekable();
        let first_row = row_iter.next().unwrap();
        s.push('{');
        self.build_field(&mut s, 0, &first_row);

        let mut last_pk: i32 = first_row.get(0);
        let col_offset = self.table_query_infos.get(0).unwrap().column_offset;
        while let Some(row) = row_iter.next() {
            for (i, col_info) in self
                .table_query_infos
                .get(0)
                .unwrap()
                .graphql_fields
                .iter()
                .enumerate()
            {
                match col_info {
                    ColumnInfo::Foreign(key, child_index) => {
                        s.push_str(&[&JsonBuilder::stringify(key), ":["].concat());
                        self.build_child(s, 0, row, row_iter);
                        s.push(']');
                    }
                    ColumnInfo::Terminal(key, closure_index) => {
                        let col_val = self.closures[*closure_index](row, col_offset + i + 1);
                        s.push_str(
                            &[&JsonBuilder::stringify(key), ":", &col_val.to_string(), ","]
                                .concat(),
                        );
                    }
                };
            }

            if let Some(TableQueryInfo {
                graphql_fields: _,
                parent_key_name,
                column_offset: _,
            }) = self.table_query_infos.get(1)
            {
                s.push_str(&[&JsonBuilder::stringify(&parent_key_name), ":["].concat())
            }
            //one left of the start of the next tables cols is primary key
            let pk: i32 = row.get(0);
            if pk != last_pk {
                //parent changed
                s.drain(s.len() - 1..s.len());
                s.push_str(&["},{"].concat());
                self.build_field(&mut s, 0, &row)
            }
            last_pk = pk;
        }

        //drop trailing comma (not allowed in some JSON parsers)
        s.drain(s.len() - 1..s.len());

        s.push_str("}]}");
        s
    }
    fn build_field(&self, s: &mut String, table_index: usize, row: &Row) {}
    fn build_child<I>(
        &self,
        s: &mut String,
        parent_pk_index: usize,
        row: Row,
        row_iter: &mut std::iter::Peekable<I>,
    ) where
        I: std::iter::Iterator<Item = Row>,
    {
        let parent_pk = row.get(parent_pk_index);
        let col_offset = self.table_query_infos.get(0).unwrap().column_offset;
        for (i, col_info) in self
            .table_query_infos
            .get(0)
            .unwrap()
            .graphql_fields
            .iter()
            .enumerate()
        {
            match col_info {
                ColumnInfo::Foreign(key, child_index) => {}
                ColumnInfo::Terminal(key, closure_index) => {
                    let col_val = self.closures[*closure_index](row, col_offset + i + 1);
                    s.push_str(
                        &[&JsonBuilder::stringify(key), ":", &col_val.to_string(), ","].concat(),
                    );
                }
            };
        }
    }

    fn stringify(field: &str) -> String {
        ["\"", field, "\""].concat()
    }
}
