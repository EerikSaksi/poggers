#[cfg(test)]
#[path = "./test.rs"]
mod test;
use crate::internal_schema_info::{GraphQLEdgeInfo, GraphQLType, QueryEdgeInfo};
use async_graphql_parser::types::{
    DocumentOperations, Selection, SelectionSet
};
use async_graphql_parser::{parse_query, Positioned};
use async_graphql_value::Value;
use convert_case::{Case, Casing};
use petgraph::graph::DiGraph;
use petgraph::prelude::NodeIndex;
use std::collections::HashMap;

pub struct Poggers {
    g: DiGraph<GraphQLType, GraphQLEdgeInfo>,
    query_to_type: HashMap<String, QueryEdgeInfo>,
    local_id: u8,
}

#[allow(dead_code)]
impl<'b> Poggers {
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
    fn visit_query(
        &mut self,
        selection_set: Positioned<SelectionSet>,
    ) -> String {
        let mut query_string = String::from(
            "select to_json(
              json_build_array(__local_0__.\"id\")
            ) as \"__identifiers\",
        ",
        );

        //create a __local__ string that we can use to distinguish this selection, and increment
        //the local_id to ensure that this stays as unique
        let mut local_string = String::from("__local_");
        local_string.push_str(&self.local_id.to_string());
        local_string.push_str("__");

        if let Selection::Field(field) = &selection_set.node.items.get(0).unwrap().node {
            let query_type = self
                .query_to_type
                .get(field.node.name.node.as_str())
                .unwrap();
            query_string.push_str(&self.build_selection(
                selection_set.node.items.get(0).unwrap(),
                query_type.node_index,
            ));
            if query_type.is_many {
                query_string.push_str(" from ( select ");
                query_string.push_str(&local_string);
                query_string.push_str(".* from \"public\".\"");
                query_string.push_str(&self.g[query_type.node_index].table_name);
                query_string.push_str("\" as  ");
                query_string.push_str(&local_string);
                query_string.push_str(" order by ");
                query_string.push_str(&local_string);
                query_string.push_str(".\"id\" ASC ) ");
                query_string.push_str(&local_string);
            } else {
                query_string.push_str(" from \"public\".\"");
                query_string.push_str(&self.g[query_type.node_index].table_name);
                query_string.push_str("\" as __local_0__ where ( __local_0__.\"id\" = ");
                match &field.node.arguments.get(0).unwrap().1.node {
                    Value::Number(num) => {
                        query_string.push_str(&num.to_string());
                        query_string.push_str(" )");
                    },
                    _ => println!("Didn't get Value::Number"),
                }
            }
        } else {
            panic!("First selection_set item isn't a field");
        }
        self.local_id += 1;
        query_string
    }

    fn build_selection(
        &self,
        selection: &Positioned<Selection>,
        node_index: NodeIndex<u32>,
    ) -> String {
        match &selection.node {
            Selection::Field(field) => {
                let gql_type = &self.g[node_index];
                //first we recursively get all queries from the children

                let mut to_return = String::new();
                for selection in &field.node.selection_set.node.items {
                    match &selection.node {
                        Selection::Field(child_field) => {
                            //this field is terminal
                            let child_name = child_field.node.name.node.as_str();
                            if gql_type.terminal_fields.contains(child_name) {
                                Poggers::build_terminal_field(&mut to_return, child_name);
                            } else {
                                let mut edges = self
                                    .g
                                    .neighbors_directed(
                                        node_index,
                                        petgraph::EdgeDirection::Outgoing,
                                    )
                                    .detach();
                                while let Some(edge) = edges.next_edge(&self.g) {
                                    //found the edge which corresponds to this field
                                    if self.g[edge].graphql_field_name == child_name {
                                        let endpoints = self.g.edge_endpoints(edge);
                                        match endpoints {
                                            Some(endpoints) => {
                                                to_return.push_str(&self.build_foreign_field(
                                                    selection,
                                                    endpoints,
                                                    &self.g[edge].foreign_key_name,
                                                    child_name,
                                                ));
                                                break;
                                            }
                                            None => panic!("No endpoints found"),
                                        }
                                    }
                                }
                            }
                        }
                        _ => panic!("Non field selection"),
                    }
                }
                to_return.pop();
                to_return
            }
            Selection::FragmentSpread(_) => String::from("FragmentSpread not implemented"),
            Selection::InlineFragment(_) => String::from("InlineFragment not implemented"),
        }
    }
    fn build_terminal_field(to_return: &mut String, field_name: &str) {
        to_return.push_str("to_json((__local_0__.\"");
        to_return.push_str(&field_name.to_case(Case::Snake));
        to_return.push_str("\")) as \"");
        to_return.push_str(field_name);
        to_return.push_str("\",");
    }

    fn build_foreign_field<'a>(
        &self,
        selection: &Positioned<Selection>,
        endpoints: (NodeIndex<u32>, NodeIndex<u32>),
        foreign_key_name: &str,
        parent_field_name: &str,
    ) -> String {
        let mut to_return = String::from(
            "\nto_json(
                      (
                        select coalesce(
                          (
                            select json_agg(__local_",
        );

        to_return.push_str(&(self.local_id + 1).to_string());
        to_return.push_str(
            "__.\"object\")
                            from (
                              select json_build_object(
                                '__identifiers'::text,
                                json_build_array(__local_",
        );
        to_return.push_str(&(self.local_id + 2).to_string());
        to_return.push_str("__.\"id\"), ");

        if let Selection::Field(field) = &selection.node {
            for selection in &field.node.selection_set.node.items {
                if let Selection::Field(child_field) = &selection.node {
                    let child_name = child_field.node.name.node.as_str();
                    to_return.push('\'');
                    to_return.push_str(child_name);
                    to_return.push_str("'::text, (__local_");
                    to_return.push_str(&(self.local_id + 2).to_string());
                    to_return.push_str("__.\"");
                    to_return.push_str(&child_name.to_case(Case::Snake));
                    to_return.push_str("\")");
                }
            }
        }
        to_return.push_str(" ) as object ");
        to_return.push_str("from ( select __local_");
        to_return.push_str(&(self.local_id + 2).to_string());
        to_return.push_str(
            "__.*
                           from \"public\".\"",
        );

        let gql_type = &self.g[endpoints.1];
        to_return.push_str(&gql_type.table_name);
        to_return.push_str("\" as __local_");
        to_return.push_str(&(self.local_id + 2).to_string());
        to_return.push_str(
            "__
                                where (__local_",
        );
        to_return.push_str(&(self.local_id + 2).to_string());
        to_return.push_str("__.\"");
        to_return.push_str(foreign_key_name);
        to_return.push_str("\" = __local_");
        to_return.push_str(&(self.local_id).to_string());
        to_return.push_str("__.\"id\") order by __local_");
        to_return.push_str(&(self.local_id + 2).to_string());
        to_return.push_str(
            "__.\"id\" ASC
                              ) __local_",
        );
        to_return.push_str(&(self.local_id + 2).to_string());
        to_return.push_str(
            "__
                            ) as __local_",
        );
        to_return.push_str(&(self.local_id + 1).to_string());
        to_return.push_str(
            "__ ),
                          '[]'::json
                        )
                      )
                    )",
        );
        to_return.push_str(" as \"@");
        to_return.push_str(parent_field_name);
        to_return.push_str("\",");
        to_return
    }
}
