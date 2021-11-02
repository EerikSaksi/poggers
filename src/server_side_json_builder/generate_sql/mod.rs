#[cfg(test)]
#[path = "./test.rs"]
mod test;
use crate::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};
use crate::server_side_json_builder::ColumnInfo;
use async_graphql_parser::types::{DocumentOperations, Selection, SelectionSet};
use async_graphql_parser::{parse_query, Positioned};
use convert_case::{Case, Casing};
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
use std::collections::HashMap;

use super::TableQueryInfo;

pub struct ServerSidePoggers {
    pub g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
    pub query_to_type: HashMap<String, QueryEdgeInfo>,
    pub local_id: u8,
    pub num_select_cols: usize,
}

#[allow(dead_code)]
impl ServerSidePoggers {
    pub fn new(
        g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
        query_to_type: HashMap<String, QueryEdgeInfo>,
    ) -> ServerSidePoggers {
        ServerSidePoggers {
            g,
            query_to_type,
            local_id: 0,
            num_select_cols: 0,
        }
    }

    pub fn build_root(
        &mut self,
        query: &str,
    ) -> Result<(String, Vec<TableQueryInfo>, String), async_graphql_parser::Error> {
        let ast = parse_query::<&str>(query)?;
        match ast.operations {
            DocumentOperations::Single(Positioned { node, pos: _ }) => {
                Ok(self.visit_query(node.selection_set))
            }
            DocumentOperations::Multiple(_) => {
                panic!("DocumentOperations::Multiple(operation)")
            }
        }
    }
    fn visit_query(
        &mut self,
        selection_set: Positioned<SelectionSet>,
    ) -> (String, Vec<TableQueryInfo>, String) {
        let mut select = String::from("SELECT ");
        let mut from = String::from(" FROM ");
        let mut order_by = String::new();
        let mut table_query_info: Vec<TableQueryInfo> = vec![];

        let root_key_name: &str;
        if let Selection::Field(field) = &selection_set.node.items.get(0).unwrap().node {
            let node_index;
            root_key_name = field.node.name.node.as_str();
            //we need to wrap this so that query_type is dropped, and copy out the is_many and
            //node_index fields to satisfy the borrow checker
            {
                let query_type = self.query_to_type.get(root_key_name).unwrap();
                node_index = query_type.node_index;
            }
            from.push_str(&self.g[node_index].table_name);
            from.push_str(" AS __table_0__");
            self.build_selection(
                &mut select,
                &mut from,
                &mut order_by,
                &mut table_query_info,
                selection_set.node.items.get(0).unwrap(),
                node_index,
                &field.node.name.node,
            );
        } else {
            panic!("First selection_set item isn't a field");
        }

        //we don't necessarily need to order (e.g if no joins) so check if there are any fields to
        //order and only then concat " ORDER BY " and the orderable column
        select.drain(select.len() - 2..select.len());

        match order_by.is_empty() {
            true => (
                [select, from].concat(),
                table_query_info,
                root_key_name.to_owned(),
            ),
            false => {
                order_by.drain(order_by.len() - 2..order_by.len());
                (
                    [&select, &from, " ORDER BY ", &order_by].concat(),
                    table_query_info,
                    root_key_name.to_owned(),
                )
            }
        }
    }

