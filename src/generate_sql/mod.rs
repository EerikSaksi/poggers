mod component_builder;
#[cfg(test)]
#[path = "./test.rs"]
mod test;
use std::iter::Zip;
use std::slice::Iter;

use crate::build_schema::{GraphQLEdgeInfo, GraphQLType, Operation};
use crate::server_side_json_builder::ColumnInfo;
use async_graphql_parser::{parse_query, types::{Selection, DocumentOperations, SelectionSet}, Positioned};
use convert_case::{Case, Casing};
use inflector::Inflector;
use petgraph::{graph::DiGraph, prelude::NodeIndex};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GraphQLSchema {
    pub g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
    pub field_to_operation: HashMap<String, Operation>,
}
#[derive(Debug)]
pub struct JsonBuilderContext {
    pub sql_query: String,
    pub table_query_infos: Vec<TableQueryInfo>,
    pub root_key_name: String,
    pub root_query_is_many: bool,
}
pub struct SqlQueryComponents {
    selections: String,
    from: String,
    filter: String,
    order_by: String,
}
pub struct TableQueryInfo {
    graphql_fields: Vec<ColumnInfo>,
    primary_key_range: Range<usize>,
}

#[allow(dead_code)]
impl GraphQLSchema {
    pub fn new(
        g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
        field_to_operation: HashMap<String, Operation>,
    ) -> GraphQLSchema {
        GraphQLSchema {
            g,
            field_to_operation,
        }
    }

    pub fn build_root(&self, query: &str) -> Result<JsonBuilderContext, String> {
        let ast;
        match parse_query::<&str>(query) {
            Ok(tree) => ast = tree,
            Err(e) => return Err(e.to_string()),
        }
        match ast.operations {
            DocumentOperations::Single(Positioned { pos: _, node }) => {
                self.visit_query(&node.selection_set)
            }
            DocumentOperations::Multiple(operation_map) => {
                self.visit_query(&operation_map.values().next().unwrap().node.selection_set)
            }
        }
    }
    fn visit_query(
        &self,
        selection_set: &Positioned<SelectionSet>,
    ) -> Result<JsonBuilderContext, String> {
        let mut sql = SqlQueryComponents {
            selections: String::new(),
            from: String::new(),
            filter: String::from(" WHERE "),
            order_by: String::new(),
        };
        let mut table_query_infos: Vec<TableQueryInfo> = vec![];

        let root_key_name: &str;
        if let Selection::Field(field) = &selection_set.node.items.get(0).unwrap().node {
            root_key_name = field.node.name.node.as_str();

            //clone operation (or if invalid throw error)
            let operation;
            match self.field_to_operation.get(root_key_name) {
                Some(op) => operation = op.clone(),
                None => return Err(format!("No operation named \"{}\"", root_key_name)),
            }

            //we want to extract include_filter (whether we should build the where a = b clause)
            //we need this as we need to know whether we are expecting arguments (and to throw an
            //error if we don't receive them)

            //extract node index sql.from the operation (this can be used to perform shared logic
            //between operations)
            let (is_many, node_index) = match operation {
                Operation::Query(is_many, node_index) => (is_many, node_index),
                Operation::Delete(node_index) => (false, node_index),
                Operation::Update(node_index) => (false, node_index),
                Operation::Insert(node_index) => (false, node_index),
            };
            if let Err(e) = &self.build_selection(
                &mut sql,
                &mut table_query_infos,
                selection_set.node.items.get(0).unwrap(),
                node_index,
                0,
                0,
            ) {
                return Err(e.to_string());
            }

            if !is_many {
                match &selection_set.node.items.get(0).unwrap().node {
                    Selection::Field(Positioned { pos: _, node }) => {
                        //if the value of the first (or only) primary key was provided, we can assume
                        //that we can build a where clause for all (or one) primay keys
                        for pk in &self.g[node_index].primary_keys {
                            match node.get_argument(&pk.to_camel_case()) {
                                Some(pk_val) => sql
                                    .filter
                                    .push_str(&format!("__table_0__.{} = {} and ", pk, pk_val)),
                                None => return Err(format!("Expected input field {}", pk)),
                            }
                        }
                        sql.filter.drain(sql.filter.len() - 4..sql.filter.len());
                    }
                    _ => panic!("Didn't get Selection::Field"),
                }
            }

            //remove trailing comma sql.from select
            sql.selections
                .drain(sql.selections.len() - 2..sql.selections.len());

            let sql_query;
            match operation {
                Operation::Query(root_query_is_many, _) => {
                    match component_builder::select(
                        &mut sql,
                        &self.g[node_index].table_name,
                        root_query_is_many,
                        selection_set,
                        &self.g[node_index].field_to_types,
                    ) {
                        Ok(val) => sql_query = val,
                        Err(e) => return Err(e),
                    }
                }
                Operation::Delete(_) => {
                    sql_query = component_builder::delete(&mut sql, &self.g[node_index].table_name);
                }
                Operation::Update(_) => {
                    match component_builder::update(
                        &mut sql,
                        &self.g[node_index].table_name,
                        selection_set,
                        &self.g[node_index].field_to_types,
                    ) {
                        Ok(val) => sql_query = val,
                        Err(e) => return Err(e),
                    }
                }
                Operation::Insert(_) => {
                    match component_builder::insert(
                        &mut sql,
                        &self.g[node_index].table_name,
                        selection_set,
                        &self.g[node_index].field_to_types,
                    ) {
                        Ok(val) => sql_query = val,
                        Err(e) => return Err(e),
                    }
                }
            };

            Ok(JsonBuilderContext {
                sql_query,
                table_query_infos,
                root_key_name: root_key_name.to_owned(),
                root_query_is_many: is_many,
            })
        } else {
            panic!("First selection_set item isn't a field");
        }
    }

