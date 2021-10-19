use crate::server_side_json_builder::handle_query::TableQueryInfos;
use chrono::Utc;
use postgres::{Client, NoTls};
use std::time::Instant;
mod handle_query;

fn build_query_from_ranges(table_infos: Vec<TableQueryInfos>) -> String {
    let mut selects = String::from("SELECT ");
    let mut joins = String::from("FROM ");
    let mut groups = String::from("GROUP BY ");
    let prev_primary_keys: (&str, Option<Vec<String>>) = None;
    for (
        i,
        TableQueryInfos {
            fields,
            primary_keys,
            table_name,
        },
    ) in table_infos.iter().enumerate()
    {
        for field in fields {
            selects.push_str("__");
            selects.push_str(i.to_string());
            selects.push('.');
            selects.push_str(field);
            selects.push_str(", ");
        }
        selects.drain(selects.len() - 2..selects.len());
        match prev_primary_keys {
            Some((prev_table_name, pks)) => {
                &joins.push_str([
                                "JOIN ", &table_name, " ON " , prev_table_name, ".", pks.get(0), 
                ].concat());
            }
            None => panic!(),
        }
    }
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
        to_return.push_str("    {\n      \"id\":");
        to_return.push_str(&day_id.to_string());
        let workout_plan_id: i32 = row.get(6);
        to_return.push_str(",\n      workoutPlanId: ");
        to_return.push_str(&workout_plan_id.to_string());
        to_return.push_str("\n    },\n");
    }
    println!("build_json_server_side: {:.2?}", before.elapsed());
    Ok(to_return)
}
#[test]
fn basic_one_way_join() {
    let query_column_ranges: Vec<TableQueryInfos> = vec![
        TableQueryInfos {
            fields: vec!["parent_one, parent_two, parent_three"],
            primary_keys: vec!["id"],
            table_name: "parent_table",
        },
        TableQueryInfos {
            fields: vec!["child_one, child_two, child_three"],
            primary_keys: vec!["id"],
            table_name: "child_table",
        },
    ];
    build_query_from_ranges(query_column_ranges);
}
