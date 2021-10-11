use build_schema::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};
use handle_query::Poggers;
use juniper_implementation::JuniperPoggers;
use petgraph::graph::DiGraph;
use std::time::Instant;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
mod build_schema;
mod handle_query;
mod juniper_implementation;
use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

fn join_many_fields() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let workout_plan_terminal_fields = HashSet::from_iter(
        [
            "id",
            "appUserId",
            "name",
            "createdAt",
            "updatedAt",
            "numFollowers",
            "requiresEquipment",
            "progressionScheme",
        ]
        .iter()
        .map(|s| s.to_string()),
    );

    let workout_plan_node_index = g.add_node(GraphQLType {
        table_name: "workout_plan".to_string(),
        terminal_fields: workout_plan_terminal_fields,
    });

    let day_terminal_fields = HashSet::from_iter(
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
        terminal_fields: day_terminal_fields,
    });

    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "workoutPlans".to_string(),
        QueryEdgeInfo {
            is_many: true,
            node_index: workout_plan_node_index,
        },
    );
    g.add_edge(
        workout_plan_node_index,
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

    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let workout_plan_terminal_fields = HashSet::from_iter(
        [
            "id",
            "appUserId",
            "name",
            "createdAt",
            "updatedAt",
            "numFollowers",
            "requiresEquipment",
            "progressionScheme",
        ]
        .iter()
        .map(|s| s.to_string()),
    );

    let workout_plan_node_index = g.add_node(GraphQLType {
        table_name: "workout_plan".to_string(),
        terminal_fields: workout_plan_terminal_fields,
    });

    let day_terminal_fields = HashSet::from_iter(
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
        terminal_fields: day_terminal_fields,
    });

    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "workoutPlans".to_string(),
        QueryEdgeInfo {
            is_many: true,
            node_index: workout_plan_node_index,
        },
    );
    g.add_edge(
        workout_plan_node_index,
        day_node_index,
        GraphQLEdgeInfo {
            is_many: true,
            foreign_key_name: "workout_plan_id".to_string(),
            graphql_field_name: "workoutPlanDays".to_string(),
        },
    );

    let mut juniper_pogg = JuniperPoggers {
        g,
        query_to_type,
        local_id: 0,
    };
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
    }
  }
}
";
    run_benchmarks(pogg, juniper_pogg, "join_many_fields.csvv", query);
}
fn join_few_fields() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let workout_plan_terminal_fields =
        HashSet::from_iter(["appUserId"].iter().map(|s| s.to_string()));

    let workout_plan_node_index = g.add_node(GraphQLType {
        table_name: "workout_plan".to_string(),
        terminal_fields: workout_plan_terminal_fields,
    });

    let day_terminal_fields = HashSet::from_iter(["workoutPlanId"].iter().map(|s| s.to_string()));

    let day_node_index = g.add_node(GraphQLType {
        table_name: "workout_plan_day".to_string(),
        terminal_fields: day_terminal_fields,
    });

    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "workoutPlans".to_string(),
        QueryEdgeInfo {
            is_many: true,
            node_index: workout_plan_node_index,
        },
    );
    g.add_edge(
        workout_plan_node_index,
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

    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let workout_plan_terminal_fields =
        HashSet::from_iter(["appUserId"].iter().map(|s| s.to_string()));

    let workout_plan_node_index = g.add_node(GraphQLType {
        table_name: "workout_plan".to_string(),
        terminal_fields: workout_plan_terminal_fields,
    });

    let day_terminal_fields = HashSet::from_iter(["workoutPlanId"].iter().map(|s| s.to_string()));

    let day_node_index = g.add_node(GraphQLType {
        table_name: "workout_plan_day".to_string(),
        terminal_fields: day_terminal_fields,
    });

    let mut query_to_type: HashMap<String, QueryEdgeInfo> = HashMap::new();
    query_to_type.insert(
        "workoutPlans".to_string(),
        QueryEdgeInfo {
            is_many: true,
            node_index: workout_plan_node_index,
        },
    );
    g.add_edge(
        workout_plan_node_index,
        day_node_index,
        GraphQLEdgeInfo {
            is_many: true,
            foreign_key_name: "workout_plan_id".to_string(),
            graphql_field_name: "workoutPlanDays".to_string(),
        },
    );

    let mut juniper_pogg = JuniperPoggers {
        g,
        query_to_type,
        local_id: 0,
    };
    let query = "query{
  workoutPlans{
    appUserId
    workoutPlanDays{
      workoutPlanId
    }
  }
}";
    run_benchmarks(pogg, juniper_pogg, "join_few_fields.csvv", query);
}

fn no_join_many_fields() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let terminal_fields = HashSet::from_iter(
        [
            "bodyPart",
            "name",
            "id ",
            "popularity",
            "isBodyweight",
            "isMachine",
            "skillLevel",
        ]
        .iter()
        .map(|s| s.to_string()),
    );

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

    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let terminal_fields = HashSet::from_iter(
        [
            "bodyPart",
            "name",
            "id ",
            "popularity",
            "isBodyweight",
            "isMachine",
            "skillLevel",
        ]
        .iter()
        .map(|s| s.to_string()),
    );

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

    let mut juniper_pogg = JuniperPoggers {
        g,
        query_to_type,
        local_id: 0,
    };
    let query = "
query {
    exercises {
        bodyPart
        name
        id 
        popularity
        isBodyweight
        isMachine
        skillLevel
    }
}
        ";
    run_benchmarks(pogg, juniper_pogg, "no_joins_many_fields.csvv", query);
}
fn no_joins_few_fields() {
    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let terminal_fields = HashSet::from_iter(["bodyPart"].iter().map(|s| s.to_string()));

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

    let mut g: DiGraph<GraphQLType, GraphQLEdgeInfo> = DiGraph::new();
    let terminal_fields = HashSet::from_iter(["bodyPart"].iter().map(|s| s.to_string()));

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

    let mut juniper_pogg = JuniperPoggers {
        g,
        query_to_type,
        local_id: 0,
    };

    let query = "
        query {
          exercises {
            bodyPart
          }
        }";
    run_benchmarks(pogg, juniper_pogg, "no_joins_few_fields.csvv", query);
}

fn run_benchmarks(
    mut pogg: Poggers,
    mut juniper_pogg: JuniperPoggers,
    filename: &str,
    query: &str,
) {
    let filepath = format!("/home/eerik/postgrustql/src/benchmarks/{}", filename);
    let mut file = File::create(&filepath).unwrap();
    file.write_all(b"juniper_time, async_graphql_time\n").unwrap();
    for i in 0..10000 {
        pogg.local_id = 0;
        juniper_pogg.local_id = 0;
        println!("{}", i);
        let before = Instant::now();
        juniper_pogg.build_root(query).unwrap();
        let juniper_time = before.elapsed().as_micros();

        let before = Instant::now();
        pogg.build_root(query).unwrap();
        let async_graphql_time = before.elapsed().as_micros();

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&filepath)
            .unwrap();
        if let Err(e) = writeln!(file, "{}, {}", juniper_time, async_graphql_time) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}
fn main() {
    no_joins_few_fields();
    no_join_many_fields();
    join_few_fields();
    join_many_fields();
}
