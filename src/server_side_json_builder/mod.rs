#[cfg(test)]
#[path = "./test.rs"]
mod test;

pub use self::generate_sql::ServerSidePoggers;
use crate::server_side_json_builder::generate_sql::JsonBuilderContext;
use chrono::{DateTime, Utc};
use postgres::Row;
use std::ops::Range;
pub mod generate_sql;

#[derive(Debug)]
pub enum ColumnInfo {
    Foreign(String),
    ForeignSingular(String),
    Terminal(String, usize),
}

#[derive(Debug)]
pub struct TableQueryInfo {
    graphql_fields: Vec<ColumnInfo>,
    primary_key_range: Range<usize>,
}

pub struct JsonBuilder {
    closures: Vec<Box<dyn Fn(&Row, usize) -> String>>,
    table_query_infos: Vec<TableQueryInfo>,
    root_key_name: String,
    root_query_is_many: bool,
}
struct MutableState {
    table: usize,
    row: usize,
    s: String,
}

#[allow(dead_code)]
impl JsonBuilder {
    pub fn new(ctx: JsonBuilderContext) -> Self {
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
            root_key_name: ctx.root_key_name,
            table_query_infos: ctx.table_query_infos,
            root_query_is_many: ctx.root_query_is_many,
        }
    }

    pub fn convert(&self, rows: &[Row]) -> String {
        //the first row requires special treatment, we need to add the initial braces, and we need
        //there is no previous pks to determine if this row should be
        let mut s = ["{", &JsonBuilder::stringify(&self.root_key_name), ":"].concat();

        if self.root_query_is_many {
            s.push('[');
        }

        let first_row;
        match rows.get(0) {
            Some(row) => first_row = row,
            None => {
                //either empty list or null
                s.push_str(if self.root_query_is_many { "]" } else { "null" });
                s.push('}');
                return s;
            }
        }
        s.push('{');

        let mut state = MutableState {
            row: 0,
            table: 0,
            s,
        };
        self.build_one_root_parent(rows, &mut state);
        let mut last_pk: i32 = first_row.get(0);
        while let Some(row) = rows.get(state.row) {
            //one left of the start of the next tables cols is primary key
            let pk: i32 = row.get(0);
            if pk != last_pk {
                //parent changed
                state.s.drain(state.s.len() - 1..state.s.len());
                state.s.push_str(&["},{"].concat());
                self.build_one_root_parent(rows, &mut state);
            }
            last_pk = pk;
        }

        //drop trailing comma (not allowed in some JSON parsers)
        state.s.drain(state.s.len() - 1..state.s.len());

        state.s.push('}');
        if self.root_query_is_many {
            state.s.push(']');
        }
        state.s.push('}');
        state.s
    }
    fn build_one_root_parent(&self, rows: &[Row], state: &mut MutableState) {
        let mut col_offset = self.table_query_infos.get(0).unwrap().primary_key_range.end;
        for col_info in &self.table_query_infos.get(0).unwrap().graphql_fields {
            col_offset = match col_info {
                ColumnInfo::ForeignSingular(key) => {
                    self.build_foreign_singular(rows, state, key, col_offset)
                }
                ColumnInfo::Foreign(key) => {
                    let child_pk: Option<i32> = rows.get(state.row).unwrap().get(
                        self.table_query_infos
                            .get(1)
                            .unwrap()
                            .primary_key_range
                            .start,
                    );
                    state
                        .s
                        .push_str(&[&JsonBuilder::stringify(key), ":["].concat());
                    if child_pk.is_some() {
                        let parent_pks_range =
                            &self.table_query_infos.get(0).unwrap().primary_key_range;
                        self.build_children_no_parent_null_check(rows, state, parent_pks_range);
                        state.s.drain(state.s.len() - 1..state.s.len());
                    }
                    state.s.push_str("],");
                    col_offset
                }
                ColumnInfo::Terminal(key, closure_index) => {
                    self.build_terminal(rows, state, key, *closure_index, col_offset)
                }
            };
        }
    }
    fn build_children(
        &self,
        rows: &[Row],
        state: &mut MutableState,
        parent_pks_range: &Range<usize>,
    ) {
        let mut parent_pks: Vec<i32> = vec![];
        {
            let row = rows.get(state.row).unwrap();
            for col_offset in parent_pks_range.start..parent_pks_range.end {
                parent_pks.push(row.get(col_offset));
            }
        }
        let col_offset = self
            .table_query_infos
            .get(state.table)
            .unwrap()
            .primary_key_range
            .end;

        loop {
            self.build_one_child(rows, state, col_offset);
            match rows.get(state.row) {
                Some(next_row) => {
                    //null check the first parent column (rest will not be null if not null)
                    let first_pk: Option<i32> = next_row.get(parent_pks_range.start);
                    match first_pk {
                        Some(first_pk) => {
                            if first_pk != *parent_pks.get(0).unwrap() {
                                return;
                            };
                            let mut i = 1;
                            while i + parent_pks_range.start < parent_pks_range.end {
                                let pk_val: i32 = next_row.get(col_offset);
                                if pk_val != *parent_pks.get(i).unwrap() {
                                    return;
                                };
                                i += 1;
                            }
                        }
                        None => return,
                    }
                }
                None => return,
            }
            state.row += 1;
        }
    }

    fn build_children_no_parent_null_check(
        &self,
        rows: &[Row],
        state: &mut MutableState,
        parent_pks_range: &Range<usize>,
    ) {
        let mut parent_pks: Vec<i32> = vec![];
        let col_offset = self
            .table_query_infos
            .get(state.table)
            .unwrap()
            .primary_key_range
            .end;
        {
            let r = rows.get(state.row).unwrap();
            for col_offset in parent_pks_range.start..parent_pks_range.end {
                parent_pks.push(r.get(col_offset));
            }
        }

        loop {
            self.build_one_child(rows, state, col_offset);
            let mut i = parent_pks_range.start;
            if let Some(next_row) = rows.get(state.row + 1) {
                while i < parent_pks_range.end {
                    let pk_val: i32 = next_row.get(state.row + 1);
                    if pk_val != *parent_pks.get(i).unwrap() {
                        return;
                    };
                    i += 1;
                }
                state.row += 1
            }
        }
    }
    fn stringify(field: &str) -> String {
        ["\"", field, "\""].concat()
    }
    fn build_col_info(
        &self,
        rows: &[Row],
        state: &mut MutableState,
        col_info: &ColumnInfo,
        col_offset: usize,
    ) -> usize {
        match col_info {
            ColumnInfo::ForeignSingular(key) => {
                self.build_foreign_singular(rows, state, key, col_offset)
            }
            ColumnInfo::Foreign(key) => self.build_foreign(rows, state, key, col_offset),
            ColumnInfo::Terminal(key, closure_index) => {
                self.build_terminal(rows, state, key, *closure_index, col_offset)
            }
        }
    }

    fn build_one_child(&self, rows: &[Row], state: &mut MutableState, mut col_offset: usize) {
        state.s.push('{');
        for col_info in &self
            .table_query_infos
            .get(state.table)
            .unwrap()
            .graphql_fields
        {
            col_offset = self.build_col_info(rows, state, col_info, col_offset);
        }
        state.s.drain(state.s.len() - 1..state.s.len());
        state.s.push_str("},");
    }
    fn build_foreign_singular(
        &self,
        rows: &[Row],
        state: &mut MutableState,
        key: &str,
        col_offset: usize,
    ) -> usize {
        let pk_col_offset = self
            .table_query_infos
            .get(state.table + 1)
            .unwrap()
            .primary_key_range
            .start;

        let child_pk: Option<i32> = rows.get(state.table).unwrap().get(pk_col_offset);
        state
            .s
            .push_str(&[&JsonBuilder::stringify(key), ":"].concat());
        if child_pk.is_some() {
            self.build_one_child(rows, state, col_offset);
        } else {
            state.s.push_str("null,")
        }
        col_offset
    }

    fn build_foreign(
        &self,
        rows: &[Row],
        state: &mut MutableState,
        key: &str,
        col_offset: usize,
    ) -> usize {
        let child_pk: Option<i32> = rows.get(state.row).unwrap().get(
            self.table_query_infos
                .get(state.table + 1)
                .unwrap()
                .primary_key_range
                .end,
        );
        state
            .s
            .push_str(&[&JsonBuilder::stringify(key), ":["].concat());
        if child_pk.is_some() {
            let parent_pks_range = &self
                .table_query_infos
                .get(state.table)
                .unwrap()
                .primary_key_range;

            self.build_children(rows, state, parent_pks_range);
            state.s.drain(state.s.len() - 1..state.s.len());
        }
        state.s.push_str("],");
        col_offset
    }

    fn build_terminal(
        &self,
        rows: &[Row],
        state: &mut MutableState,
        key: &str,
        closure_index: usize,
        col_offset: usize,
    ) -> usize {
        let col_val = self.closures[closure_index](rows.get(state.row).unwrap(), col_offset);
        state
            .s
            .push_str(&[&JsonBuilder::stringify(key), ":", &col_val, ","].concat());
        col_offset + 1
    }
}
