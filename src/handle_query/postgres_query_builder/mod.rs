use convert_case::{Case, Casing};

pub trait GraphQLQueryBuilder {
    fn build_terminal_field(s: &mut String, field_name: &str);
    fn build_terminal_field_join(s: &mut String, child_name: &str, local_id: u8);
    fn sql_query_header() -> String;
    fn single_query(s: &mut String, table_name: &str, id: i64);
    fn many_query(s: &mut String, table_name: &str, table_alias: &str);
    fn table_alias(id: u8) -> String;
    fn join_query_header(s: &mut String, local_id: u8, include_to_json: bool);
    fn join_query_closer(
        s: &mut String,
        local_id: u8,
        include_to_json: bool,
        table_name: &str,
        foreign_key_name: &str,
        parent_field_name: &str,
    );
    fn nested_join_header(s: &mut String, child_name: &str);
}

pub struct PostgresBuilder {}
impl GraphQLQueryBuilder for PostgresBuilder {
    fn build_terminal_field(s: &mut String, field_name: &str) {
        s.push_str("to_json((__local_0__.\"");
        s.push_str(&field_name.to_case(Case::Snake));
        s.push_str("\")) as \"");
        s.push_str(field_name);
        s.push_str("\",\n");
    }
    fn build_terminal_field_join(s: &mut String, field_name: &str, local_id: u8) {
        s.push('\'');
        s.push_str(field_name);
        s.push_str("'::text, (");
        s.push_str(&PostgresBuilder::table_alias(local_id));  
        s.push_str(".\"");
        s.push_str(&field_name.to_case(Case::Snake));
        s.push_str("\"),\n");
    }

    fn sql_query_header() -> String {
        String::from(
            "select to_json(
              json_build_array(__local_0__.\"id\")
            ) as \"__identifiers\",
        ",
        )
    }
    fn many_query(s: &mut String, table_name: &str, table_alias: &str) {
        s.push_str(" from ( select ");
        s.push_str(table_alias);
        s.push_str(".* from \"public\".\"");
        s.push_str(table_name);
        s.push_str("\" as  ");
        s.push_str(table_alias);
        s.push_str(" order by ");
        s.push_str(table_alias);
        s.push_str(".\"id\" ASC ) ");
        s.push_str(table_alias);
    }
    fn table_alias(id: u8) -> String {
        let mut local_string = String::from("__local_");
        local_string.push_str(&id.to_string());
        local_string.push_str("__");
        local_string
    }
    fn single_query(s: &mut String, table_name: &str, id: i64) {
        s.push_str(" from \"public\".\"");
        s.push_str(table_name);
        s.push_str("\" as __local_0__ where ( __local_0__.\"id\" = ");
        s.push_str(&id.to_string());
        s.push_str(" )");
    }
    fn join_query_header(s: &mut String, local_id: u8, include_to_json: bool) {
        //include_to_json is needed, as we only include the to_json in the SQL if this isnt
        //a nested join. If this is a nested join then we need to omiit to_json.
        //include_to_json is called with true from build_selection but with false if called
        //recursively from this function (as that would be a nested join)
        if include_to_json {
            s.push_str(" to_json(\n (\n")
        }
        s.push_str(
            "
                        select coalesce(
                          (
                            select json_agg(__local_",
        );

        s.push_str(&(local_id - 1).to_string());
        s.push_str(
            "__.\"object\")
                            from (
                              select json_build_object(
                                '__identifiers'::text,
                                json_build_array(",
        );
        s.push_str(&PostgresBuilder::table_alias(local_id));
        s.push_str(".\"id\"), ");
    }
    fn join_query_closer(
        s: &mut String,
        local_id: u8,
        include_to_json: bool,
        table_name: &str,
        foreign_key_name: &str,
        parent_field_name: &str,
    ) {
        //remove last two chars
        s.drain(s.len() - 2..s.len());
        s.push_str(" ) as object ");
        s.push_str("from ( select __local_");
        s.push_str(&(local_id).to_string());
        s.push_str(
            "__.*
                           from \"public\".\"",
        );

        s.push_str(table_name);
        s.push_str("\" as ");
        s.push_str(&PostgresBuilder::table_alias(local_id));
        s.push_str(
            "
                                where (__local_",
        );
        s.push_str(&(local_id).to_string());
        s.push_str("__.\"");
        s.push_str(foreign_key_name);
        s.push_str("\" = __local_");
        s.push_str(&(local_id - 2).to_string());
        s.push_str("__.\"id\") order by __local_");
        s.push_str(&(local_id).to_string());
        s.push_str(
            "__.\"id\" ASC
                              ) __local_",
        );
        s.push_str(&(local_id).to_string());
        s.push_str(
            "__
                            ) as __local_",
        );
        s.push_str(&(local_id - 1).to_string());
        s.push_str(
            "__ ),
                          '[]'::json
                        )
                    )
                ",
        );
        if include_to_json {
            s.push_str(") as \"@");
            s.push_str(parent_field_name);
            s.push_str("\",\n");
        }
    }
    fn nested_join_header(s: &mut String, child_name: &str) {
        s.push_str("\'@");
        s.push_str(child_name);
        s.push_str("'::text, (");
    }
}
