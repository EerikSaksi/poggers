use graphql_parser::query::{parse_query, Definition, OperationDefinition};

fn main() {
    let ast = parse_query::<&str>("query MyQuery { field1, field2 }");
    if let Ok(tree) = ast{
        let definition = &tree.definitions[0];
        if let Definition::Operation(operation_definition) = definition {
            if let OperationDefinition::Query(query) = operation_definition {
                println!("{}", query.selection_set);

            }
        }
    }
}
