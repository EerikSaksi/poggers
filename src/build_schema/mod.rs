use convert_case::{Case, Casing};
use inflector::Inflector;
use std::rc::Rc;
mod read_database;
use graphql_parser::parse_schema;
use graphql_parser::schema::{Definition, ObjectType, TypeDefinition};

struct GraphQLFieldMetadata {
    //this is optional as a field may be a foreign key to (Some(table_name)), or a column or the
    //table itself (None)
    database_table_name: Option<String>,
    fields: Vec<GraphQLFieldMetadata>,
}

pub fn create_schema() -> String {
    let mut schema_types = String::from("");
    let mut current_graphql_type = String::from("");
    let mut previous_name = String::from("");

    for current_row in read_database::read_type_information().unwrap().iter() {
        let type_name: &str = current_row.get("obj_name");
        let data_type: &str = current_row.get("data_type");
        let column_name: &str = current_row.get("column_name");
        let graphql_type = convert_pg_to_gql(data_type);
        if type_name != previous_name {
            on_encounter_new_type(
                &mut current_graphql_type,
                &mut schema_types,
                type_name,
                &mut previous_name,
            )
        }
        add_row(&mut current_graphql_type, column_name, &graphql_type, "");
    }

    let mut query_types = String::from("type Query{\n");
    for current_row in read_database::read_tables().unwrap().iter() {
        //if the table names match then keep building the current graphql type
        let table_name: &str = current_row.get("table_name");
        let column_name: &str = current_row.get("column_name");
        let nullable: &str = current_row.get("nullable");
        let data_type: &str = current_row.get("data_type");
        let graphql_type = convert_pg_to_gql(data_type);
        if previous_name != table_name {
            let camel_table_name = table_name.to_camel_case();
            let many_query_name = camel_table_name.to_plural();
            on_encounter_new_type(
                &mut current_graphql_type,
                &mut schema_types,
                table_name,
                &mut previous_name,
            );

            //add select many query
            query_types.push_str(&format!(
                "\t{}: [{}!]!\n",
                many_query_name,
                table_name.to_case(Case::UpperCamel)
            ));
        }
        add_row(
            &mut current_graphql_type,
            column_name,
            &graphql_type,
            nullable,
        );
    }
    //remove the leading {\n\n inserted when the previous_table_name doesn't initially match
    let schema = format!("{}{}{}", query_types, "\n}", &schema_types[3..]);
    let ast = parse_schema::<&str>(&schema).unwrap();
    for def in ast.definitions {
        if let Definition::TypeDefinition(TypeDefinition::Object(val)) = def {
            if val.name != "Query" {
                for field in val.fields {}
            }
        }
    }
    schema
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
    current_graphql_type: &mut String,
    column_name: &str,
    graphql_type: &str,
    nullable: &str,
) {
    current_graphql_type.push_str(&format!(
        "\n\t{}: {}{}",
        column_name.to_case(Case::Camel),
        graphql_type,
        nullable
    ));
}
fn on_encounter_new_type(
    current_graphql_type: &mut String,
    schema_types: &mut String,
    table_name: &str,
    previous_name: &mut String,
) {
    //close the current type;
    current_graphql_type.push_str("\n}\n\n");

    //add the current type to the schema
    schema_types.push_str(current_graphql_type);

    //reinitialize the current type with the opening
    *current_graphql_type = format!("type {} {{", table_name.to_case(Case::UpperCamel));

    //previous name becomes new name
    *previous_name = table_name.to_string();
}
