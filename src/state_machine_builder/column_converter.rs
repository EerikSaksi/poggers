use crate::build_schema::PostgresType;
use chrono::{DateTime, Utc};
use postgres::Row;

impl PostgresType {
    fn stringify_column(&self, row: Row, index: usize) -> String {
        match self {
            PostgresType::Int => {
                let col_val: i32 = row.get(index);
                col_val.to_string()
            }
            PostgresType::Str => {
                let col_val: &str = row.get(index);
                serde_json::to_string(col_val).unwrap()
            }
            PostgresType::Float => {
                let col_val: f64 = row.get(index);
                col_val.to_string()
            }
            PostgresType::Timestamp => {
                let col_val: chrono::NaiveDateTime = row.get(index);
                ["\"", &col_val.to_string(), "\""].concat()
            }
            PostgresType::Timestamptz => {
                let col_val: DateTime<Utc> = row.get(index);
                ["\"", &col_val.to_string(), "\""].concat()
            }
            PostgresType::Boolean => {
                let col_val: bool = row.get(index);
                col_val.to_string()
            }
            PostgresType::Json => {
                let col_val: serde_json::Value = row.get(index);
                col_val.to_string()
            }
            PostgresType::NullableInt => {
                let col_val: Option<i32> = row.get(index);
                match col_val {
                    Some(val) => val.to_string(),
                    None => String::from("null"),
                }
            }
            PostgresType::NullableStr => {
                let col_val: Option<&str> = row.get(index);
                match col_val {
                    Some(val) => serde_json::to_string(val).unwrap(),
                    None => String::from("null"),
                }
            }
            PostgresType::NullableFloat => {
                let col_val: Option<f64> = row.get(index);
                match col_val {
                    Some(val) => val.to_string(),
                    None => String::from("null"),
                }
            }
            PostgresType::NullableTimestamp => {
                let col_val: Option<chrono::NaiveDateTime> = row.get(index);
                match col_val {
                    Some(val) => ["\"", &val.to_string(), "\""].concat(),
                    None => String::from("null"),
                }
            }
            PostgresType::NullableTimestamptz => {
                let col_val: Option<DateTime<Utc>> = row.get(index);
                match col_val {
                    Some(val) => ["\"", &val.to_string(), "\""].concat(),
                    None => String::from("null"),
                }
            }
            PostgresType::NullableBoolean => {
                let col_val: Option<bool> = row.get(index);
                match col_val {
                    Some(val) => val.to_string(),
                    None => String::from("null"),
                }
            }
            PostgresType::NullableJson => {
                let col_val: Option<serde_json::Value> = row.get(index);
                match col_val {
                    Some(val) => val.to_string(),
                    None => String::from("null"),
                }
            }
        }
    }
}
