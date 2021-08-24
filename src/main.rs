use convert_case::{Case, Casing};
use graphql_parser::query::{parse_query, Definition, OperationDefinition, Query, Selection};
use std::time::Instant;
fn main() {
    let before = Instant::now();
    println!("{}", build_root());
    println!("Elapsed time: {:.2?}", before.elapsed());
}

fn build_root() -> String {
    let ast = parse_query::<&str>("query MyQuery { exercises {id, bodyPart}}");
    if let Ok(tree) = ast {
        for definition in tree.definitions.iter() {
            match definition {
                Definition::Operation(operation_definition) => {
                    return build_operation_definition(operation_definition)
                }
                Definition::Fragment(_fragment_definition) => {
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
        OperationDefinition::Subscription(_) => {
            return String::from("Subscription not yet implemented");
        }
        OperationDefinition::Mutation(_) => {
            return String::from("Mutation not yet implemented");
        }
        OperationDefinition::SelectionSet(_) => {
            return String::from("SelectionSet not yet implemented");
        }
    }
}

fn build_query<'a>(query: &'a Query<&'a str>) -> String {
    let mut hardcoded = "select to_json(
      json_build_array(__local_0__.\"id\")
    ) as \"__identifiers\","
        .to_owned();

    let dynamic = build_selection(&query.selection_set.items[0]);
    hardcoded.push_str(&dynamic);
    return hardcoded;
}

fn build_selection<'a>(selection: &'a Selection<&'a str>) -> String {
    match selection {
        Selection::Field(field) => {
            //no children
            if field.selection_set.items.is_empty() {
                //simply add json field with the field name in snake case 
                format!(
                    "to_json((__local_0__.\"{}\")) as \"{}\",
                    ",
                    field.name.to_case(Case::Snake),
                    field.name,
                )
            } else {
                //otherwise call this method recursively on all children and join their outputs
                //together
                let children = field
                    .selection_set
                    .items
                    .iter()
                    .map(|selection| build_selection(selection))
                    .fold(String::new(), |a, b| format!("{}{}", a, b));

                //remove the last trailing comma of the last select
                println!("{}", children.len());
                
                //the last select has an unnecessary comma which causes syntax errors
                let without_last_comma = &children[0..children.len() - 22];

                //select all the child fields from this 
                format!(
                    "{}
                     from \"{}\" as __local_0__
                    ",
                    without_last_comma,
                    "exercise"
                )
            }
        }
        Selection::FragmentSpread(_) => String::from("FragmentSpread not implemented"),
        Selection::InlineFragment(_) => String::from("InlineFragment not implemented"),
    }
}

//from "public"."app_user" as __local_0__
//where (
//  __local_0__."id" = $1
//) and (TRUE) and (TRUE)
