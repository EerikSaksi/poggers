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
        let mut s = SQL::sql_query_header();

        //create a __local__ string that we can use to distinguish this selection, and increment
        //the local_id to ensure that this stays as unique
        let table_alias = SQL::table_alias(self.local_id);

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
        include_to_json: bool,
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

        //the start and end of this query, as well as the local_ids are different depending
        //on if its one to many or many to one. Everything in the middle is the same so
        //these arent the same methods
        self.local_id = SQL::join_head(s, self.local_id, include_to_json, one_to_many);

        //we need a copy of this, as any further recursive calls would increment local_id
        //leading to incorrect results
        let local_id_copy = self.local_id;

        if let Selection::Field(field) = &selection.node {
            for selection in &field.node.selection_set.node.items {
                if let Selection::Field(child_field) = &selection.node {
                    let child_name = child_field.node.name.node.as_str();
                    let a = &self.g[node_index].table_name;
                    //check if the child name is a terminal field
                    if !self.g[node_index].terminal_fields.contains(child_name) {
                        //if not construct a nested join by adding the header, and passing
                        //the edges of this node to the child
                        let (edge, one_to_many) =
                            self.find_edge_and_endpoints(node_index, child_name);
                        SQL::nested_join_head(s, child_name);
                        self.build_foreign_field(
                            s,
                            selection,
                            edge,
                            one_to_many,
                            false,
                            child_name,
                        );
                    } else {
                        SQL::build_terminal_field_join(s, child_name, self.local_id);
                    }
                }
            }
        }
        SQL::join_tail(
            s,
            local_id_copy,
            include_to_json,
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
        let mut incoming_edges = self
            .g
            .neighbors_directed(node_index, petgraph::EdgeDirection::Incoming)
            .detach();

        //find child referring to this parent (doing this way first as I think child -> parent
        //joins are a bit more common)
        while let Some(edge) = incoming_edges.next_edge(&self.g) {
            let a = &self.g[edge].graphql_field_name.0;
            
            if self.g[edge].graphql_field_name.0 == field_name {
                return (edge, true);
            }
        }

        let mut outgoing_edges = self
            .g
            .neighbors_directed(node_index, petgraph::EdgeDirection::Outgoing)
            .detach();

        //try to find ourselves referring to a parent
        while let Some(edge) = outgoing_edges.next_edge(&self.g) {
            if self.g[edge].graphql_field_name.1 == field_name {
                return (edge, false);
            }
        }
        panic!("Shouldve found edge");
    }
}
