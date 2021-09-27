use crate::build_schema::{convert_pg_to_gql, read_database};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

enum GraphQLSelection<'a> {
    Field {
        field_name: &'a str,
    },
    Composite {
        foreign_table_name: &'a str,
        foreign_selection: Rc<Vec<&'a GraphQLSelection<'a>>>,
    },
}

fn create() {
    let mut table_to_selection: HashMap<&str, Rc<Vec<&GraphQLSelection>>> = HashMap::new();
    for current_row in read_database::read_tables().unwrap().iter() {
        let table_name: &str = current_row.get("table_name");
        let column_name: &str = current_row.get("column_name");
        //let nullable: &str = current_row.get("nullable");
        let data_type: &str = current_row.get("data_type");
        let foreign_table_name: Option<&str> = current_row.get("foreign_table_name");
        let graphql_type = convert_pg_to_gql(data_type);

        if !table_to_selection.contains_key(table_name) {
            table_to_selection.insert(table_name, Rc::new(Vec::new()));
        }
        let selections = table_to_selection.get_mut(table_name).unwrap();
        match foreign_table_name {
            Some(foreign_table_name) => {
                table_to_selection
                    .entry(foreign_table_name)
                    .or_insert_with(|| Rc::new(Vec::new()));

                let foreign_selection = table_to_selection.get(foreign_table_name).unwrap();

                let a = GraphQLSelection::Composite {
                    foreign_table_name,
                    foreign_selection: Rc::clone(foreign_selection),
                };
                selections.borrow_mut()
            }
            None => println!("nice"),
        }
    }
}
