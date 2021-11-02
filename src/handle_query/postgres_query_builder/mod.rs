use convert_case::{Case, Casing};

pub trait GraphQLQueryBuilder {
    fn build_terminal_field(s: &mut String, field_name: &str);
    fn build_terminal_field_join(s: &mut String, child_name: &str, local_id: u8);
    fn sql_query_header(s: &mut String, primary_keys: &Vec<String>);
    fn single_query(s: &mut String, table_name: &str, id: i64);
    fn many_query(s: &mut String, table_name: &str, table_alias: &str);
    fn table_alias(id: u8) -> String;
    fn nested_join_head(s: &mut String, child_name: &str);
    fn join_head(s: &mut String, local_id: u8, is_nested_join: bool, one_to_many: bool) -> u8;
    fn join_tail(
        s: &mut String,
        local_id: u8,
        is_nested_join: bool,
        table_name: &str,
        foreign_keys: (&Vec<String>, &Vec<String>),
        parent_field_name: &str,
        one_to_many: bool,
    );
    fn join_by_foreign_keys(
        s: &mut String,
        foreign_keys: (&Vec<String>, &Vec<String>),
        left_table: &str,
        right_table: &str,
    );
}

pub struct PostgresBuilder {}
impl GraphQLQueryBuilder for PostgresBuilder {
    fn build_terminal_field(s: &mut String, field_name: &str) {
        s.push_str("to_json((__local_0__.\"");
        s.push_str(&field_name.to_case(Case::Snake));
        s.push_str("\"))::text as \"");
        s.push_str(field_name);
        s.push_str("\",\n");
    }
    fn build_terminal_field_join(s: &mut String, field_name: &str, local_id: u8) {
        s.push('\'');
        s.push_str(field_name);
        s.push_str("'::text, (");
        s.push_str(&PostgresBuilder::table_alias(local_id));
        s.push_str(".\"");
        s.push_str(&field_name.to_case(Case::Snake));
        s.push_str("\"),\n");
    }

    fn sql_query_header(s: &mut String, primary_keys: &Vec<String>) {
        s.push_str("select to_json( json_build_array(");
        for pk in primary_keys {
            s.push_str("__local_0__.\"");
            s.push_str(&pk);
            s.push_str("\", ");
        }

        //remove last comma
        s.drain(s.len() - 2..s.len());
        s.push_str(") ) as \"__identifiers\", ")
    }
    fn many_query(s: &mut String, table_name: &str, table_alias: &str) {
        s.push_str(" from ( select ");
        s.push_str(table_alias);
        s.push_str(".* from \"public\".\"");
        s.push_str(table_name);
        s.push_str("\" as  ");
        s.push_str(table_alias);
        s.push_str(" order by ");
        s.push_str(table_alias);
        s.push_str(".\"id\" ASC ) ");
        s.push_str(table_alias);
    }
    fn table_alias(id: u8) -> String {
        let mut local_string = String::from("__local_");
        local_string.push_str(&id.to_string());
        local_string.push_str("__");
        local_string
    }
    fn single_query(s: &mut String, table_name: &str, id: i64) {
        s.push_str(" from \"public\".\"");
        s.push_str(table_name);
        s.push_str("\" as __local_0__ where ( __local_0__.\"id\" = ");
        s.push_str(&id.to_string());
        s.push_str(" )");
    }
    fn nested_join_head(s: &mut String, child_name: &str) {
        s.push_str("\'@");
        s.push_str(child_name);
        s.push_str("'::text, ( ");
    }
    fn join_head(s: &mut String, mut local_id: u8, is_nested_join: bool, one_to_many: bool) -> u8 {
        if is_nested_join {
            s.push_str(" to_json(\n (\n")
        }
        if one_to_many {
            local_id += 2;
            s.push_str("select coalesce( ( select json_agg(");
            s.push_str(&PostgresBuilder::table_alias(local_id - 1));
            s.push_str(".\"object\") from (");
        } else {
            local_id += 1;
        }
        s.push_str(" select json_build_object( '__identifiers'::text, json_build_array(");
        s.push_str(&PostgresBuilder::table_alias(local_id));
        s.push_str(".\"id\"), ");
        local_id
    }
    fn join_tail(
        s: &mut String,
        local_id: u8,
        is_nested_join: bool,
        table_name: &str,
        foreign_keys: (&Vec<String>, &Vec<String>),
        parent_field_name: &str,
        one_to_many: bool,
    ) {
        s.drain(s.len() - 2..s.len());
        s.push_str(" ) as object  ");

        s.push_str("from ");
        if one_to_many {
            s.push_str("( select ");
            s.push_str(&PostgresBuilder::table_alias(local_id));
            s.push_str(".* from \"public\".\"");
            s.push_str(table_name);
            s.push_str("\" as ");
            s.push_str(&PostgresBuilder::table_alias(local_id));
            PostgresBuilder::join_by_foreign_keys(
                s,
                foreign_keys,
                &PostgresBuilder::table_alias(local_id),
                &PostgresBuilder::table_alias(local_id - 2),
            );
            s.push_str(") order by ");
            s.push_str(&PostgresBuilder::table_alias(local_id));
            s.push_str(".\"id\" ASC ) ");
            s.push_str(&PostgresBuilder::table_alias(local_id));
            s.push_str(" ) as ");
            s.push_str(&PostgresBuilder::table_alias(local_id - 1));
            s.push_str(
                " ), '[]'::json ) )
                ",
            );
        } else {
            s.push_str(" \"public\".\"");
            s.push_str(table_name);
            s.push_str("\" as ");
            s.push_str(&PostgresBuilder::table_alias(local_id));
            PostgresBuilder::join_by_foreign_keys(
                s,
                foreign_keys,
                &PostgresBuilder::table_alias(local_id - 1),
                &PostgresBuilder::table_alias(local_id),
            );
            s.push_str(") )  ");
        }
        if is_nested_join {
            s.push_str(") as \"@");
            s.push_str(parent_field_name);
            s.push_str("\", ");
        }
    }
    fn join_by_foreign_keys(
        s: &mut String,
        foreign_keys: (&Vec<String>, &Vec<String>),
        left_table: &str,
        right_table: &str,
    ) {
        for fk in foreign_keys.0.iter().zip(foreign_keys.1) {
            s.push_str(" where (");
            s.push_str(left_table);
            s.push_str(".\"");
            s.push_str(&fk.0);
            s.push_str("\" = ");
            s.push_str(right_table);
            s.push_str(".\"");
            s.push_str(&fk.1);
            s.push('\"');
        }
    }
}