    fn build_selection(
        &mut self,
        select: &mut String,
        from: &mut String,
        order_by: &mut String,
        table_query_info: &mut Vec<TableQueryInfo>,
        selection: &Positioned<Selection>,
        node_index: NodeIndex<u32>,
        parent_key_name: &str,
    ) {
        if let Selection::Field(field) = &selection.node {
            //first we recursively get all queries from the children
            //this field is terminal
            let id_copy = self.local_id;
            let current_alias = ServerSidePoggers::table_alias(self.local_id);
            let mut children: Vec<(&Positioned<Selection>, NodeIndex<u32>, &str)> = vec![];

            //we need to add all primary keys of this particular table (so we know how to group
            //separate objects)
            for (i, pk) in self.g[node_index].primary_keys.iter().enumerate() {
                select.push_str(&current_alias);
                select.push('.');
                select.push_str(&pk);
                select.push_str(" AS ");
                select.push_str(" __t");
                select.push_str(&id_copy.to_string());
                select.push_str("_pk");
                select.push_str(&i.to_string());
                select.push_str("__, ");
            }

            let mut encountered_join = false;
            let mut graphql_fields: Vec<ColumnInfo> = vec![];
            let column_offset = self.num_select_cols;

            for selection in &field.node.selection_set.node.items {
                if let Selection::Field(child_field) = &selection.node {
                    let child_name = child_field.node.name.node.as_str();
                    let column_name =
                        &[&current_alias, ".", &child_name.to_case(Case::Snake)].concat();

                    match self.g[node_index].field_to_types.get(child_name) {
                        Some(closure_index) => {
                            graphql_fields
                                .push(ColumnInfo::Terminal(child_name.to_string(), *closure_index));
                            select.push_str(column_name);
                            select.push_str(" as __t");
                            select.push_str(&id_copy.to_string());
                            select.push_str("_c");
                            select.push_str(&(self.num_select_cols - column_offset).to_string());
                            select.push_str("__, ");
                            self.num_select_cols += 1;
                        }
                        None => {
                            //if we have a child join then we need to order the parent by its primary
                            //key to allow us to capture all children for the parent when iterating
                            if !encountered_join {
                                encountered_join = true;
                                for pk in &self.g[node_index].primary_keys {
                                    order_by.push_str(&current_alias);
                                    order_by.push('.');
                                    order_by.push_str(&pk);
                                    order_by.push_str(", ");
                                }
                            }
                            let child_alias = ServerSidePoggers::table_alias(self.local_id + 1);
                            self.local_id += 1;
                            let (edge, _) = self.find_edge_and_endpoints(node_index, child_name);
                            let child_node_index = self.g.edge_endpoints(edge).unwrap().0;

                            from.push_str(" LEFT JOIN ");
                            from.push_str(&self.g[child_node_index].table_name);
                            from.push_str(" AS ");
                            from.push_str(&child_alias);
                            from.push_str(" ON ");

                            //if its not terminal, this field must be some foreign field. Search the nodes
                            //edges for the edge that corresponds to this graphql field, and whether its a
                            //one to many or many to one relation
                            for (pk, fk) in self.g[node_index]
                                .primary_keys
                                .iter()
                                .zip(&self.g[edge].foreign_keys)
                            {
                                self.num_select_cols += 1;
                                let parent_pk = [&current_alias, ".", pk].concat();
                                from.push_str(&parent_pk);
                                from.push_str(" = ");
                                from.push_str(&child_alias);
                                from.push('.');
                                from.push_str(fk);
                            }

                            graphql_fields.push(ColumnInfo::Foreign(child_name.to_string()));
                            children.push((selection, child_node_index, child_name));
                        }
                    }
                }
            }
            table_query_info.push(TableQueryInfo {
                graphql_fields,
                column_offset,
            });

            for child in children {
                self.build_selection(
                    select,
                    from,
                    order_by,
                    table_query_info,
                    child.0,
                    child.1,
                    child.2,
                );
            }
        }
    }
    fn find_edge_and_endpoints(
        &self,
        node_index: NodeIndex<u32>,
        field_name: &str,
    ) -> (EdgeIndex<u32>, bool) {
        //given a node, find an edge for which the edges weight's graphql_field_name = field_name.
        //Case 1: This edge was found in incoming edges. This means that a child table is refering
        //to this parent_table in the database. In that case this relation is one_to_many

        //Case 2: This edge was found in outgoing edges. This means that the current graphql
        //type is a child table referring to a parent, making this a many to one relationship.

        //Case 3: No edge found, unrecoverable error. This is only possible if the query
        //validation wasn't done properly or we're accidentally searching for a terminal field
        //as a foreign one. Either problem in schema representation or programming error

        let mut incoming_edges = self
            .g
            .neighbors_directed(node_index, petgraph::EdgeDirection::Incoming)
            .detach();

        //check if there is some child table referring to us. The left hand graphql_field_name is
        //the child tables field name, and the right hand is the parent tables field name. Since
        //we're looking for a child field we're accessing the left most value
        while let Some(edge) = incoming_edges.next_edge(&self.g) {
            if self.g[edge].graphql_field_name.0 == field_name {
                return (edge, true);
            }
        }

        let mut outgoing_edges = self
            .g
            .neighbors_directed(node_index, petgraph::EdgeDirection::Outgoing)
            .detach();

        //check if we're referring to some parent. Opposite to the incoming edges, read the right
        //most graphql_field_name tuple value (parent field name)
        while let Some(edge) = outgoing_edges.next_edge(&self.g) {
            if self.g[edge].graphql_field_name.1 == field_name {
                return (edge, false);
            }
        }
        panic!("Shouldve found edge {}", field_name);
    }
    fn table_alias(local_id: u8) -> String {
        ["__table_", &local_id.to_string(), "__"].concat()
    }
}
