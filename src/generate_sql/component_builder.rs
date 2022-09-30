use crate::build_schema::PostgresType;
use crate::generate_sql::SqlQueryComponents;
use async_graphql_parser::{
    types::{Selection, SelectionSet},
    Positioned,
};
use async_graphql_value::{indexmap::IndexMap, Name, Value};
use std::collections::HashMap;

pub fn select(
    sql: &mut SqlQueryComponents,
    table_name: &str,
    is_many: bool,
    selection_set: &Positioned<SelectionSet>,
    field_to_types: &HashMap<String, (String, PostgresType)>,
) -> Result<String, String> {
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

    if let Selection::Field(Positioned { pos: _, node }) =
        &selection_set.node.items.get(0).unwrap().node
    {
        //check if where filter was applied
        if let Some(where_node) = node.get_argument("where") {
            //ensure its an object
            if let Value::Object(where_obj) = &where_node.node {
                sql_query.push_str(" WHERE ");

                //set where equal to values
                if let Err(e) = assign_cols_vals(&mut sql_query, where_obj, field_to_types, " AND ")
                {
                    return Err(e);
                }
            } else {
                return Err(String::from("Where was not an object"));
            }
        }
    }
    if !sql.order_by.is_empty() {
        sql.order_by
            .drain(sql.order_by.len() - 2..sql.order_by.len());
        sql_query.push_str(" ORDER BY ");
        sql_query.push_str(&sql.order_by);
    }
    Ok(sql_query)
}

pub fn delete(sql: &mut SqlQueryComponents, table_name: &str) -> String {
    let sql_query = [
        "WITH __table_0__ AS ( DELETE FROM ",
        table_name,
        " AS __table_0__",
        &sql.filter,
    ]
    .concat();
    mutation_selections(sql_query, sql).unwrap()
}

pub fn update(
    sql: &mut SqlQueryComponents,
    table_name: &str,
    selection_set: &Positioned<SelectionSet>,
    field_to_types: &HashMap<String, (String, PostgresType)>,
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
                    //set where equal to values
                    if let Err(e) = assign_cols_vals(&mut sql_query, patch, field_to_types, ",") {
                        return Err(e);
                    }
                }
                _ => return Err("Patch wasn't an object".to_string()),
            },
            None => return Err("Didn't get expected patch input".to_string()),
        },
        _ => panic!("Didn't get Selection::Field"),
    }

    sql_query.push_str(&sql.filter);
    sql_query.push_str(" RETURNING *) SELECT ");
    sql_query.push_str(&sql.selections);
    sql_query.push_str(" from __table_0__");
    sql_query.push_str(&sql.from);
    Ok(sql_query)
}

pub fn insert(
    sql: &mut SqlQueryComponents,
    table_name: &str,
    selection_set: &Positioned<SelectionSet>,
    field_to_types: &HashMap<String, (String, PostgresType)>,
) -> Result<String, String> {
    let mut sql_query = [
        "WITH __table_0__ AS ( INSERT INTO ",
        table_name,
        " AS __table_0__",
    ]
    .concat();
    match &selection_set.node.items.get(0).unwrap().node {
        Selection::Field(Positioned { pos: _, node }) => {
            let mut col_names = String::from("(");
            let mut vals = String::from(" VALUES(");
            for (new_name, new_val) in &node.arguments {
                match field_to_types.get(&new_name.to_string()) {
                    Some((col_name, _)) => {
                        col_names.push_str(&col_name.to_string());
                        vals.push_str(&value_to_string(&new_val.node));
                        col_names.push(',');
                        vals.push(',');
                    }
                    None => {
                        return Err(format!("Received unexpected argument {}", new_name));
                    }
                }
            }
            //replace trailing commas with close bracket
            col_names.pop();
            col_names.push(')');
            vals.pop();
            vals.push(')');
            sql_query.push_str(&col_names);
            sql_query.push_str(&vals);
        }
        _ => panic!("Didn't get Selection::Field"),
    }
    mutation_selections(sql_query, sql)
}

fn assign_cols_vals(
    sql_query: &mut String,
    input_fields: &IndexMap<Name, Value>,
    field_to_types: &HashMap<String, (String, PostgresType)>,
    delimiter: &str,
) -> Result<(), String> {
    for arg in input_fields.keys() {
        match field_to_types.get(&arg.to_string()) {
            Some((col_name, _)) => sql_query.push_str(
                &[
                    &col_name.to_string(),
                    "=",
                    &value_to_string(input_fields.get(arg).unwrap()),
                    delimiter,
                ]
                .concat(),
            ),
            None => {
                return Err(format!("Patch received unexpected argument {}", arg));
            }
        }
    }
    sql_query.drain(sql_query.len() - delimiter.len()..sql_query.len());
    Ok(())
}

fn mutation_selections(
    mut sql_query: String,
    sql: &mut SqlQueryComponents,
) -> Result<String, String> {
    sql_query.push_str("RETURNING *) SELECT ");
    sql_query.push_str(&sql.selections);
    sql_query.push_str(" FROM __table_0__");
    sql_query.push_str(&sql.from);
    if !sql.order_by.is_empty() {
        sql.order_by
            .drain(sql.order_by.len() - 2..sql.order_by.len());
        sql_query.push_str(" ORDER BY ");
        sql_query.push_str(&sql.order_by);
    }
    Ok(sql_query)
}

fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => ["'", &s.replace("'", "''"), "'"].concat(),
        other => other.to_string(),
    }
}
