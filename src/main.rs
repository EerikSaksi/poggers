use build_schema::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};
use handle_query::Poggers;
use petgraph::graph::DiGraph;

use std::fs::OpenOptions;
use std::io::prelude::*;
mod build_schema;
mod handle_query;
use graphql_parser::query::parse_query;
use std::time::Instant;
use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

fn main() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();

    let mut workout_plan_terminal_fields = HashSet::new();
    workout_plan_terminal_fields.insert("appUserId".to_string());
    workout_plan_terminal_fields.insert("id".to_string());
    workout_plan_terminal_fields.insert("appUserId".to_string());
    workout_plan_terminal_fields.insert("name".to_string());
    workout_plan_terminal_fields.insert("createdAt".to_string());
    workout_plan_terminal_fields.insert("updatedAt".to_string());
    workout_plan_terminal_fields.insert("numFollowers".to_string());
    workout_plan_terminal_fields.insert("requiresEquipment".to_string());
    workout_plan_terminal_fields.insert("progressionScheme".to_string());

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

    let days_terminal_fields = HashSet::from_iter(
        [
            "id",
            "workoutPlanId",
            "name",
            "createdAt",
            "updatedAt",
            "ordering",
            "bodypartFocus",
            "estimatedLength",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
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
    for i in 0..10000 {
        println!("{}", i);
        let query = "
query{
  workoutPlans{
    id
    appUserId
    name
    createdAt
    updatedAt
    numFollowers
    requiresEquipment
    progressionScheme
    workoutPlanDays{
      id
      workoutPlanId
      name
      createdAt
      updatedAt
      ordering
      bodypartFocus
      estimatedLength
      workoutPlanExercises {
          id
          createdAt
          updatedAt
          sets
          reps
          exercise {
            id
            name
            bodyPart
            popularity
            equipmentRequired
          }
      }
    }
  }
}";

        let before = Instant::now();
        let ast = parse_query::<&str>(query).unwrap();
        let juniper_parser_time = before.elapsed();

        let before_async = Instant::now();
        async_graphql_parser::parse_query::<&str>(query).unwrap();
        let async_graphql_parser_time = before_async.elapsed();

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("/home/eerik/postgrustql/src/handle_query/four_way_join_many_fields.csv")
            .unwrap();
        if let Err(e) = writeln!(
            file,
            "{}, {}",
            juniper_parser_time.as_micros().to_string(),
            async_graphql_parser_time.as_micros().to_string(),
        ) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}
