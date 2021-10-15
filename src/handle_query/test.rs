use super::*;
use async_graphql_parser::Error;
use petgraph::graph::DiGraph;
use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

struct BuildGraphInput<'a> {
    table_name: &'a str,
    query_info: Option<(&'a str, bool)>,
    terminal_fields: Vec<&'a str>,
    edge_info: Option<GraphQLEdgeInfo>,
}
fn build_graph(inputs: Vec<BuildGraphInput>) -> Poggers<PostgresBuilder> {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    let mut previous_index: NodeIndex<u32> = NodeIndex::new(0);
    for BuildGraphInput {
        terminal_fields,
        query_info,
        table_name,
        edge_info,
    } in inputs
    {
        //add this node to the graph
        let node_index = g.add_node(GraphQLType {
            terminal_fields: HashSet::from_iter(terminal_fields.iter().map(|s| s.to_string())),
            table_name: table_name.to_string(),
        });

        //if we have info to connect the previous edge to this then do so
        if let Some(edge_info) = edge_info {
            g.add_edge(previous_index, node_index, edge_info);
        }

        if let Some(query_info) = query_info {
            query_to_type.insert(
                query_info.0.to_string(),
                QueryEdgeInfo {
                    node_index,
                    is_many: query_info.1,
                },
            );
        }
        previous_index = node_index;
    }
    Poggers {
        query_to_type,
        local_id: 0,
        query_builder: PostgresBuilder {},
        g,
    }
}

fn test_sql_equality(actual: Result<String, Error>, expected: &str) {
    assert!(actual.is_ok());

    let mut actual_iter = actual.as_ref().unwrap().split_ascii_whitespace().peekable();

    let mut expected_iter = expected.split_ascii_whitespace().peekable();
    let mut actual_cumm = String::new();
    let mut expected_cumm = String::new();
    while actual_iter.peek().is_some() && expected_iter.peek().is_some() {
        let actual_val = actual_iter.next().unwrap();
        let expected_val = expected_iter.next().unwrap();
        actual_cumm.push_str(&format!("{} ", actual_val));
        expected_cumm.push_str(&format!("{} ", expected_val));
        if actual_val != expected_val {
            println!("Actual\n\n{}\n", actual_cumm);
            println!("Expected\n{}", expected_cumm);
            panic!();
        }
    }
    //println!("{}", actual_cumm);
    if actual_iter.peek().is_some() {
        println!("Actual still has vals");
        for token in actual_iter {
            print!("{} ", token);
        }
        panic!("\n");
    }
    if expected_iter.peek().is_some() {
        println!("expected still has vals");
        for token in expected_iter {
            print!("{} ", token);
        }
        println!();
        panic!();
    }
}

