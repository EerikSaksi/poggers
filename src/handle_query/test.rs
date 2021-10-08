use super::{GraphQLEdgeInfo, GraphQLType, Poggers, QueryEdgeInfo};
use petgraph::graph::DiGraph;
use async_graphql_parser::Error;
use std::collections::{HashMap, HashSet};


fn test_sql_equality(actual: Result<String, Error>, expected: &str) {
    assert!(actual.is_ok());

    let mut actual_iter = actual
        .as_ref()
        .unwrap()
        .split_ascii_whitespace().peekable();

    let mut expected_iter = expected.split_ascii_whitespace().peekable();
    let mut actual_cumm = String::new();
    let mut expected_cumm = String::new();
    while actual_iter.peek().is_some() && expected_iter.peek().is_some() {
        let actual_val = actual_iter.next().unwrap();
        let expected_val = expected_iter.next().unwrap();
        actual_cumm.push_str(&format!("{} ", actual_val));
        expected_cumm.push_str(&format!("{} ", expected_val));
        if actual_val != expected_val {
            println!("\n{}\n", actual_cumm);
            println!("{}", expected_cumm);
            panic!();
        }
    }
    println!("{}", actual_cumm);
    if actual_iter.peek().is_some() {
        println!("Actual still has vals");
        for token in actual_iter {
            print!("{} ", token);
        }
        panic!("\n");
    }
    if expected_iter.peek().is_some() {
        println!("expected still has vals");
        for (i, token) in expected_iter.enumerate() {
            print!("{} ", token);
        }
        println!();
        panic!();
    }
}

#[test]
fn simple_query() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();

    let mut terminal_fields = HashSet::new();
    terminal_fields.insert("bodyPart".to_string());

    let node_index = g.add_node(GraphQLType {
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

    let mut pogg = Poggers {
        g,
        query_to_type,
        local_id: 0,
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
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();

    let mut terminal_fields = HashSet::new();
    terminal_fields.insert("bodyPart".to_string());

    let node_index = g.add_node(GraphQLType {
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

    let mut pogg = Poggers {
        g,
        query_to_type,
        local_id: 0,
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
#[test]
fn join() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();

    let mut workout_plan_terminal_fields = HashSet::new();
    workout_plan_terminal_fields.insert("appUserId".to_string());

    let node_index = g.add_node(GraphQLType {
        table_name: "workout_plan".to_string(),
        terminal_fields: workout_plan_terminal_fields,
    });
    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "workoutPlans".to_string(),
        QueryEdgeInfo {
            is_many: true,
            node_index,
        },
    );

    let mut days_terminal_fields = HashSet::new();
    days_terminal_fields.insert("workoutPlanId".to_string());
    let day_node_index = g.add_node(GraphQLType {
        table_name: "workout_plan_day".to_string(),
        terminal_fields: days_terminal_fields,
    });

    g.add_edge(
        node_index,
        day_node_index,
        GraphQLEdgeInfo {
            is_many: true,
            foreign_key_name: "workout_plan_id".to_string(),
            graphql_field_name: "workoutPlanDays".to_string(),
        },
    );

    let mut pogg = Poggers {
        g,
        query_to_type,
        local_id: 0,
    };
    let actual = pogg.build_root(
        "
        query{
          workoutPlans{
            appUserId
            workoutPlanDays{
              workoutPlanId
            }
          }
        }",
    );
    let expected = "
        select to_json(
          json_build_array(__local_0__.\"id\")
        ) as \"__identifiers\",
        to_json((__local_0__.\"app_user_id\")) as \"appUserId\",
        to_json(
          (
            select coalesce(
              (
                select json_agg(__local_1__.\"object\")
                from (
                  select json_build_object(
                    '__identifiers'::text,
                    json_build_array(__local_2__.\"id\"),
                    'workoutPlanId'::text,
                    (__local_2__.\"workout_plan_id\")
                  ) as object
                  from (
                    select __local_2__.*
                    from \"public\".\"workout_plan_day\" as __local_2__
                    where (__local_2__.\"workout_plan_id\" = __local_0__.\"id\") 
                    order by __local_2__.\"id\" ASC
                  ) __local_2__
                ) as __local_1__
              ),
              '[]'::json
            )
          )
        ) as \"@workoutPlanDays\"
        from (
          select __local_0__.*
          from \"public\".\"workout_plan\" as __local_0__
          order by __local_0__.\"id\" ASC
        ) __local_0__";
    test_sql_equality(actual, expected);
}
