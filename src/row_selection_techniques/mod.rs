use chrono::{DateTime, Utc};
use postgres::types::Timestamp;
use postgres::{Client, NoTls, Row};
extern crate test;
pub fn insert_rows() {
    let mut sql = String::from(
        "
        create table multi_types(
            i32 integer, 
            f64 float,
            str varchar,
            text text,
            time timestamp with time zone,
            object json,
            bool boolean
        )\n;",
    );
    for i in 1..100000 {
        sql.push_str(&format!("insert into multi_types (i32, f64, str, text, time, object, bool) values ({}, {}.5, {}, {}, {}, {}, {});\n", i, i, "'Hello world'", "'Longer hello world column. Text columns can hold much longer data'", "'2021-10-27 09:06:16.187459+01'", "'{ \"customer\": \"Lily Bush\", \"items\": {\"product\": \"Diaper\",\"qty\": 24}}'", "true"));
    }

    let mut client = Client::connect(
        "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
        NoTls,
    )
    .unwrap();
    //https://stackoverflow.com/questions/1152260/how-to-list-table-foreign-keys
    client.batch_execute(&*sql).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    fn get_rows(all_rows: bool) -> Vec<Row> {
        let mut client = Client::connect(
            "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
            NoTls,
        )
        .unwrap();
        client
            .query(
                &*["select * from multi_types", {
                    if all_rows {
                        " "
                    } else {
                        " limit 100"
                    }
                }]
                .concat(),
                &[],
            )
            .unwrap()
    }

    fn get_rows_integer_table() -> Vec<Row> {
        let mut client = Client::connect(
            "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
            NoTls,
        )
        .unwrap();
        client.query("select * from test_table", &[]).unwrap()
    }

    #[bench]
    fn text_query(b: &mut Bencher) {
        let mut client = Client::connect(
            "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
            NoTls,
        )
        .unwrap();
        b.iter(|| client.query("select i32::text, f64::text, str::text, text::text, time::text, object::text, bool::text from multi_types;", &[]).unwrap());
    }
    #[bench]
    fn regular_query(b: &mut Bencher) {
        let mut client = Client::connect(
            "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
            NoTls,
        )
        .unwrap();
        b.iter(|| client.query("select * from multi_types", &[]).unwrap());
    }

    #[bench]
    fn hardcoded_types(b: &mut Bencher) {
        let rows = get_rows(true);
        b.iter(|| {
            let mut s = test::black_box(String::new());
            for row in &rows {
                let int: i32 = row.get(0);
                s.push_str(&int.to_string());

                let float: f64 = test::black_box(row.get(1));
                s.push_str(&float.to_string());

                let varchar: &str = test::black_box(row.get(2));
                s.push_str(&["\"", &varchar.to_string(), "\""].concat());

                let text: &str = test::black_box(row.get(3));
                s.push_str(&["\"", &text.to_string(), "\""].concat());

                let time: DateTime<Utc> = test::black_box(row.get(4));
                s.push_str(&time.to_string());

                let object: serde_json::Value = test::black_box(row.get(5));
                s.push_str(&object.to_string());

                let boolean: bool = test::black_box(row.get(6));
                s.push_str(&boolean.to_string());
            }
        })
    }
    #[bench]
    fn using_closures(b: &mut Bencher) {
        let rows = get_rows(true);
        let mut s = test::black_box(String::new());
        b.iter(|| {
            let closures: Vec<Box<dyn Fn(&Row, usize) -> String>> = vec![
                Box::new(|row, index| {
                    let col_val: i32 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: f64 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: &str = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: &str = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: DateTime<Utc> = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: serde_json::Value = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: bool = row.get(index);
                    col_val.to_string()
                }),
            ];
            for row in &rows {
                for i in 0..6 {
                    s.push_str(&closures[i](row, i))
                }
            }
        })
    }
    #[bench]
    fn test_table_using_generated_closures(b: &mut Bencher) {
        let rows = get_rows_integer_table();
        let mut s = test::black_box(String::new());

        //generate closures
        b.iter(|| {
            let mut closures: Vec<Box<dyn Fn(&Row, usize) -> String>> = test::black_box(Vec::new());

            //generate 100 closures
            for _ in 0..100 {
                closures.push(test::black_box(Box::new(|row, index| {
                    let col_val: i32 = row.get(index);
                    col_val.to_string()
                })));
            }
            for row in &rows {
                for col_index in 0..100 {
                    s.push_str(&closures[col_index](&row, col_index));
                }
            }
        })
    }

    #[bench]
    fn test_table_using_closure_references(b: &mut Bencher) {
        let rows = get_rows_integer_table();
        let mut s = test::black_box(String::new());

        //generate closures
        b.iter(|| {
            let closures: Vec<Box<dyn Fn(&Row, usize) -> String>> =
                test::black_box(vec![Box::new(|row, index| {
                    let col_val: i32 = row.get(index);
                    col_val.to_string()
                })]);

            //all cols refer to the only closure
            let col_to_closure: Vec<usize> = test::black_box((0..100).map(|_| 0).collect());

            for row in &rows {
                for col_index in 0..100 {
                    s.push_str(&closures[col_to_closure[col_index]](row, col_index));
                }
            }
        })
    }
    #[bench]
    fn database_to_string(b: &mut Bencher) {
        let mut client = Client::connect(
            "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
            NoTls,
        )
        .unwrap();
        let is_json_strings = vec![false, false, true, true, true, true, false];
        let rows = client.query("select i32::text, f64::text, str::text, text::text, time::text, object::text, bool::text from multi_types;", &[]).unwrap();

        let mut s = test::black_box(String::new());
        //generate closures
        b.iter(|| {
            for row in &rows {
                for col_index in 0..6 {
                    let col_val: &str = row.get(col_index);
                    if is_json_strings[col_index] {
                        s.push_str(&["\"", col_val, "\""].concat());
                    } else {
                        s.push_str(col_val);
                    }
                }
            }
        })
    }

    #[bench]
    fn hardcoded_types_limit_100(b: &mut Bencher) {
        let rows = get_rows(false);
        b.iter(|| {
            let mut s = test::black_box(String::new());
            for row in &rows {
                let int: i32 = row.get(0);
                s.push_str(&int.to_string());

                let float: f64 = test::black_box(row.get(1));
                s.push_str(&float.to_string());

                let varchar: &str = test::black_box(row.get(2));
                s.push_str(&["\"", &varchar.to_string(), "\""].concat());

                let text: &str = test::black_box(row.get(3));
                s.push_str(&["\"", &text.to_string(), "\""].concat());

                let time: DateTime<Utc> = test::black_box(row.get(4));
                s.push_str(&time.to_string());

                let object: serde_json::Value = test::black_box(row.get(5));
                s.push_str(&object.to_string());

                let boolean: bool = test::black_box(row.get(6));
                s.push_str(&boolean.to_string());
            }
        })
    }

    #[bench]
    fn using_closures_limit_100(b: &mut Bencher) {
        let rows = get_rows(false);
        let mut s = test::black_box(String::new());
        b.iter(|| {
            let closures: Vec<Box<dyn Fn(&Row, usize) -> String>> = vec![
                Box::new(|row, index| {
                    let col_val: i32 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: f64 = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: &str = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: &str = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: DateTime<Utc> = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: serde_json::Value = row.get(index);
                    col_val.to_string()
                }),
                Box::new(|row, index| {
                    let col_val: bool = row.get(index);
                    col_val.to_string()
                }),
            ];
            for row in &rows {
                for i in 0..6 {
                    s.push_str(&closures[i](row, i))
                }
            }
        })
    }
}
