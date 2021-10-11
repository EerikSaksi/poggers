use convert_case::{Case, Casing};
pub mod read_database;
pub mod internal_schema_info;
use std::collections::HashMap;



#[allow(dead_code)]
pub fn create_schema_document() -> String {
    let mut schema_types: HashMap<String, String> = HashMap::new();
    //for current_row in read_database::read_type_information().unwrap().iter() {
    //    let type_name: &str = current_row.get("obj_name");
    //    let data_type: &str = current_row.get("data_type");
    //    let column_name: &str = current_row.get("column_name");
    //    let graphql_type = convert_pg_to_gql(data_type);
    //    add_row(&mut schema_types, column_name, &graphql_type, type_name, "");
    //}

    for current_row in read_database::read_tables().unwrap().iter() {
        //if the table names match then keep building the current graphql type
        let table_name: &str = current_row.get("table_name");
        let column_name: &str = current_row.get("column_name");
        let nullable: &str = current_row.get("nullable");
        let data_type: &str = current_row.get("data_type");
        let graphql_type = convert_pg_to_gql(data_type);
        add_row(
            &mut schema_types,
            column_name,
            &graphql_type,
            table_name,
            nullable,
        );
    }
    let formatted = schema_types
        .values()
        .cloned()
        .fold(String::new(), |a, b| a + &b + "\n");
    println!("{}", formatted);

    //    for def in ast.definitions {
    //        if let Definition::TypeDefinition(TypeDefinition::Object(val)) = def {
    //            if val.name != "Query" {
    //                for field in val.fields {}
    //            }
    //        }
    //    }
    formatted
}
fn convert_pg_to_gql(data_type: &str) -> String {
    match data_type {
        "integer" | "smallint" => String::from("Int"),
        "character varying" => String::from("String"),
        "text" => String::from("String"),
        "timestamp with time zone" | "timestamp" => String::from("Datetime"),
        "double precision" => String::from("Float"),
        other => other.to_case(Case::UpperCamel),
    }
}

fn add_row(
    schema_types: &mut HashMap<String, String>,
    column_name: &str,
    graphql_type: &str,
    table_name: &str,
    nullable: &str,
) {
    if schema_types.contains_key(table_name) {
        schema_types.get_mut(table_name).unwrap().push_str(&format!(
            "\n\t{}: {}{}",
            column_name.to_case(Case::Camel),
            graphql_type,
            nullable
        ));
    } else {
        schema_types.insert(
            table_name.to_string(),
            format!("type {} {{", table_name.to_case(Case::UpperCamel)),
        );
    }
}
