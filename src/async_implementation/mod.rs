use async_graphql::*;
use postgres::{Client, NoTls, Row};
use std::panic::catch_unwind;

#[derive(SimpleObject)]
struct WorkoutDay {
    order: u32,
}

#[derive(SimpleObject)]
struct Workout {
    name: String,
    workout_days: Vec<WorkoutDay>,
}

struct Query;

#[Object]
impl Query {
    async fn workout(&self, ctx: &Context<'_>) -> Result<Workout> {
        let mut to_return = String::from(
            "select to_json(
              json_build_array(__local_0__.\"id\")
            ) as \"__identifiers\"",
        );
        let look_ahead = ctx.look_ahead();
        if look_ahead.field("name").exists() {
            to_return.push_str("to_json((__local_0__.\"name\")) as \"name\"");
        }
        to_return.push_str(
            "from (
              select __local_0__.*
              from \"public\".\"exercise\" as __local_0__
              order by __local_0__.\"id\" ASC
            ) __local_0__",
        );
        Err(Error {
            message: to_return,
            extensions: None,
        })
    }
}

pub async fn execute_add() -> String {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let res = schema.execute("{ workout {name} }").await;
    serde_json::to_string(&res).unwrap()
}
