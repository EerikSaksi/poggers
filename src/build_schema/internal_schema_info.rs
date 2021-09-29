use crate::build_schema::read_database;
use petgraph::graph::DiGraph;
use std::collections::HashSet;

pub struct GraphQLType {
    selections: HashSet<String>,
    table_name: String,
}

pub fn create() -> DiGraph<GraphQLType, ()> {
    let mut g: DiGraph<GraphQLType, ()> = DiGraph::new();
    for current_row in read_database::read_tables().unwrap().iter() {
        let table_name: String = current_row.get("table_name");
        let column_name: String = current_row.get("column_name");
        let foreign_table_name: Option<String> = current_row.get("foreign_table_name");
        //let nullable: &str = current_row.get("nullable");
        //let data_type: &str = current_row.get("data_type");

        //add this table as a node if no such node

        if let Some(foreign_table_name) = foreign_table_name {
            //unwrap as we inserted this node if it was missing right above
            //we dont know about the foreign table
            let source_index_optional = g.node_indices().find(|i| g[*i].table_name == table_name);
            let source_index = match source_index_optional {
                Some(index) => index,
                None => g.add_node(GraphQLType {
                    selections: HashSet::new(),
                    table_name: table_name.clone(),
                }),
            };
            let foreign_index_optional = g
                .node_indices()
                .find(|i| g[*i].table_name == foreign_table_name);

            //either return the index we found or insert and return the index of that new item
            let foreign_index = match foreign_index_optional {
                Some(foreign_index) => foreign_index,
                None => g.add_node(GraphQLType {
                    selections: HashSet::new(),
                    table_name: foreign_table_name,
                }),
            };
            g.add_edge(source_index, foreign_index, ());
        }
        g.node_weights_mut().for_each(|graphql_type| {
            if graphql_type.table_name == table_name {
                graphql_type.selections.insert(column_name.clone());
            }
        });
    }
    g
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_tables_in_graph() {
        let g = super::create();
        for table_name in [
            "app_user",
            "completed_set",
            "completed_workout",
            "completed_workout_exercise",
            "exercise",
            "exercise_alias",
            "session_analytics",
            "sets_and_exercise_id",
            "user_exercise",
            "workout_plan",
            "workout_plan_day",
            "workout_plan_exercise",
        ] {
            assert!(g
                .node_weights()
                .any(|gql_type| gql_type.table_name == table_name));
        }
    }
}
