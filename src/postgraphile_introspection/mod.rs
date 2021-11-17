mod query;
use serde::Deserialize;
use serde_json::{from_value, Value};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClassData {
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

//https://github.com/graphile/graphile-engine/blob/master/packages/graphile-build-pg/src/plugins/PgIntrospectionPlugin.ts
pub enum PostgresEntity {
    Class(ClassData),
    Attribute,
    Constraint,
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
                let to_return: ClassData = from_value(value).unwrap();
                Some(PostgresEntity::Class(to_return))
            }
            "attribute" => Some(PostgresEntity::Attribute),
            "constraint" => Some(PostgresEntity::Constraint),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use query::introspection_query_rows;
    use serde_json::Value;

    #[test]
    fn all_tables_present() {
        let rows = introspection_query_rows();
        let mut tables: Vec<ClassData> = vec![];
        for row in rows {
            let val: Value = row.get(0);
            if let Some(PostgresEntity::Class(data)) = PostgresEntity::from(val) {
                tables.push(data);
            }
        }
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
            assert!(tables.iter().any(|table| table.name == expected_name));
        }
        for table in &tables {
            println!("{}", table.namespace_id);
        }
        assert_eq!(
            expected_names.len(),
            tables.len(),
            "{:?}",
            tables
                .iter()
                .map(|d| d.name.to_owned())
                .collect::<Vec<String>>()
        );
    }
}