    fn build_selection(
        &self,
        sql: &mut SqlQueryComponents,
        table_query_infos: &mut Vec<TableQueryInfo>,
        selection: &Positioned<Selection>,
        node_index: NodeIndex<u32>,
        column_offset: usize,
        mut local_id: u8,
    ) -> Result<(usize, u8), String> {
        let SqlQueryComponents {
            from,
            selections,
            filter: _,
            order_by,
        } = sql;
        let mut new_col_offset = column_offset;
        if let Selection::Field(field) = &selection.node {
            //first we recursively get all queries from the children
            //this field is terminal
            let id_copy = local_id;
            let current_alias = GraphQLSchema::table_alias(local_id);
            let mut children: Vec<(&Positioned<Selection>, NodeIndex<u32>)> = vec![];

            //we need to add all primary keys of this particular table (so we know how to group
            //separate objects)
            for (i, pk) in self.g[node_index].primary_keys.iter().enumerate() {
                selections.push_str(&current_alias);
                selections.push('.');
                selections.push_str(pk);
                selections.push_str(" AS");
                selections.push_str(" __t");
                selections.push_str(&id_copy.to_string());
                selections.push_str("_pk");
                selections.push_str(&i.to_string());
                selections.push_str("__, ");
            }

            let mut encountered_join = false;
            let mut graphql_fields: Vec<ColumnInfo> = vec![];
            for selection in &field.node.selection_set.node.items {
                if let Selection::Field(child_field) = &selection.node {
                    let child_name = child_field.node.name.node.as_str();
                    match self.g[node_index].field_to_types.get(child_name) {
                        Some(column_info) => {
                            let column_name = &[&current_alias, ".", &column_info.0].concat();
                            graphql_fields
                                .push(ColumnInfo::Terminal(child_name.to_string(), column_info.1));
                            selections.push_str(column_name);
                            selections.push_str(" AS __t");
                            selections.push_str(&id_copy.to_string());
                            selections.push_str("_c");
                            selections.push_str(&(new_col_offset - column_offset).to_string());
                            selections.push_str("__, ");
                            new_col_offset += 1;
                        }
                        None => {
                            //if we have a child join then we need to order the parent by its primary
                            //key to allow us to capture all children for the parent when iterating
                            if !encountered_join {
                                encountered_join = true;
                                for pk in &self.g[node_index].primary_keys {
                                    order_by.push_str(&current_alias);
                                    order_by.push('.');
                                    order_by.push_str(pk);
                                    order_by.push_str(", ");
                                }
                            }
                            local_id += 1;
                            let child_alias = GraphQLSchema::table_alias(local_id);

                            let join_cols: Zip<Iter<String>, Iter<String>>;
                            let child_node_index: NodeIndex<u32>;
                            match self.find_edge_and_endpoints(
                                node_index,
                                child_name,
                                &mut graphql_fields,
                            ) {
                                Ok((j_c, node)) => {
                                    join_cols = j_c;
                                    child_node_index = node;
                                }
                                Err(e) => return Err(e),
                            }

                            from.push_str(" LEFT JOIN ");
                            from.push_str(&self.g[child_node_index].table_name);
                            from.push_str(" AS ");
                            from.push_str(&child_alias);
                            from.push_str(" ON ");

                            //if its not terminal, this field must be some foreign field. Search the nodes
                            //edges for the edge that corresponds to this graphql field, and whether its a
                            //one to many or many to one relation
                            //
                            for (col1, col2) in join_cols {
                                new_col_offset += 1;
                                let parent_pk = [&current_alias, ".", col1].concat();
                                from.push_str(&parent_pk);
                                from.push_str(" = ");
                                from.push_str(&child_alias);
                                from.push('.');
                                from.push_str(col2);
                                from.push_str(" AND ");
                            }
                            //remove trailing " and "
                            from.drain(from.len() - 5..from.len());
                            children.push((selection, child_node_index));
                        }
                    }
                }
            }
            table_query_infos.push(TableQueryInfo {
                graphql_fields,
                //the value at which primary keys start is the column offset before we started
                //adding any new columns (column offset was copied before we started modifiying it
                //this recursive call. The right hand is the column offset + the number of primary
                //keys that this table has.)
                primary_key_range: (column_offset
                    ..column_offset + self.g[node_index].primary_keys.len()),
            });

            for child in children {
                match self.build_selection(
                    sql,
                    table_query_infos,
                    child.0,
                    child.1,
                    new_col_offset,
                    local_id,
                ) {
                    Ok((col_offset, new_local_id)) => {
                        new_col_offset = col_offset;
                        local_id = new_local_id
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        Ok((new_col_offset, local_id))
    }

    //this method will try to identify whether it's an incoming or outgoing edge. If it's incoming
    //we need to return an iterator (incoming, outgoing) (the table corresponding to the incoming
    //will be on the left). Otherwise we return a zip iterator over (outgoing, incoming). Throws an
    //Err if not in incoming or outgoing edges (this means that the field requested does not exist)

    //The second tuple value we return is the NodeIndex corresponding to this selection. This is so that we can both
    //access the table name (self.g[node].table_name) and call build_selection with this node
    fn find_edge_and_endpoints(
        &self,
        node_index: NodeIndex<u32>,
        field_name: &str,
        graphql_fields: &mut Vec<ColumnInfo>,
    ) -> Result<(Zip<Iter<String>, Iter<String>>, NodeIndex<u32>), String> {
        let mut incoming_edges = self
            .g
            .neighbors_directed(node_index, petgraph::EdgeDirection::Incoming)
            .detach();

        while let Some(edge) = incoming_edges.next_edge(&self.g) {
            if self.g[edge].graphql_field_name.incoming == field_name {
                graphql_fields.push(ColumnInfo::Foreign(field_name.to_string()));
                let node_index = self.g.edge_endpoints(edge).unwrap().0;

                //if incoming child fields on left, not right
                return Ok((
                    self.g[edge]
                        .outgoing_node_cols
                        .iter()
                        .zip(self.g[edge].incoming_node_cols.iter()),
                    node_index,
                ));
            }
        }

        let mut outgoing_edges = self
            .g
            .neighbors_directed(node_index, petgraph::EdgeDirection::Outgoing)
            .detach();

        //check if we're referring to some parent. Opposite to the incoming edges, read the right
        //most graphql_field_name tuple value (parent field name)
        while let Some(edge) = outgoing_edges.next_edge(&self.g) {
            if self.g[edge].graphql_field_name.outgoing == field_name {
                graphql_fields.push(ColumnInfo::ForeignSingular(field_name.to_string()));
                let node_index = self.g.edge_endpoints(edge).unwrap().1;
                //if incoming child fields on right, not left
                return Ok((
                    self.g[edge]
                        .incoming_node_cols
                        .iter()
                        .zip(self.g[edge].outgoing_node_cols.iter()),
                    node_index,
                ));
            }
        }
        let gql_type = self.g[node_index].table_name.to_case(Case::UpperCamel);
        Err(format!(
            "{} does not have selection {}",
            gql_type, field_name
        ))
    }
    fn table_alias(local_id: u8) -> String {
        ["__table_", &local_id.to_string(), "__"].concat()
    }
}
