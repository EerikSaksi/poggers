#![feature(test)]
use build_schema::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};

mod build_schema;
mod handle_query;
mod integer_indexing;
mod juniper_implementation;
mod row_selection_techniques;

fn main() {
    row_selection_techniques::insert_rows();
}
