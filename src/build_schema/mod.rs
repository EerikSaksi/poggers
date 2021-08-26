use convert_case::{Case, Casing};
use postgres::{Client, NoTls};
pub fn client_connect() -> Result<String, postgres::Error> {
    let mut client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls)?;

    let mut schema = String::from("");
    let mut current_graphql_type = String::from("type{{\n");

    let rows = client.query("select table_name, column_name, data_type from information_schema.columns where table_schema = 'public' group by table_name, data_type, column_name", &[])?.iter().peekable();

    let previous_row: &str;
    match rows.peek(){
        Some(first_row) => previous_row = first_row,
        None => panic!("Your database has no columns!")
    };


    while let Some(current_row) = rows.next() {
        //if the table names match then keep building the current graphql type
        let table_name: &str = current_row.get("table_name");
        let column_name: &str = current_row.get("column_name");
        let data_type: &str = current_row.get("data_type");
        let graphql_type = match data_type {
            "integer" => "Int",
            "smallint" => "Int",
            "boolean" => "Boolean",
            "character varying" => "String",
            "Text" => "String",
            _ => "Ooopsie",
        };
        let previous_table_name: &str = previous_row.get("table_name");
        if previous_table_name != table_name {
            //close the current type;
            current_graphql_type.push_str("\n}}\n");

            //add the current type to the schema
            schema.push_str(&current_graphql_type);

            //reinitialize the current type with the opening
            current_graphql_type = String::from("type{{\n");
        }

        current_graphql_type.push_str(&format!(
            "\n\t{}: {}",
            column_name.to_case(Case::Camel),
            graphql_type
        ));
    }
    Ok("It went ok".to_string())
}
