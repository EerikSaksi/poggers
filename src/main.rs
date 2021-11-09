mod build_schema;
mod handle_query;
mod server_side_json_builder;
use async_graphql::parser::parse_query;
use async_graphql::{
    check_rules,
    indexmap::IndexMap,
    registry::{MetaField, MetaType, Registry},
};
use build_schema::internal_schema_info;
use std::collections::BTreeMap;

fn main() {
    //let mut fields: IndexMap<String, MetaField> = IndexMap::new();
    //fields.insert(
    //    "text".to_string(),
    //    MetaField {
    //        name: "text".to_string(),
    //        visible: None,
    //        description: None,
    //        cache_control: Default::default(),
    //        ty: "string".to_string(),
    //        args: IndexMap::new(),
    //        external: false,
    //        requires: None,
    //        provides: None,
    //        deprecation: async_graphql::registry::Deprecation::NoDeprecated,
    //        compute_complexity: None,
    //    },
    //);
    let mut types: BTreeMap<String, MetaType> = BTreeMap::new();
    let mut fields: IndexMap<String, MetaField> = IndexMap::new();
    fields.insert(
        "eight".to_string(),
        MetaField {
            name: "eight".to_string(),
            visible: None,
            description: None,
            cache_control: Default::default(),
            ty: "Int".to_string(),
            args: IndexMap::new(),
            external: false,
            requires: None,
            provides: None,
            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
            compute_complexity: None,
        },
    );

    types.insert(
        "query".to_string(),
        MetaType::Object {
            fields,
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
    let registry = Registry {
        types,
        query_type: "query".to_string(),
        ..Default::default()
    };
    let query = "
        query noice {
            eight
        }";
    let doc = parse_query(query).unwrap();
    check_rules(&registry, &doc, None, async_graphql::ValidationMode::Strict).unwrap();
}
