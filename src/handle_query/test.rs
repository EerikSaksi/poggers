use super::{GraphQLEdgeInfo, GraphQLType, Poggers, QueryEdgeInfo};
use graphql_parser::query::ParseError;
use petgraph::graph::DiGraph;
use std::collections::{HashMap, HashSet};

fn test_sql_equality(actual: Result<String, ParseError>, expected: &str) {
    assert!(actual.is_ok());
    actual
        .as_ref()
        .unwrap()
        .split_ascii_whitespace()
        .zip(expected.split_ascii_whitespace())
        .for_each(|(a, b)| assert_eq!(a, b));

    //zip will only compare the smaller elements list so we should also make sure the sizes match
    assert_eq!(
        actual.unwrap().split_ascii_whitespace().count(),
        expected.split_ascii_whitespace().count()
    )
}

#[test]
fn simple_query() {
    let mut type_graph: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();

    let mut terminal_fields = HashSet::new();
    terminal_fields.insert("bodyPart".to_string());

    let node_index = type_graph.add_node(GraphQLType {
        table_name: "exercise".to_string(),
        terminal_fields,
    });

    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "exercises".to_string(),
        QueryEdgeInfo {
            is_many: true,
            node_index,
        },
    );

    let pogg = Poggers {
        type_graph,
        query_to_type,
    };
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
    let mut type_graph: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();

    let mut terminal_fields = HashSet::new();
    terminal_fields.insert("bodyPart".to_string());

    let node_index = type_graph.add_node(GraphQLType {
        table_name: "exercise".to_string(),
        terminal_fields,
    });

    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "exercise".to_string(),
        QueryEdgeInfo {
            is_many: false,
            node_index,
        },
    );

    let pogg = Poggers {
        type_graph,
        query_to_type,
    };

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
//#[test]
//fn join() {
//    let mut graphql_query_to_operation = HashMap::new();
//    graphql_query_to_operation.insert(
//        String::from("exercise"),
//        SqlOperation {
//            is_many: false,
//            table_name: "exercise",
//        },
//    );
//    let pogg = Poggers {
//        graphql_query_to_operation,
//    };
//    let actual = pogg.build_root(
//        "
//        let query{
//          workoutPlans{
//            appUserId
//            workoutPlanDays{
//              workoutPlanId
//            }
//          }
//        }"
//    );
//    let expected = "
//        select to_json(
//          json_build_array(__local_0__.\"id\")
//        ) as \"__identifiers\",
//        to_json((__local_0__.\"app_user_id\")) as \"appUserId\",
//        to_json(
//          (
//            select coalesce(
//              (
//                select json_agg(__local_1__.\"object\")
//                from (
//                  select json_build_object(
//                    '__identifiers'::text,
//                    json_build_array(__local_2__.\"id\"),
//                    'workoutPlanId'::text,
//                    (__local_2__.\"workout_plan_id\")
//                  ) as object
//                  from (
//                    select __local_2__.*
//                    from \"public\".\"workout_plan_day\" as __local_2__
//                    where (__local_2__.\"workout_plan_id\" = __local_0__.\"id\") and (TRUE) and (TRUE)
//                    order by __local_2__.\"id\" ASC
//                  ) __local_2__
//                ) as __local_1__
//              ),
//              '[]'::json
//            )
//          )
//        ) as \"@workoutPlanDays\"
//        from (
//          select __local_0__.*
//          from \"public\".\"workout_plan\" as __local_0__
//          where (TRUE) and (TRUE)
//          order by __local_0__.\"id\" ASC
//        ) __local_0__";
//    test_sql_equality(actual, expected);
//}
