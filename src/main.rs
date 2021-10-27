#![feature(test)]
use build_schema::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};
use std::sync::{Arc, Mutex};

mod build_schema;
mod handle_query;
mod integer_indexing;
mod juniper_implementation;
mod row_selection_techniques;
use postgres::{Client, NoTls, Row};
use std::{thread, time::Duration};
fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    let client = Arc::new(Mutex::new(
        Client::connect(
            "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
            NoTls,
        )
        .unwrap(),
    ));

    for i in 0..4 {
        let client = Arc::clone(&client);
        let handle = thread::spawn(move || {
            {
                let mut locked_client = client.lock().unwrap();
                println!("Thread {} locked client", i);
                locked_client
                    .query("select * from multi_types", &[])
                    .unwrap();
                println!("Thread {} received rows", i);
            }
            thread::sleep(Duration::from_millis(500));
            println!("Thread {} Done building JSON", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}
