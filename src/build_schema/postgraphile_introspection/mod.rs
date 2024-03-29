pub mod query;
pub use query::{introspection_query_data, IntrospectionOutput};
use serde::Deserialize;
use serde_json::{from_value, Value};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ClassData {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub class_kind: String,
    pub namespace_id: String,
    pub namespace_name: String,
    pub type_id: String,
    pub is_selectable: bool,
    pub is_insertable: bool,
    pub is_updatable: bool,
    pub is_deletable: bool,
    pub acl_selectable: bool,
    pub acl_insertable: bool,
    pub acl_updatable: bool,
    pub acl_deletable: bool,
    //namespace: PgNamespace,
    //type: PgType,
    //tags: SmartTags,
    //attributes: Array<PgAttribute>,
    //constraints: Array<PgConstraint>,
    //foreign_constraints: Array<PgConstraint>,
    //primary_key_constraint: Option<PgConstraint>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AttributeData {
    pub class_id: String,
    pub num: i32,
    pub name: String,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub type_id: String,
    pub is_not_null: bool,
    pub has_default: bool,
    pub acl_selectable: bool,
    pub acl_insertable: bool,
    pub acl_updatable: bool,
    pub is_indexed: Option<bool>,
    pub is_unique: Option<bool>,
    pub column_level_select_grant: bool,
    //identity: "" | "a" | "d",
    //class: ClassData,
    //type: PgType,
    //namespace: PgNamespace,
    //tags: SmartTags,
    //type_modifier: i32,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ConstraintData {
    pub id: String,
    pub name: String,
    pub class_id: String,
    pub foreign_class_id: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub key_attribute_nums: Vec<i32>,
    pub foreign_key_attribute_nums: Vec<i32>,
    pub is_indexed: Option<bool>,
    //class: PgClass,
    //foreign_class: Option<PgClass>,
    //key_attributes: Vec<AttributeData>,
    //foreign_key_attributes: Vec<AttributeData>,
    //namespace: PgNamespace,
    //tags: SmartTags,
    //is_fake: bool,
    //type: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct TypeData {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub namespace_id: String,
    pub namespace_name: String,
    pub r#type: String,
    pub category: String,
    pub domain_is_not_null: bool,
    pub array_item_type_id: Option<String>,
    pub type_length: Option<i32>,
    pub is_pg_array: bool,
    pub class_id: Option<String>,
    pub domain_base_type_id: Option<String>,
    pub domain_type_modifier: Option<i32>,
    pub domain_has_default: bool,
    pub enum_variants: Option<Vec<String>>,
    pub range_sub_type_id: Option<String>,
    //tags: SmartTags,
    //array_item_type: Option<PgType>,
    //array_type: Option<PgType>,
    //class: Option<PgClass>,
    //domain_base_type: Option<PgType>,
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
    use super::*;
    use crate::build_schema::get_schema_and_client;
    use query::introspection_query_data;
    use std::collections::HashSet;

    #[actix_rt::test]
    async fn test_attributes_present() {
        let (_, client) = get_schema_and_client().await;
        let IntrospectionOutput {
            type_map: _,
            class_map,
            attribute_map,
            constraint_map: _,
        } = introspection_query_data(&client).await;
        let post_class = class_map
            .values()
            .find(|class| class.name == "post")
            .unwrap();
        assert_eq!(
            attribute_map
                .values()
                .filter(|att| att.class_id == post_class.id)
                .count(),
            21
        )
    }

    //#[test]
    //fn types_present() {
    //    let type_map = introspection_query_data().type_map;
    //}
    #[actix_rt::test]
    async fn constraints_present() {
        let (_, client) = get_schema_and_client().await;
        let IntrospectionOutput {
            type_map: _,
            class_map,
            attribute_map: _,
            constraint_map,
        } = introspection_query_data(&client).await;

        //get the comment classes id
        let comment_id = &class_map
            .values()
            .find(|class| class.name == "comment")
            .unwrap()
            .id;

        //using the id get all the constraints of the commetn class
        let comment_constraints = constraint_map
            .values()
            .filter(|constraint| &constraint.class_id == comment_id)
            .map(|constraint| constraint.name.to_string())
            .collect::<HashSet<String>>();
        assert_eq!(
            HashSet::from(
                ["fk_comment_site_user", "comments_pkey", "fk_comment_post"].map(|s| s.to_string())
            ),
            comment_constraints
        );
    }

    #[actix_rt::test]
    async fn all_tables_present() {
        let (_, client) = get_schema_and_client().await;
        let class_map = introspection_query_data(&client).await.class_map;
        let expected_names = [
            "postlink",
            "comment",
            "post",
            "site_user",
            "mutation_test_child",
            "compound_child_table",
            "badge",
            "compound_table",
            "mutation_test",
            "foreign_primary_key",
            "posthistory",
            "tag",
            "vote",
        ];
        for expected_name in expected_names {
            assert!(
                class_map.values().any(|table| table.name == expected_name),
                "{} not found",
                expected_name
            );
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
