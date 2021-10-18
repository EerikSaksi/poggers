use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use postgres::{Client, NoTls};
use std::time::Instant;
pub fn setup_fixtures() -> Result<(), postgres::Error> {
    let mut client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls)?;

    client.query("delete from workout_plan", &[])?;
    client.query("delete from app_user", &[])?;
    client.query(
        "insert into app_user(id, username, password) values(1, 'Bobby tables', '123')",
        &[],
    )?;

    let mut day_id = 1;
    for workout_plan_id in 0..1000 {
        println!("{}", workout_plan_id);
        client.query(
            "insert into workout_plan(id, app_user_id, name) overriding system value values($1, 1, 'Doesnt matter')",
            &[&workout_plan_id],
        )?;
        for _ in 0..1000 {
            client.query("insert into workout_plan_day(name, id, workout_plan_id) overriding system value values('Also doesnt matter', $1, $2)", &[&day_id, &workout_plan_id])?;
            day_id += 1;
        }
    }
    Ok(())
}
pub fn build_json_server_side() -> Result<String, postgres::Error> {
    let before = Instant::now();
    let mut client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls)?;
    let res = client.query("select workout_plan.id as wp_id, workout_plan.name, workout_plan.app_user_id, workout_plan.created_at, workout_plan.updated_at, workout_plan_day.id as wpd_id, workout_plan_id from workout_plan join workout_plan_day on workout_plan_day.workout_plan_id = workout_plan.id group by wp_id, wpd_id", &[])?;

    let mut to_return = String::from("workoutPlans: [\n\t");
    let mut last_id = -1;
    for row in res {
        let new_id: i32 = row.get(0);
        if new_id != last_id {
            to_return.push_str("  ]\n},\n{\n  \"id\": ");
            to_return.push_str(&new_id.to_string());

            to_return.push_str("  \"name\": ");
            let name: &str = row.get(1);
            to_return.push_str(&name.to_string());

            to_return.push_str(",\n  \"appUserId\": ");
            let col: i32 = row.get(2);
            to_return.push_str(&col.to_string());

            to_return.push_str(",\n  \"createdAt\": ");
            let col: chrono::DateTime<Utc> = row.get(3);
            to_return.push_str(&col.to_string());

            to_return.push_str(",\n  \"updatedAt\": ");
            let col: chrono::DateTime<Utc> = row.get(4);
            to_return.push_str(&col.to_string());

            to_return.push_str(",\n  \"workoutPlanDays\": [\n");
            last_id = new_id;
        }

        let day_id: i32 = row.get(5);
        if day_id % 1000 <= 2 {
            to_return.push_str("    {\n      \"id\":");
            to_return.push_str(&day_id.to_string());
            let workout_plan_id: i32 = row.get(6);
            to_return.push_str(",\n      workoutPlanId: ");
            to_return.push_str(&workout_plan_id.to_string());
            to_return.push_str("\n    },\n");
        }
    }
    println!("build_json_server_side: {:.2?}", before.elapsed());
    Ok(to_return)
}

pub fn postgraphile_query() -> Result<(), postgres::Error> {
    let before = Instant::now();
    let mut client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls)?;
    let query = "
        select to_json(
          json_build_array(__local_0__.\"id\")
        ) as \"__identifiers\",
        to_json((__local_0__.\"id\")) as \"id\",
        to_json((__local_0__.\"name\")) as \"name\",
        to_json((__local_0__.\"app_user_id\")) as \"appUserId\",
        to_json((__local_0__.\"created_at\")) as \"createdAt\",
        to_json((__local_0__.\"updated_at\")) as \"updatedAt\",
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
                    (__local_2__.\"workout_plan_id\")
                  ) as object
                  from (
                    select __local_2__.*
                    from \"public\".\"workout_plan_day\" as __local_2__
                    where (__local_2__.\"workout_plan_id\" = __local_0__.\"id\") and (TRUE) and (TRUE)
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
          where (TRUE) and (TRUE)
          order by __local_0__.\"id\" ASC
        ) __local_0__
    ";
    client.query(query, &[]).unwrap();
    println!("postgraphile_query {:.2?}", before.elapsed());
    Ok(())
}
