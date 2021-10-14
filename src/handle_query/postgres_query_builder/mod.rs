use convert_case::{Case, Casing};
pub trait GraphQLQueryBuilder {
    fn build_terminal_field(s: &mut String, field_name: &str);
    fn sql_query_header() -> String;
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
}