#[test]
fn many_select() {
    let mut pogg = build_graph(vec![BuildGraphInput {
        table_name: "exercise",
        terminal_fields: vec!["id", "bodyPart", "exerciseType"],
        query_info: Some(("exercises", true)),
        edge_info: None,
    }]);

    let actual = pogg.build_root(
        "
            query{
              exercises{
                id
                bodyPart
                exerciseType
              }
            }
        ",
    );

    let expected = "
        select to_json(
          json_build_array(__local_0__.\"id\")
        ) as \"__identifiers\",
        to_json((__local_0__.\"id\")) as \"id\",
        to_json((__local_0__.\"body_part\")) as \"bodyPart\",
        to_json((__local_0__.\"exercise_type\")) as \"exerciseType\"
        from (
          select __local_0__.*
          from \"public\".\"exercise\" as __local_0__
          order by __local_0__.\"id\" ASC
        ) __local_0__";
    test_sql_equality(actual, expected);
}

#[test]
fn simple_query_with_filter() {
    let mut pogg = build_graph(vec![BuildGraphInput {
        table_name: "exercise",
        terminal_fields: vec!["bodyPart"],
        query_info: Some(("exercise", false)),
        edge_info: None,
    }]);

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
    let mut pogg = build_graph(vec![
        BuildGraphInput {
            table_name: "workout_plan",
            terminal_fields: vec!["appUserId"],
            query_info: Some(("workoutPlans", true)),
            edge_info: None,
        },
        BuildGraphInput {
            table_name: "workout_plan_day",
            terminal_fields: vec!["workoutPlanId"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_id".to_string(),
                graphql_field_name: "workoutPlanDays".to_string(),
            }),
        },
    ]);
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

#[test]
fn three_way_join_multiple_fields() {
    let mut pogg = build_graph(vec![
        BuildGraphInput {
            table_name: "workout_plan",
            terminal_fields: vec!["id", "appUserId", "updatedAt", "createdAt"],
            query_info: Some(("workoutPlans", true)),
            edge_info: None,
        },
        BuildGraphInput {
            table_name: "workout_plan_day",
            terminal_fields: vec!["id", "workoutPlanId", "name"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_id".to_string(),
                graphql_field_name: "workoutPlanDays".to_string(),
            }),
        },
        BuildGraphInput {
            table_name: "workout_plan_exercise",
            terminal_fields: vec!["id", "ordering", "sets", "reps", "workoutPlanDayId"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_day_id".to_string(),
                graphql_field_name: "workoutPlanExercises".to_string(),
            }),
        },
    ]);
    let actual = pogg.build_root(
        "
        query{
          workoutPlans{
            appUserId
            id
            updatedAt
            createdAt
            workoutPlanDays{
              id
              workoutPlanId
              name
              workoutPlanExercises{
                id
                ordering
                sets
                reps
                workoutPlanDayId      
              }
            }
          }
        }",
    );
    let expected = "
    select to_json(
      json_build_array(__local_0__.\"id\")
    ) as \"__identifiers\",
    to_json((__local_0__.\"app_user_id\")) as \"appUserId\",
    to_json((__local_0__.\"id\")) as \"id\",
    to_json((__local_0__.\"updated_at\")) as \"updatedAt\",
    to_json((__local_0__.\"created_at\")) as \"createdAt\",
    to_json(
      (
        select coalesce(
          (
            select json_agg(__local_1__.\"object\")
            from (
              select json_build_object(
                '__identifiers'::text,
                json_build_array(__local_2__.\"id\"),
                'id'::text,
                (__local_2__.\"id\"),
                'workoutPlanId'::text,
                (__local_2__.\"workout_plan_id\"),
                'name'::text,
                (__local_2__.\"name\"),
                '@workoutPlanExercises'::text,
                (
                  select coalesce(
                    (
                      select json_agg(__local_3__.\"object\")
                      from (
                        select json_build_object(
                          '__identifiers'::text,
                          json_build_array(__local_4__.\"id\"),
                          'id'::text,
                          (__local_4__.\"id\"),
                          'ordering'::text,
                          (__local_4__.\"ordering\"),
                          'sets'::text,
                          (__local_4__.\"sets\"),
                          'reps'::text,
                          (__local_4__.\"reps\"),
                          'workoutPlanDayId'::text,
                          (__local_4__.\"workout_plan_day_id\")
                        ) as object
                        from (
                          select __local_4__.*
                          from \"public\".\"workout_plan_exercise\" as __local_4__
                          where (__local_4__.\"workout_plan_day_id\" = __local_2__.\"id\") 
                          order by __local_4__.\"id\" ASC
                        ) __local_4__
                      ) as __local_3__
                    ),
                    '[]'::json
                  )
                )
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

#[test]
fn test_arbitrary_depth_join() {
    let mut pogg = build_graph(vec![
        BuildGraphInput {
            table_name: "workout_plan",
            terminal_fields: vec!["id", "appUserId", "updatedAt", "createdAt"],
            query_info: Some(("workoutPlans", true)),
            edge_info: None,
        },
        BuildGraphInput {
            table_name: "workout_plan_day",
            terminal_fields: vec!["id", "workoutPlanId", "name"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_id".to_string(),
                graphql_field_name: "workoutPlanDays".to_string(),
            }),
        },
        BuildGraphInput {
            table_name: "workout_plan_exercise",
            terminal_fields: vec!["id", "ordering", "sets", "reps", "workoutPlanDayId"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_day_id".to_string(),
                graphql_field_name: "workoutPlanExercises".to_string(),
            }),
        },
        BuildGraphInput {
            table_name: "table1",
            terminal_fields: vec!["id", "one", "two", "three"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_exercise_id".to_string(),
                graphql_field_name: "table1s".to_string(),
            }),
        },
        BuildGraphInput {
            table_name: "table2",
            terminal_fields: vec!["id", "four", "five", "six"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "table1_id".to_string(),
                graphql_field_name: "table2s".to_string(),
            }),
        },
        BuildGraphInput {
            table_name: "table3",
            terminal_fields: vec!["id", "seven", "eight", "nine"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "table2_id".to_string(),
                graphql_field_name: "table3s".to_string(),
            }),
        },
    ]);
    let query = "
        query {
          workoutPlans {
            appUserId
            id
            updatedAt
            createdAt
            workoutPlanDays {
              id
              workoutPlanId
              name
              workoutPlanExercises {
                id
                ordering
                sets
                reps
                workoutPlanDayId
                table1s {
                  id
                  one
                  two
                  three
                  table2s {
                    id
                    four
                    five
                    six
                    table3s {
                      seven
                      eight
                      nine
                    }
                  }
                }
              }
            }
          }
        }
        ";

    let actual = pogg.build_root(query);
    let expected = "
        select to_json(
          json_build_array(__local_0__.\"id\")
        ) as \"__identifiers\",
        to_json((__local_0__.\"app_user_id\")) as \"appUserId\",
        to_json((__local_0__.\"id\")) as \"id\",
        to_json((__local_0__.\"updated_at\")) as \"updatedAt\",
        to_json((__local_0__.\"created_at\")) as \"createdAt\",
        to_json(
          (
            select coalesce(
              (
                select json_agg(__local_1__.\"object\")
                from (
                  select json_build_object(
                    '__identifiers'::text,
                    json_build_array(__local_2__.\"id\"),
                    'id'::text,
                    (__local_2__.\"id\"),
                    'workoutPlanId'::text,
                    (__local_2__.\"workout_plan_id\"),
                    'name'::text,
                    (__local_2__.\"name\"),
                    '@workoutPlanExercises'::text,
                    (
                      select coalesce(
                        (
                          select json_agg(__local_3__.\"object\")
                          from (
                            select json_build_object(
                              '__identifiers'::text,
                              json_build_array(__local_4__.\"id\"),
                              'id'::text,
                              (__local_4__.\"id\"),
                              'ordering'::text,
                              (__local_4__.\"ordering\"),
                              'sets'::text,
                              (__local_4__.\"sets\"),
                              'reps'::text,
                              (__local_4__.\"reps\"),
                              'workoutPlanDayId'::text,
                              (__local_4__.\"workout_plan_day_id\"),
                              '@table1s'::text,
                              (
                                select coalesce(
                                  (
                                    select json_agg(__local_5__.\"object\")
                                    from (
                                      select json_build_object(
                                        '__identifiers'::text,
                                        json_build_array(__local_6__.\"id\"),
                                        'id'::text,
                                        (__local_6__.\"id\"),
                                        'one'::text,
                                        (__local_6__.\"one\"),
                                        'two'::text,
                                        (__local_6__.\"two\"),
                                        'three'::text,
                                        (__local_6__.\"three\"),
                                        '@table2s'::text,
                                        (
                                          select coalesce(
                                            (
                                              select json_agg(__local_7__.\"object\")
                                              from (
                                                select json_build_object(
                                                  '__identifiers'::text,
                                                  json_build_array(__local_8__.\"id\"),
                                                  'id'::text,
                                                  (__local_8__.\"id\"),
                                                  'four'::text,
                                                  (__local_8__.\"four\"),
                                                  'five'::text,
                                                  (__local_8__.\"five\"),
                                                  'six'::text,
                                                  (__local_8__.\"six\"),
                                                  '@table3s'::text,
                                                  (
                                                    select coalesce(
                                                      (
                                                        select json_agg(__local_9__.\"object\")
                                                        from (
                                                          select json_build_object(
                                                            '__identifiers'::text,
                                                            json_build_array(__local_10__.\"id\"),
                                                            'seven'::text,
                                                            (__local_10__.\"seven\"),
                                                            'eight'::text,
                                                            (__local_10__.\"eight\"),
                                                            'nine'::text,
                                                            (__local_10__.\"nine\")
                                                          ) as object
                                                          from (
                                                            select __local_10__.*
                                                            from \"public\".\"table3\" as __local_10__
                                                            where (__local_10__.\"table2_id\" = __local_8__.\"id\") 
                                                            order by __local_10__.\"id\" ASC
                                                          ) __local_10__
                                                        ) as __local_9__
                                                      ),
                                                      '[]'::json
                                                    )
                                                  )
                                                ) as object
                                                from (
                                                  select __local_8__.*
                                                  from \"public\".\"table2\" as __local_8__
                                                  where (__local_8__.\"table1_id\" = __local_6__.\"id\")
                                                  order by __local_8__.\"id\" ASC
                                                ) __local_8__
                                              ) as __local_7__
                                            ),
                                            '[]'::json
                                          )
                                        )
                                      ) as object
                                      from (
                                        select __local_6__.*
                                        from \"public\".\"table1\" as __local_6__
                                        where (__local_6__.\"workout_plan_exercise_id\" = __local_4__.\"id\") 
                                        order by __local_6__.\"id\" ASC
                                      ) __local_6__
                                    ) as __local_5__
                                  ),
                                  '[]'::json
                                )
                              )
                            ) as object
                            from (
                              select __local_4__.*
                              from \"public\".\"workout_plan_exercise\" as __local_4__
                              where (__local_4__.\"workout_plan_day_id\" = __local_2__.\"id\") 
                              order by __local_4__.\"id\" ASC
                            ) __local_4__
                          ) as __local_3__
                        ),
                        '[]'::json
                      )
                    )
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

#[test]
fn test_many_to_one() {
    let mut pogg = build_graph(vec![
        BuildGraphInput {
            table_name: "workout_plan_day",
            terminal_fields: vec!["workoutPlanId", "name"],
            query_info: Some(("workoutPlanDays", true)),
            edge_info: None,
        },
        BuildGraphInput {
            table_name: "workout_plan",
            terminal_fields: vec!["id", "appUserId", "name"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: false,
                foreign_key_name: "workout_plan_id".to_string(),
                graphql_field_name: "workoutPlan".to_string(),
            }),
        },
    ]);
    let query = "
        query{
          workoutPlanDays{
            workoutPlanId
            name
            workoutPlan{
              id
              name
              appUserId
            }
          }
        }";
    let actual = pogg.build_root(query);
    let expected = "
        select to_json(
          json_build_array(__local_0__.\"id\")
        ) as \"__identifiers\",
        to_json((__local_0__.\"workout_plan_id\")) as \"workoutPlanId\",
        to_json((__local_0__.\"name\")) as \"name\",
        to_json(
          (
            select json_build_object(
              '__identifiers'::text,
              json_build_array(__local_1__.\"id\"),
              'id'::text,
              (__local_1__.\"id\"),
              'name'::text,
              (__local_1__.\"name\"),
              'appUserId'::text,
              (__local_1__.\"app_user_id\")
            ) as object
            from \"public\".\"workout_plan\" as __local_1__
            where (__local_0__.\"workout_plan_id\" = __local_1__.\"id\") 
          )
        ) as \"@workoutPlan\"
        from (
          select __local_0__.*
          from \"public\".\"workout_plan_day\" as __local_0__
          order by __local_0__.\"id\" ASC
        ) __local_0__";
    test_sql_equality(actual, expected);
}

#[test]
fn test_nested_many_to_one() {
    let mut pogg = build_graph(vec![
        BuildGraphInput {
            table_name: "workout_plan",
            terminal_fields: vec!["id", "name", "createdAt"],
            query_info: Some(("workoutPlans", true)),
            edge_info: None,
        },
        BuildGraphInput {
            table_name: "workout_plan_day",
            terminal_fields: vec!["name"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_id".to_string(),
                graphql_field_name: "workoutPlanDays".to_string(),
            }),
        },
        BuildGraphInput {
            table_name: "workout_plan_exercise",
            terminal_fields: vec!["exerciseId"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: true,
                foreign_key_name: "workout_plan_day_id".to_string(),
                graphql_field_name: "workoutPlanExercises".to_string(),
            }),
        },
        BuildGraphInput {
            table_name: "exercise",
            terminal_fields: vec!["id", "bodyPart"],
            query_info: None,
            edge_info: Some(GraphQLEdgeInfo {
                one_to_many: false,
                foreign_key_name: "exercise_id".to_string(),
                graphql_field_name: "exercise".to_string(),
            }),
        },
    ]);
    let query = "
        query{
          workoutPlans{
            id
            name
            createdAt
            workoutPlanDays{
              name
              workoutPlanExercises{
                exerciseId
                exercise{
                  id
                  bodyPart
                }
              }
            }
          }
        }";
    let actual = pogg.build_root(query);
    let expected = "
        select to_json(
          json_build_array(__local_0__.\"id\")
        ) as \"__identifiers\",
        to_json((__local_0__.\"id\")) as \"id\",
        to_json((__local_0__.\"name\")) as \"name\",
        to_json((__local_0__.\"created_at\")) as \"createdAt\",
        to_json(
          (
            select coalesce(
              (
                select json_agg(__local_1__.\"object\")
                from (
                  select json_build_object(
                    '__identifiers'::text,
                    json_build_array(__local_2__.\"id\"),
                    'name'::text,
                    (__local_2__.\"name\"),
                    '@workoutPlanExercises'::text,
                    (
                      select coalesce(
                        (
                          select json_agg(__local_3__.\"object\")
                          from (
                            select json_build_object(
                              '__identifiers'::text,
                              json_build_array(__local_4__.\"id\"),
                              'exerciseId'::text,
                              (__local_4__.\"exercise_id\"),
                              '@exercise'::text,
                              (
                                select json_build_object(
                                  '__identifiers'::text,
                                  json_build_array(__local_5__.\"id\"),
                                  'id'::text,
                                  (__local_5__.\"id\"),
                                  'bodyPart'::text,
                                  (__local_5__.\"body_part\")
                                ) as object
                                from \"public\".\"exercise\" as __local_5__
                                where (__local_4__.\"exercise_id\" = __local_5__.\"id\") 
                              )
                            ) as object
                            from (
                              select __local_4__.*
                              from \"public\".\"workout_plan_exercise\" as __local_4__
                              where (__local_4__.\"workout_plan_day_id\" = __local_2__.\"id\") 
                              order by __local_4__.\"id\" ASC
                            ) __local_4__
                          ) as __local_3__
                        ),
                        '[]'::json
                      )
                    )
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
        ) __local_0__
    ";
    test_sql_equality(actual, expected);
}
