use crate::server_side_json_builder::ServerSidePoggers;
use async_graphql::{
    indexmap::IndexMap,
    registry::{MetaField, MetaType, Registry},
};
use std::collections::BTreeMap;

fn internal_to_async(pogg: ServerSidePoggers) -> Registry {
    let mut query_fields: IndexMap<String, MetaField> = IndexMap::new();
    let mut query_type: BTreeMap<String, MetaType> = BTreeMap::new();
    for query_name in pogg.query_to_type.keys() {
        query_fields.insert(
            query_name.clone(),
            MetaField {
                name: query_name.clone(),
                visible: None,
                description: None,
                cache_control: Default::default(),
                ty: "".to_string(),
                args: IndexMap::new(),
                external: false,
                requires: None,
                provides: None,
                deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                compute_complexity: None,
            },
        );
    }
    query_type.insert(
        "query".to_string(),
        MetaType::Object {
            fields: query_fields,
            name: "query".to_string(),
            keys: None,
            extends: false,
            visible: None,
            description: None,
            cache_control: Default::default(),
            rust_typename: "Query",
            is_subscription: false,
        },
    );
    Registry {
        types: query_type,
        query_type: "query".to_string(),
        ..Default::default()
    }
}
