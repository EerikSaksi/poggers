//use async_graphql::Schema;
//use async_graphql::{Context, EmptyMutation, EmptySubscription, Object};
//use postgres::{Client, NoTls, Row};
//
//struct WorkoutDay {
//    order: u32,
//}
//
//struct Workout {
//    name: String,
//    workout_days: Vec<WorkoutDay>,
//}
//
//
//impl WorkoutDay {
//    async fn order(&self) -> u32 {
//        self.order
//    }
//}
//
//struct Query;
//
//#[Object]
//impl Query {
//    async fn workout(&self, ctx: &Context<'_>) -> Workout {
//        let mut to_return = String::from(
//            "select to_json(
//              json_build_array(__local_0__.\"id\")
//            ) as \"__identifiers\"",
//        );
//        let look_ahead = ctx.look_ahead();
//        if look_ahead.field("name").exists() {
//            to_return.push_str("to_json((__local_0__.\"name\")) as \"name\"");
//        }
//
//        to_return.push_str(
//            "from (
//              select __local_0__.*
//              from \"public\".\"exercise\" as __local_0__
//              order by __local_0__.\"id\" ASC
//            ) __local_0__",
//        );
//
//        let client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls).unwrap();
//
//        Workout {
//            name: String::from("noice"),
//            workout_days: vec![WorkoutDay{order: 0}]
//        }
//    }
//}
//
//pub async fn execute_add() -> String {
//    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
//    let res = schema.execute("{workout{name}}").await;
//    serde_json::to_string(&res).unwrap()
//}
