use graphql_parser::query::{parse_query, Definition, OperationDefinition};
fn main() {
    println!("{}", build_query());
}

fn build_root() -> String {
    let ast = parse_query::<&str>("query MyQuery { exercises {id, bodyPart}}");
    if let Ok(tree) = ast {
        for definition in tree.definitions.iter(){
            match definition {
                Definition::Operation(operation_definition) => build_operation_definition(operation_definition),
                Definition::Fragment(fragment_definition) => return String("Definition::Fragment not implemented yet")

            }
        }
        if let Definition::Operation(operation_definition) = definition {
            if let OperationDefinition::Query(query) = operation_definition {
                let to_return = String::from("");
                for item in query.selection_set.items.iter(){
                    match item {
                        Selection::Field()
                    }
                }
                return to_return;
            }
        }
    }
    String::from("uh oh")
}

fn build_operation_definition(operation_definition: &OperationDefinition<&str>){

}


fn build_query(query: ) -> &str{
}
