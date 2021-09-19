use convert_case::{Case, Casing};
use graphql_parser::query::{parse_query, Definition, OperationDefinition, Query, Selection};
use inflector::Inflector;

pub fn build_root(query: &str) -> Option<String> {
    let ast = parse_query::<&str>(query);
    if let Ok(tree) = ast {
        for definition in tree.definitions.iter() {
            match definition {
                Definition::Operation(operation_definition) => {
                    return Some(build_operation_definition(operation_definition));
                }
                Definition::Fragment(_fragment_definition) => {
                    return Some(String::from("Definition::Fragment not implemented yet"));
                }
            }
        }
    }
    None
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
        ) as \"__identifiers\",\n"
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
                    "to_json((__local_0__.\"{}\")) as \"{}\",",
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
                    .collect::<Vec<String>>()
                    .join("");

                //remove the last trailing comma of the last select
                println!("{}", children.len());

                //the last select has an unnecessary comma which causes syntax errors
                let without_last_comma = &children[0..children.len() - 1];

                //select all the child fields from this
                format!(
                    "{}
                          from (
                              select __local_0__.*
                              from \"public\".\"{}\" as __local_0__
                              where (TRUE) and (TRUE)
                              order by __local_0__.\"id\" ASC
                          )
                        ",
                    without_last_comma,
                    field.name.to_singular()
                )
            }
        }
        Selection::FragmentSpread(_) => String::from("FragmentSpread not implemented"),
        Selection::InlineFragment(_) => String::from("InlineFragment not implemented"),
    }
}
#[cfg(test)]
mod tests {
    use super::build_root;

    fn test_sql_equality(actual: Option<String>, expected: &str) {
        assert!(actual.is_some());
        actual
            .unwrap()
            .split_ascii_whitespace()
            .zip(expected.split_ascii_whitespace())
            .for_each(|(a, b)| assert_eq!(a,b));
    }

    #[test]
    fn simple_query() {
        let actual = build_root(
            "
            query {
                exercises {
                    bodyPart
                }
            }
        ",
        );

        let expected = "select to_json(
                          json_build_array(__local_0__.\"id\")
                        ) as \"__identifiers\",
                        to_json((__local_0__.\"body_part\")) as \"bodyPart\"
                        from (
                          select __local_0__.*
                          from \"public\".\"exercise\" as __local_0__
                          order by __local_0__.\"id\" ASC
                        ) __local_0__";
        test_sql_equality(actual, expected);
    }

    #[test]
    fn simple_query_with_filter() {
        let actual = build_root(
            "
            query {
              exercise(id: 1) {
                bodyPart
              } 
            }",
        );
        let expected = "select to_json(
                          json_build_array(__local_0__.\"id\")
                        ) as \"__identifiers\",
                        to_json((__local_0__.\"body_part\")) as \"bodyPart\"
                        from \"public\".\"exercise\" as __local_0__
                        where (
                          __local_0__.\"id\" = $1
                        )";
        test_sql_equality(actual, expected);
    }
}
