use crate::build_schema::GraphQLEdgeInfo;
use crate::build_schema::GraphQLType;
use crate::build_schema::Operation;
use convert_case::{Case, Casing};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

pub fn build_mutations(
    g: &DiGraph<GraphQLType, GraphQLEdgeInfo>,
    query_to_type: &mut HashMap<String, Operation>,
) {
    for node_index in g.node_indices() {
        query_to_type.insert(
            ["delete", &g[node_index].table_name.to_case(Case::UpperCamel)].concat(),
            Operation::Delete(node_index),
        );
    }
}
