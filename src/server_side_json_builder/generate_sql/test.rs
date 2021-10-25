use super::*;
use crate::internal_schema_info::create;
use std::collections::HashSet;
use std::iter::FromIterator;

fn test_sql_equality(actual: String, expected: &str) {
    let mut actual_iter = actual.split_ascii_whitespace().peekable();

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
fn one_way_join() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let plan_node = g.add_node(GraphQLType {
        table_name: "workout_plan".to_string(),
        primary_keys: vec!["id".to_string()],
        terminal_fields: HashSet::from_iter(["id", "name"].iter().map(|s| s.to_string())),
    });

    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "workoutPlans".to_string(),
        QueryEdgeInfo {
            is_many: true,
            node_index: plan_node,
        },
    );

    let day_node = g.add_node(GraphQLType {
        table_name: "workout_plan_day".to_string(),
        primary_keys: vec!["workoutPlanId".to_string()],
        terminal_fields: HashSet::from_iter(
            ["workoutPlanId", "name", "id"]
                .iter()
                .map(|s| s.to_string()),
        ),
    });

    g.add_edge(
        day_node,
        plan_node,
        GraphQLEdgeInfo {
            foreign_keys: vec!["workout_plan_id".to_string()],
            graphql_field_name: ("workoutPlanDays".to_string(), "workoutPlanDay".to_string()),
        },
    );
    let mut pogg = ServerSidePoggers {
        query_to_type,
        g,
        local_id: 0,
        num_select_cols: 0,
    };

    let actual = pogg.build_root(
        "
        query{
          workoutPlans{
            name
            workoutPlanDays{
              workoutPlanId
              name
              id
            }
          }
        }",
    );
    let expected = "SELECT __table_0__.name as __t0_c0__, __table_0__.id AS __t0_pk0__, __table_1__.workout_plan_id as __t1_c0__, __table_1__.name as __t1_c1__, __table_1__.id as __t1_c2__ FROM workout_plan AS __table_0__ JOIN workout_plan_day AS __table_1__ ON __table_0__.id = __table_1__.workout_plan_id ORDER BY __table_0__.id";
    match actual {
        Ok(actual) => {
            test_sql_equality(actual.0, expected);
            assert_eq!(
                actual.1,
                vec![
                    TableQueryInfo {
                        parent_key_name: "workoutPlans".to_string(),
                        graphql_fields: vec!["name".to_string()],
                        column_offset: 0
                    },
                    TableQueryInfo {
                        parent_key_name: "workoutPlanDays".to_string(),
                        graphql_fields: vec![
                            "workoutPlanId".to_string(),
                            "name".to_string(),
                            "id".to_string(),
                        ],
                        column_offset: 2
                    },
                ]
            );
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn column_offsets() {
    let mut pogg = create("postgres://eerik:Postgrizzly@localhost:5432/pets");
    let query = "
        query{
          siteUsers{
            reputation
            views
            upvotes
            downvotes
            posts{
              id
              posttypeid
            }
          }
        }";
    let (_, table_query_infos) = pogg.build_root(query).unwrap();
    assert_eq!(table_query_infos.get(0).unwrap().column_offset, 0);
    assert_eq!(table_query_infos.get(1).unwrap().column_offset, 5);
}
