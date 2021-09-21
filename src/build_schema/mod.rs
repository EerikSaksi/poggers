use crate::handle_query::{Poggers, SqlOperation};
use convert_case::{Case, Casing};
use graphql_parser::parse_schema;
use graphql_parser::schema::Document;
use inflector::Inflector;
use postgres::{Client, NoTls, Row};
use std::collections::HashMap;
pub fn client_connect() -> Result<Vec<Row>, postgres::Error> {
    let mut client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls)?;

    let query_res = client.query(
        "
            select table_name, column_name, data_type,
            case is_nullable
                when 'NO' then '!'
                when 'YES' then ''
            end as nullable
            from information_schema.columns where table_schema = 'public' 
            group by table_name, column_name, data_type, nullable;
",
        &[],
    )?;
    Ok(query_res)
}

pub fn create_schema<'a>() -> (String, Poggers<'a>) {
    let mut previous_table_name = String::from("");
    let mut schema_types = String::from("");
    let mut query_types = String::from("type query{\n");
    let mut current_graphql_type = String::from("");
    let poggers_hashmap: HashMap<String, Poggers> = HashMap::new();

    let rows = client_connect().unwrap();
    while let Some(current_row) = rows.iter().next() {
        //if the table names match then keep building the current graphql type
        let table_name: &str = current_row.get("table_name");
        let column_name: &str = current_row.get("column_name");
        let data_type: &str = current_row.get("data_type");
        let nullable: &str = current_row.get("nullable");
        let graphql_type = match data_type {
            "integer" => "Int",
            "smallint" => "Int",
            "boolean" => "Boolean",
            "character varying" => "String",
            "text" => "String",
            "timestamp with time zone" => "Datetime",
            "timestamp" => "Datetime",
            "float" => "Float",
            "double precision" => "Float",
            _ => "Ooopsie",
        };
        if previous_table_name != table_name {
            //close the current type;
            current_graphql_type.push_str("\n}\n\n");

            //add the current type to the schema
            schema_types.push_str(&current_graphql_type);

            let camel_table_name = table_name.to_camel_case();
            let upper_camel_table_name = table_name.to_case(Case::UpperCamel);
            let many_query_name = camel_table_name.to_plural();

            query_types.push_str(&format!(
                "\t{}: [{}!]!\n",
                many_query_name,
                upper_camel_table_name
            ));
            poggers_hashmap.insert(many_query_name, SqlOperation{table_name, is_many: true});

            //reinitialize the current type with the opening
            current_graphql_type = format!("type {} {{", upper_camel_table_name);

            previous_table_name = table_name.to_string();
        }

        current_graphql_type.push_str(&format!(
            "\n\t{}: {}{}",
            column_name.to_case(Case::Camel),
            graphql_type,
            nullable
        ));
    }
    //remove the leading {\n\n inserted when the previous_table_name doesn't initially match
    let complete_schema = format!("{}{}{}", query_types, "\n}", &schema_types[3..]);

    (complete_schema, Poggers{graphql_query_to_operation: poggers_hashmap})
}
