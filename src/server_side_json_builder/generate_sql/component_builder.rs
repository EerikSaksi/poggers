use crate::server_side_json_builder::generate_sql::SqlQueryComponents;
use async_graphql::parser::types::{Selection, SelectionSet, Value};
use async_graphql::Positioned;
use std::collections::HashMap;

pub fn query(sql: &mut SqlQueryComponents, table_name: &str, is_many: bool) -> String {
    let mut sql_query = [
        "SELECT ",
        &sql.selections,
        " from ",
        table_name,
        " AS __table_0__ ",
        &sql.from,
    ]
    .concat();
    if !is_many {
        sql_query.push_str(&sql.filter);
    }
    if !sql.order_by.is_empty() {
        sql.order_by
            .drain(sql.order_by.len() - 2..sql.order_by.len());
        sql_query.push_str(" ORDER BY ");
        sql_query.push_str(&sql.order_by);
    }
    sql_query
}

pub fn delete(sql: &SqlQueryComponents, table_name: &str) -> String {
    [
        "WITH __table_0__ AS ( DELETE FROM ",
        table_name,
        " AS __table_0__ ",
        &sql.filter,
        "RETURNING *) SELECT ",
        &sql.selections,
        " FROM __table_0__",
        &sql.from,
    ]
    .concat()
}

pub fn update(
    sql: &SqlQueryComponents,
    table_name: &str,
    selection_set: &Positioned<SelectionSet>,
    field_to_types: &HashMap<String, (String, usize)>,
) -> Result<String, String> {
    let mut sql_query = [
        "WITH __table_0__ AS ( UPDATE ",
        table_name,
        " AS __table_0__ SET ",
    ]
    .concat();
    match &selection_set.node.items.get(0).unwrap().node {
        Selection::Field(Positioned { pos: _, node }) => match node.get_argument("patch") {
            Some(patch) => match &patch.node {
                Value::Object(patch) => {
                    for arg in patch.keys() {
                        match field_to_types.get(&arg.to_string()) {
                            Some((col_name, _)) => {
                                sql_query.push_str(&format!(
                                    "{} = {},",
                                    col_name,
                                    patch.get(arg).unwrap()
                                ));
                            }
                            None => {
                                return Err(format!("Patch received unexpected argument {}", arg));
                            }
                        }
                    }
                }
                _ => return Err("Patch wasn't an object".to_string()),
            },
            None => return Err("Didn't get expected patch input".to_string()),
        },
        _ => panic!("Didn't get Selection::Field"),
    }
    //remove trailing comma
    sql_query.drain(sql_query.len() - 1..sql_query.len());
    sql_query.push_str(&sql.filter);
    sql_query.push_str(" RETURNING *) SELECT ");
    sql_query.push_str(&sql.selections);
    sql_query.push_str(" from __table_0__");
    sql_query.push_str(&sql.from);
    Ok(sql_query)
}
