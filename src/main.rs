use graphql_parser::query::{parse_query, Definition, OperationDefinition, Query};
fn main() {
    println!("{}", build_root());
}

fn build_root() -> String {
    let ast = parse_query::<&str>("query MyQuery { exercises {id, bodyPart}}");
    if let Ok(tree) = ast {
        for definition in tree.definitions.iter() {
            match definition {
                Definition::Operation(operation_definition) => {
                    build_operation_definition(operation_definition)
                }
                Definition::Fragment(fragment_definition) => {
                    return String::from("Definition::Fragment not implemented yet")
                }
            }
        }
    }
    String::from("uh oh")
}

fn build_operation_definition<'a>(
    operation_definition: &'a OperationDefinition<&'a str>,
) -> String {
    match operation_definition {
        OperationDefinition::Query(query) => build_query(query),
        OperationDefinition::Subscription() => {
            return String::from("Subscription not yet implemented");
        }
        OperationDefinition::Mutation() => {
            return String::from("Mutation not yet implemented");
        }
        OperationDefinition::SelectionSet() => {
            return String::from("SelectionSet not yet implemented");
        }
    }
}

fn build_query<'a>(query: &'a Query<&'a str>) -> String {
    let constant = "select to_json(
      json_build_array(__local_0__.\"id\")
    ) as \"__identifiers\",";
    String::from("stinky")
}
//to_json((__local_0__."body_part")) as "bodyPart",
//to_json((__local_0__."exercise_type")) as "exerciseType"
//from (
//  select __local_0__.*
//  from "public"."exercise" as __local_0__
//  where (TRUE) and (TRUE)
//  order by __local_0__."id" ASC
//) __local_0__"
