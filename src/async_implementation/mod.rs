//use async_graphql::Schema;
//use async_graphql::{Context, EmptyMutation, EmptySubscription, Object};
//
//#[derive(Debug)]
//struct WorkoutDay {
//    order: u32,
//}
//struct Workout {
//    name: String,
//    workout_days: Vec<WorkoutDay>,
//}
//
//impl WorkoutDay {
//    async fn order(&self) -> u32 {
//        self.order
//    }
//}
//
//#[Object]
//impl Workout {
//    async fn workout_days(&self) -> String {
//        format!(
//            "{:?}",
//            vec![
//                WorkoutDay { order: 0 },
//                WorkoutDay { order: 1 },
//                WorkoutDay { order: 2 },
//            ]
//        )
//    }
//}
//
//struct Query;
//
//#[Object]
//impl Query {
//    async fn workout(&self, ctx: &Context<'_>) -> Workout{
//    }
//}
//
//pub async fn execute_add() -> String {
//    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
//    let res = schema.execute("query{exercises{workoutDays}}").await;
//    serde_json::to_string(&res).unwrap()
//}
