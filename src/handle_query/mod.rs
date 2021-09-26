use convert_case::{Case, Casing};
use graphql_parser::query::{
    parse_query, Definition, OperationDefinition, ParseError, Query, Selection,
};
use std::collections::HashMap;
use std::time::Instant;
#[cfg(test)]
#[path = "./test.rs"]
mod test;

pub struct SqlOperation<'a> {
    pub table_name: &'a str,
    pub is_many: bool,
}
pub struct Poggers<'a> {
    pub graphql_query_to_operation: HashMap<String, SqlOperation<'a>>,
}

impl Poggers<'_> {
    pub fn build_root(&self, query: &str) -> Result<String, ParseError> {
        let ast = parse_query::<&str>(query)?;
        let definition = ast.definitions.get(0).unwrap();
        match definition {
            Definition::Operation(operation_definition) => {
                let before = Instant::now();
                let val = self.build_operation_definition(operation_definition);
                println!("Elapsed time: {:.2?}", before.elapsed());
                Ok(val)
            }
            Definition::Fragment(_fragment_definition) => {
                Ok(String::from("Definition::Fragment not implemented yet"))
            }
        }
    }

    fn build_operation_definition<'a>(
        &self,
        operation_definition: &'a OperationDefinition<&'a str>,
    ) -> String {
        match operation_definition {
            OperationDefinition::Query(query) => self.build_query(query),
            OperationDefinition::Subscription(_) => {
                String::from("Subscription not yet implemented")
            }

            OperationDefinition::Mutation(_) => String::from("Mutation not yet implemented"),

            OperationDefinition::SelectionSet(_) => {
                String::from("SelectionSet not yet implemented")
            }
        }
    }

    fn build_query<'a>(&self, query: &'a Query<&'a str>) -> String {
        let mut query_string = String::from(
            "select to_json(
              json_build_array(__local_0__.\"id\")
            ) as \"__identifiers\",
        ",
        );
        query_string.push_str(&self.build_selection(&query.selection_set.items[0]));
        query_string
    }

    fn build_selection<'a>(&self, selection: &'a Selection<&'a str>) -> String {
        match selection {
            Selection::Field(field) => {
                //leaf node
                if field.selection_set.items.is_empty() {
                    //simply add json field with the field name in snake case
                    let mut to_return = String::from("to_json((__local_0__.\"");
                    to_return.push_str(&field.name.to_case(Case::Snake));
                    to_return.push_str("\")) as \"");
                    to_return.push_str(field.name);
                    to_return.push_str("\",");
                    to_return
                } else {
                    //first we recursively get all queries from the children
                    let mut query_string = field
                        .selection_set
                        .items
                        .iter()
                        .map(|selection| self.build_selection(selection))
                        .collect::<Vec<String>>()
                        .join("");

                    //the last select has an unnecessary comma which causes syntax errors
                    query_string.pop();

                    match self.graphql_query_to_operation.get(field.name) {
                        Some(SqlOperation {
                            table_name,
                            is_many,
                        }) => {
                            if *is_many {
                                //select all the child fields from this
                                //
                                query_string
                                    .push_str(" from ( select __local_0__.* from \"public\".\"");
                                query_string.push_str(table_name);
                                //query_string.push_str(&field.name.to_singular());
                                query_string.push_str(
                                    "\" as __local_0__ order by __local_0__.\"id\" ASC )",
                                );
                            } else if let Some((name, val)) = field.arguments.get(0) {
                                query_string.push_str(" from \"public\".\"");
                                query_string.push_str(table_name);
                                //query_string.push_str(&field.name.to_singular());
                                query_string.push_str("\" as __local_0__");
                                query_string.push_str(" where ( __local_0__.\"");
                                query_string.push_str(name);
                                query_string.push_str("\" = ");
                                query_string.push_str(&val.to_string());
                                query_string.push_str(" )");
                            }
                        }
                        None => panic!("graphql_query_to_operation doesn't contain {}", field.name),
                    }
                    query_string
                }
            }
            Selection::FragmentSpread(_) => String::from("FragmentSpread not implemented"),
            Selection::InlineFragment(_) => String::from("InlineFragment not implemented"),
        }
    }
}
