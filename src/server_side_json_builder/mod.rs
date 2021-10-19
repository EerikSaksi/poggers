#[cfg(test)]
#[path = "./test.rs"]
mod test;
use crate::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};
use async_graphql_parser::types::{DocumentOperations, Selection, SelectionSet};
use async_graphql_parser::{parse_query, Positioned};
use convert_case::{Case, Casing};
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
use std::collections::HashMap;

pub struct ServerSidePoggers {
    pub g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
    pub query_to_type: HashMap<String, QueryEdgeInfo>,
    pub local_id: u8,
}

pub struct TableQueryInfos<'a> {
    pub fields: Vec<&'a str>,
    pub primary_keys: Vec<&'a str>,
    pub table_name: &'a str,
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
        let mut cols = String::new();
        let mut from_s = String::from(" FROM ");

        if let Selection::Field(field) = &selection_set.node.items.get(0).unwrap().node {
            let node_index;

            //we need to wrap this so that query_type is dropped, and copy out the is_many and
            //node_index fields to satisfy the borrow checker
            {
                let query_type = self
                    .query_to_type
                    .get(field.node.name.node.as_str())
                    .unwrap();
                node_index = query_type.node_index;
            }
            from_s.push_str(&self.g[node_index].table_name);
            from_s.push_str(" AS __local_0__");
            self.build_selection(
                &mut cols,
                &mut from_s,
                selection_set.node.items.get(0).unwrap(),
                node_index,
            );
            cols.drain(cols.len() - 2..cols.len());
        } else {
            panic!("First selection_set item isn't a field");
        }
        ["SELECT ", &cols, &from_s, " GROUP BY ", &cols].concat()
    }

    fn build_selection(
        &mut self,
        cols: &mut String,
        from_s: &mut String,
        selection: &Positioned<Selection>,
        node_index: NodeIndex<u32>,
    ) {
        if let Selection::Field(field) = &selection.node {
            //first we recursively get all queries from the children
            //this field is terminal
            let parent_alias = ServerSidePoggers::table_alias(self.local_id);
            let child_alias = ServerSidePoggers::table_alias(self.local_id + 1);
            self.local_id += 1;
            for selection in &field.node.selection_set.node.items {
                if let Selection::Field(child_field) = &selection.node {
                    let child_name = child_field.node.name.node.as_str();

                    if self.g[node_index].terminal_fields.contains(child_name) {
                        cols.push_str(
                            &[&parent_alias, ".", &child_name.to_case(Case::Snake), ", "].concat(),
                        );
                    } else {
                        let (edge, _) = self.find_edge_and_endpoints(node_index, child_name);
                        let child_node_index = self.g.edge_endpoints(edge).unwrap().0;

                        from_s.push_str(" JOIN ");
                        from_s.push_str(&self.g[child_node_index].table_name);
                        from_s.push_str(" AS ");
                        from_s.push_str(&child_alias);
                        from_s.push_str(" ON ");

                        //if its not terminal, this field must be some foreign field. Search the nodes
                        //edges for the edge that corresponds to this graphql field, and whether its a
                        //one to many or many to one relation
                        for (pk, fk) in self.g[node_index]
                            .primary_keys
                            .iter()
                            .zip(&self.g[edge].foreign_keys)
                        {
                            let parent_pk = [&parent_alias, ".", pk].concat();
                            from_s.push_str(&parent_pk);
                            from_s.push_str(" = ");
                            from_s.push_str(&child_alias);
                            from_s.push('.');
                            from_s.push_str(fk);
                        }
                        self.build_selection(cols, from_s, selection, child_node_index);
                    }
                }
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
        ["__local_", &local_id.to_string(), "__"].concat()
    }
}
