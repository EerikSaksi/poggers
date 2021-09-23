use super::{Poggers, SqlOperation};
use graphql_parser::query::ParseError;
use std::collections::HashMap;
fn test_sql_equality(actual: Result<String, ParseError>, expected: &str) {
    assert!(actual.is_ok());
    actual
        .unwrap()
        .split_ascii_whitespace()
        .zip(expected.split_ascii_whitespace())
        .for_each(|(a, b)| assert_eq!(a, b));
}

#[test]
fn simple_query() {
    let mut graphql_query_to_operation = HashMap::new();
    graphql_query_to_operation.insert(String::from("exercises"), SqlOperation{is_many: true, table_name: "exercise"});
    let pogg = Poggers{graphql_query_to_operation};
    let actual = pogg.build_root(
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
    let mut graphql_query_to_operation = HashMap::new();
    graphql_query_to_operation.insert(String::from("exercise"), SqlOperation{is_many: false, table_name: "exercise"});
    let pogg = Poggers{graphql_query_to_operation};
    let actual = pogg.build_root(
        "
            query {
              exercise(id: 123) {
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
                          __local_0__.\"id\" = 123
                        )";
    test_sql_equality(actual, expected);
}
