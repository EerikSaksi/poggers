fn main() {
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls).unwrap();
    //https://stackoverflow.com/questions/1152260/how-to-list-table-foreign-keys

    let expected = "
    select
	to_json((__local_0__.\"app_user_id\"))::text as \"appUserId\",
	to_json((__local_0__.\"id\"))::text as \"id\",
	to_json((__local_0__.\"updated_at\"))::text as \"updatedAt\",
	to_json((__local_0__.\"created_at\"))::text as \"createdAt\",
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
	)::text as \"@workoutPlanDays\"
	from (
		select __local_0__.*
		from \"public\".\"workout_plan\" as __local_0__
		order by __local_0__.\"id\" ASC
	) __local_0__
    ";

    use std::time::Instant;

    let before = Instant::now();
    let query_res = client.query(expected, &[]);

    let mut s = String::new();
    if let Some(row) = query_res.unwrap().get(1) {
        s.push_str("{{");
        for col in row.columns() {
            let val: String = row.get(col.name());
            s.push_str(&format!("\t{}: {},", col.name(), val));
        }
        s.push_str("\n}}");
    }
    println!("Elapsed time: {:.2?}", before.elapsed());
}
