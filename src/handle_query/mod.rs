#[cfg(test)]
#[path = "./test.rs"]
mod test;
use crate::handle_query::postgres_query_builder::PostgresBuilder;
use crate::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};
use async_graphql_parser::types::{DocumentOperations, Selection, SelectionSet};
use async_graphql_parser::{parse_query, Positioned};
use async_graphql_value::Value;
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
use std::collections::HashMap;
pub mod postgres_query_builder;
pub mod join_database_json;

pub struct Poggers<SQL: postgres_query_builder::GraphQLQueryBuilder> {
    pub g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
    pub query_to_type: HashMap<String, QueryEdgeInfo>,
    pub local_id: u8,
    pub query_builder: SQL,
}

#[allow(dead_code)]
impl<SQL: postgres_query_builder::GraphQLQueryBuilder> Poggers<SQL> {
    pub fn new(
        g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
        query_to_type: HashMap<String, QueryEdgeInfo>,
    ) -> Poggers<postgres_query_builder::PostgresBuilder> {
        Poggers {
            g,
            query_to_type,
            local_id: 0,
            query_builder: PostgresBuilder {},
        }
    }

    pub fn build_root(&mut self, query: &str) -> Result<String, async_graphql_parser::Error> {
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
    fn visit_query(&mut self, selection_set: Positioned<SelectionSet>) -> String {

        //create a __local__ string that we can use to distinguish this selection
        let table_alias = SQL::table_alias(self.local_id);

        let mut s = String::new();

        if let Selection::Field(field) = &selection_set.node.items.get(0).unwrap().node {
            let node_index;
            let is_many;

            //we need to wrap this so that query_type is dropped, and copy out the is_many and
            //node_index fields to satisfy the borrow checker
            {
                let query_type = self
                    .query_to_type
                    .get(field.node.name.node.as_str())
                    .unwrap();
                is_many = query_type.is_many;
                node_index = query_type.node_index;
            }

            SQL::sql_query_header(&mut s, &self.g[node_index].primary_keys);

            self.build_selection(&mut s, selection_set.node.items.get(0).unwrap(), node_index);
            if is_many {
                SQL::many_query(&mut s, &self.g[node_index].table_name, &table_alias);
            } else {
                match &field.node.arguments.get(0).unwrap().1.node {
                    Value::Number(num) => SQL::single_query(
                        &mut s,
                        &self.g[node_index].table_name,
                        num.as_i64().unwrap(),
                    ),
                    _ => println!("Didn't get Value::Number"),
                }
            }
        } else {
            panic!("First selection_set item isn't a field");
        }
        self.local_id += 1;
        s
    }

    fn build_selection(
        &mut self,
        s: &mut String,
        selection: &Positioned<Selection>,
        node_index: NodeIndex<u32>,
    ) {
        if let Selection::Field(field) = &selection.node {
            //first we recursively get all queries from the children
            for selection in &field.node.selection_set.node.items {
                if let Selection::Field(child_field) = &selection.node {
                    //this field is terminal
                    let child_name = child_field.node.name.node.as_str();
                    if self.g[node_index].terminal_fields.contains(child_name) {
                        SQL::build_terminal_field(s, child_name);
                    } else {
                        //if its not terminal, this field must be some foreign field. Search the nodes
                        //edges for the edge that corresponds to this graphql field, and whether its a
                        //one to many or many to one relation
                        let (edge, one_to_many) =
                            self.find_edge_and_endpoints(node_index, child_name);
                        self.build_foreign_field(s, selection, edge, one_to_many, true, child_name);
                    }
                }
            }
        }
        s.drain(s.len() - 2..s.len());
    }

    fn build_foreign_field(
        &mut self,
        s: &mut String,
        selection: &Positioned<Selection>,
        edge: EdgeIndex<u32>,
        one_to_many: bool,
        is_nested_join: bool,
        json_parent_key: &str,
    ) {
        //when we have a one_to_many relationship, the node_index is stored on the right (left
        //source right destination) otherwise on the left
        let node_index = {
            let endpoints = self.g.edge_endpoints(edge).unwrap();
            if one_to_many {
                endpoints.0
            } else {
                endpoints.1
            }
        };

        //create a join head and return the new local id. The local id is only incremented by one
        //for many_to_one but by two for one_to_many
        self.local_id = SQL::join_head(s, self.local_id, is_nested_join, one_to_many);

        //we need a copy of this, as any further recursive calls would increment local_id, and we
        //don't know by how much as we dont know the depth and the nature of the joins
        let local_id_copy = self.local_id;

        if let Selection::Field(field) = &selection.node {
            for selection in &field.node.selection_set.node.items {
                if let Selection::Field(child_field) = &selection.node {
                    let child_name = child_field.node.name.node.as_str();

                    //check if the child name is a terminal field
                    if self.g[node_index].terminal_fields.contains(child_name) {
                        SQL::build_terminal_field_join(s, child_name, self.local_id);
                    } else {
                        //find the corresponding edge like in build_selection
                        let (edge, one_to_many) =
                            self.find_edge_and_endpoints(node_index, child_name);
                        SQL::nested_join_head(s, child_name);

                        //dont include to_json as we're already in a nested join
                        self.build_foreign_field(
                            s,
                            selection,
                            edge,
                            one_to_many,
                            false,
                            child_name,
                        );
                    }
                }
            }
        }
        SQL::join_tail(
            s,
            local_id_copy,
            is_nested_join,
            &self.g[node_index].table_name,
            (&self.g[edge].foreign_keys, &self.g[node_index].primary_keys),
            json_parent_key,
            one_to_many,
        );
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
}
