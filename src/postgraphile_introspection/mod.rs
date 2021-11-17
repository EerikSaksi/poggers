pub mod query;
use serde::Deserialize;
use serde_json::{from_value, Value};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ClassData {
    id: String,
    name: String,
    comment: Option<String>,
    description: Option<String>,
    class_kind: String,
    namespace_id: String,
    namespace_name: String,
    type_id: String,
    is_selectable: bool,
    is_insertable: bool,
    is_updatable: bool,
    is_deletable: bool,
    //namespace: PgNamespace,
    //type: PgType,
    //tags: SmartTags,
    //attributes: Array<PgAttribute>,
    //constraints: Array<PgConstraint>,
    //foreign_constraints: Array<PgConstraint>,
    //primary_key_constraint: Option<PgConstraint>,
    acl_selectable: bool,
    acl_insertable: bool,
    acl_updatable: bool,
    acl_deletable: bool,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AttributeData {
    class_id: String,
    num: i32,
    name: String,
    comment: Option<String>,
    description: Option<String>,
    type_id: String,
    //type_modifier: i32,
    is_not_null: bool,
    has_default: bool,
    //identity: "" | "a" | "d",
    //class: ClassData,
    //type: PgType,
    //namespace: PgNamespace,
    //tags: SmartTags,
    acl_selectable: bool,
    acl_insertable: bool,
    acl_updatable: bool,
    is_indexed: Option<bool>,
    is_unique: Option<bool>,
    column_level_select_grant: bool,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ConstraintData {
    id: String,
    name: String,
    //type: String,
    class_id: String,
    //class: PgClass,
    foreign_class_id: Option<String>,
    //foreign_class: Option<PgClass>,
    comment: Option<String>,
    description: Option<String>,
    key_attribute_nums: Vec<i32>,
    //key_attributes: Vec<AttributeData>,
    foreign_key_attribute_nums: Vec<i32>,
    //foreign_key_attributes: Vec<AttributeData>,
    //namespace: PgNamespace,
    is_indexed: Option<bool>,
    //tags: SmartTags,
    //is_fake: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct TypeData {
    id: String,
    name: String,
    comment: Option<String>,
    description: Option<String>,
    namespace_id: String,
    namespace_name: String,
    r#type: String,
    category: String,
    domain_is_not_null: bool,
    array_item_type_id: Option<String>,
    //array_item_type: Option<PgType>,
    //array_type: Option<PgType>,
    type_length: Option<i32>,
    is_pg_array: bool,
    class_id: Option<String>,
    //class: Option<PgClass>,
    domain_base_type_id: Option<String>,
    //domain_base_type: Option<PgType>,
    domain_type_modifier: Option<i32>,
    domain_has_default: bool,
    enum_variants: Option<Vec<String>>,
    range_sub_type_id: Option<String>,
    //tags: SmartTags,
}

//https://github.com/graphile/graphile-engine/blob/master/packages/graphile-build-pg/src/plugins/PgIntrospectionPlugin.ts
#[allow(dead_code)]
pub enum PostgresEntity {
    Class(ClassData),
    Attribute(AttributeData),
    Constraint(ConstraintData),
    Type(TypeData),
}

impl PostgresEntity {
    //export enum PgEntityKind {
    //  NAMESPACE = "namespace",
    //  PROCEDURE = "procedure",
    //  CLASS = "class",
    //  TYPE = "type",
    //  ATTRIBUTE = "attribute",
    //  CONSTRAINT = "constraint",
    //  EXTENSION = "extension",
    //  INDEX = "index",
    //}
    pub fn from(value: Value) -> Option<Self> {
        let obj = value.as_object().unwrap();
        let kind = obj.get("kind").unwrap().as_str().unwrap();
        match kind {
            "class" => {
                let value_as_data: ClassData = from_value(value).unwrap();
                Some(PostgresEntity::Class(value_as_data))
            }
            "attribute" => {
                let value_as_data: AttributeData = from_value(value).unwrap();
                Some(PostgresEntity::Attribute(value_as_data))
            }
            "constraint" => {
                let value_as_data: ConstraintData = from_value(value).unwrap();
                Some(PostgresEntity::Constraint(value_as_data))
            }
            "type" => {
                let value_as_data: TypeData = from_value(value).unwrap();
                Some(PostgresEntity::Type(value_as_data))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::postgraphile_introspection::query::IntrospectionOutput;

    use super::*;
    use query::introspection_query_data;

    #[test]
    fn test_attributes_present() {
        let IntrospectionOutput {
            type_map: _,
            class_map,
            attribute_vec,
            constraint_map,
        } = introspection_query_data();
        let post_class = class_map
            .values()
            .find(|class| class.name == "post")
            .unwrap();
        assert_eq!(
            attribute_vec
                .iter()
                .filter(|att| att.class_id == post_class.id)
                .count(),
            21
        )
    }

    #[test]
    fn types_present() {
        let type_map = introspection_query_data().type_map;
        type_map.values().for_each(|t| println!("{}", t.name));
        panic!();
    }
    #[test]
    fn constraints_present() {
        let IntrospectionOutput {
            type_map: _,
            class_map,
            attribute_vec: _,
            constraint_map,
        } = introspection_query_data();
        let comment_class = class_map
            .values()
            .find(|class| class.name == "comment")
            .unwrap();
        let count = constraint_map
            .values()
            .filter(|att| att.class_id == comment_class.id)
            .fold(0, |count, con| {
                println!("{}", con.name);
                assert!(["post_id_fkey", "site_user_fkey", "comments_pkey",]
                    .iter()
                    .any(|expected| *expected == con.name));
                count + 1
            });
        assert_eq!(count, 3);
    }

    #[test]
    fn all_tables_present() {
        let class_map = introspection_query_data().class_map;
        let expected_names = [
            "badge",
            "child_table",
            "comment",
            "foreign_primary_key",
            "mutation_test",
            "mutation_test_child",
            "parent_table",
            "post",
            "posthistory",
            "site_user",
            "tag",
            "vote",
        ];
        for expected_name in expected_names {
            assert!(class_map.values().any(|table| table.name == expected_name));
        }
        assert_eq!(
            expected_names.len(),
            class_map.len(),
            "{:?}",
            class_map
                .values()
                .map(|d| d.name.to_owned())
                .collect::<Vec<String>>()
        );
    }
}
