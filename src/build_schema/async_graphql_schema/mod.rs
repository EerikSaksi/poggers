use crate::build_schema::QueryEdgeInfo;
use async_graphql::{
    indexmap::IndexMap,
    registry::{MetaField, MetaInputValue, MetaType, Registry},
};
use std::collections::{BTreeMap, HashMap};

pub fn internal_to_async(query_to_type: &HashMap<String, QueryEdgeInfo>) -> Registry {
    let mut query_fields: IndexMap<String, MetaField> = IndexMap::new();
    let mut query_type: BTreeMap<String, MetaType> = BTreeMap::new();
    for (
        query_name,
        QueryEdgeInfo {
            is_many,
            node_index: _,
        },
    ) in query_to_type
    {
        if *is_many {
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
        } else {
            let mut args: IndexMap<&str, MetaInputValue> = IndexMap::new();
            args.insert(
                "id",
                MetaInputValue {
                    name: "id",
                    visible: None,
                    description: None,
                    ty: "Int".to_string(),
                    validator: None,
                    is_secret: false,
                    default_value: None,
                },
            );
            query_fields.insert(
                query_name.clone(),
                MetaField {
                    name: query_name.clone(),
                    visible: None,
                    description: None,
                    cache_control: Default::default(),
                    ty: "".to_string(),
                    args,
                    external: false,
                    requires: None,
                    provides: None,
                    deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                    compute_complexity: None,
                },
            );
        }
    }
    query_type.insert(
        "Int".to_string(),
        MetaType::Object {
            fields: IndexMap::new(),
            name: "Int".to_string(),
            keys: None,
            extends: false,
            visible: None,
            description: None,
            cache_control: Default::default(),
            rust_typename: "Int",
            is_subscription: false,
        },
    );
    query_type.insert(
        "Float".to_string(),
        MetaType::Object {
            fields: IndexMap::new(),
            name: "Float".to_string(),
            keys: None,
            extends: false,
            visible: None,
            description: None,
            cache_control: Default::default(),
            rust_typename: "Float",
            is_subscription: false,
        },
    );

    query_type.insert(
        "String".to_string(),
        MetaType::Object {
            fields: IndexMap::new(),
            name: "String".to_string(),
            keys: None,
            extends: false,
            visible: None,
            description: None,
            cache_control: Default::default(),
            rust_typename: "String",
            is_subscription: false,
        },
    );

    query_type.insert(
        "Boolean".to_string(),
        MetaType::Object {
            fields: IndexMap::new(),
            name: "Boolean".to_string(),
            keys: None,
            extends: false,
            visible: None,
            description: None,
            cache_control: Default::default(),
            rust_typename: "Boolean",
            is_subscription: false,
        },
    );
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
            rust_typename: "query",
            is_subscription: false,
        },
    );
    Registry {
        types: query_type,
        query_type: "query".to_string(),
        ..Default::default()
    }
}
