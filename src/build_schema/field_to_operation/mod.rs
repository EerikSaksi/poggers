use crate::build_schema::postgraphile_introspection::ClassData;
use crate::build_schema::Operation;
use convert_case::{Case, Casing};
use inflector::Inflector;
use petgraph::prelude::NodeIndex;
use std::collections::HashMap;

///Given
pub fn build_mutation(
    node: NodeIndex<u32>,
    field_to_operation: &mut HashMap<String, Operation>,
    class: &ClassData,
) {
    if class.is_selectable {
        field_to_operation.insert(
            class.name.to_case(Case::Camel),
            Operation::Query(false, node),
        );
        field_to_operation.insert(
            class.name.to_case(Case::Camel).to_plural(),
            Operation::Query(true, node),
        );
    }
    if class.is_deletable {
        field_to_operation.insert(
            ["delete", &class.name.to_case(Case::UpperCamel)].concat(),
            Operation::Delete(node),
        );
    }
    if class.is_insertable {
        field_to_operation.insert(
            ["update", &class.name.to_case(Case::UpperCamel)].concat(),
            Operation::Update(node),
        );
    }
    if class.is_deletable {
        field_to_operation.insert(
            ["insert", &class.name.to_case(Case::UpperCamel)].concat(),
            Operation::Insert(node),
        );
    }
}
