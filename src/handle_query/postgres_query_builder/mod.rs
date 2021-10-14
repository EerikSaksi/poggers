use convert_case::{Case, Casing};

//use a table alias wrapper to prevent errors, as we take a lot of string arguments
pub struct TableAlias(String);
pub trait GraphQLQueryBuilder {
    fn build_terminal_field(s: &mut String, field_name: &str);
    fn sql_query_header() -> String;
    fn single_query(s: &mut String, table_name: &str, id: i64);
    fn many_query(s: &mut String, table_name: &str, table_alias: TableAlias);
    fn table_alias(id: u8) -> TableAlias;
    fn join_query(s: &mut String, local_id: u8, include_to_json: bool);
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
    fn sql_query_header() -> String {
        String::from(
            "select to_json(
              json_build_array(__local_0__.\"id\")
            ) as \"__identifiers\",
        ",
        )
    }
    fn many_query(s: &mut String, table_name: &str, table_alias: TableAlias) {
        s.push_str(" from ( select ");
        s.push_str(&table_alias.0);
        s.push_str(".* from \"public\".\"");
        s.push_str(table_name);
        s.push_str("\" as  ");
        s.push_str(&table_alias.0);
        s.push_str(" order by ");
        s.push_str(&table_alias.0);
        s.push_str(".\"id\" ASC ) ");
        s.push_str(&table_alias.0);
    }
    fn table_alias(id: u8) -> TableAlias {
        let mut local_string = String::from("__local_");
        local_string.push_str(&id.to_string());
        local_string.push_str("__");
        TableAlias(local_string)
    }
    fn single_query(s: &mut String, table_name: &str, id: i64) {
        s.push_str(" from \"public\".\"");
        s.push_str(table_name);
        s.push_str("\" as __local_0__ where ( __local_0__.\"id\" = ");
        s.push_str(&id.to_string());
        s.push_str(" )");
    }
    fn join_query(s: &mut String, local_id: u8, include_to_json: bool) {

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
                                json_build_array(__local_",
        );
        s.push_str(&(local_id).to_string());
        s.push_str("__.\"id\"), ");
    }
}
