pub trait GraphQLQueryBuilder {
    fn build_query_root() -> String;
}
pub struct PostgresBuilder {}

impl GraphQLQueryBuilder for PostgresBuilder {
    fn build_query_root() -> String {}
}
