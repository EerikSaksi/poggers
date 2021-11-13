use postgres::{Client, NoTls, Row};

pub fn read_tables(database_url: &str) -> Result<Vec<Row>, postgres::Error> {
    let mut client = Client::connect(database_url, NoTls)?;
    let query = "
        select
          cols.table_name,
          cols.column_name,
          data_type,
          foreign_keys.parent_table,
          foreign_keys.parent_column,
					primary_keys.column_name is not null as is_primary,
            is_nullable
        from
          information_schema.columns as cols
          left join (
            select
              att2.attname as \"child_column\",
              cl.relname as \"parent_table\",
              att.attname as \"parent_column\",
              child_table
            from
              (
                select
                  unnest(con1.conkey) as \"parent\",
                  unnest(con1.confkey) as \"child\",
                  con1.confrelid,
                  con1.conrelid,
                  con1.conname,
                  cl.relname as child_table,
                  ns.nspname as child_schema
                from
                  pg_class cl
                  join pg_namespace ns on cl.relnamespace = ns.oid
                  join pg_constraint con1 on con1.conrelid = cl.oid
                where
                  con1.contype = 'f'
              ) con
              join pg_attribute att on att.attrelid = con.confrelid
              and att.attnum = con.child
              join pg_class cl on cl.oid = con.confrelid
              join pg_attribute att2 on att2.attrelid = con.conrelid
              and att2.attnum = con.parent
          ) as foreign_keys on foreign_keys.child_column = cols.column_name
          and foreign_keys.child_table = cols.table_name
					left join 
					(
						SELECT c.column_name, tc.table_name
						FROM information_schema.table_constraints tc 
						JOIN information_schema.constraint_column_usage AS ccu USING (constraint_schema, constraint_name) 
						JOIN information_schema.columns AS c ON c.table_schema = tc.constraint_schema
							AND tc.table_name = c.table_name AND ccu.column_name = c.column_name
						WHERE constraint_type = 'PRIMARY KEY'
					) as primary_keys on primary_keys.column_name = cols.column_name and primary_keys.table_name = cols.table_name
        where cols.table_schema = 'public'
        order by table_name";
    let query_res = client.query(&*query, &[])?;
    Ok(query_res)
}

#[allow(dead_code)]
pub fn read_type_information() -> Result<Vec<Row>, postgres::Error> {
    let mut client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls)?;
    let rows = client.query(
        "
        WITH types AS (
            SELECT n.nspname,
                    pg_catalog.format_type ( t.oid, NULL ) AS obj_name,
                    CASE
                        WHEN t.typrelid != 0 THEN CAST ( 'tuple' AS pg_catalog.text )
                        WHEN t.typlen < 0 THEN CAST ( 'var' AS pg_catalog.text )
                        ELSE CAST ( t.typlen AS pg_catalog.text )
                        END AS obj_type,
                    coalesce ( pg_catalog.obj_description ( t.oid, 'pg_type' ), '' ) AS description
                FROM pg_catalog.pg_type t
                JOIN pg_catalog.pg_namespace n
                    ON n.oid = t.typnamespace
                WHERE ( t.typrelid = 0
                        OR ( SELECT c.relkind = 'c'
                                FROM pg_catalog.pg_class c
                                WHERE c.oid = t.typrelid ) )
                    AND NOT EXISTS (
                            SELECT 1
                                FROM pg_catalog.pg_type el
                                WHERE el.oid = t.typelem
                                AND el.typarray = t.oid )
                    AND n.nspname <> 'pg_catalog'
                    AND n.nspname <> 'information_schema'
                    AND n.nspname !~ '^pg_toast'
        ),
        cols AS (
            SELECT n.nspname::text AS schema_name,
                    pg_catalog.format_type ( t.oid, NULL ) AS obj_name,
                    a.attname::text AS column_name,
                    pg_catalog.format_type ( a.atttypid, a.atttypmod ) AS data_type,
                    a.attnotnull AS is_required,
                    a.attnum AS ordinal_position,
                    pg_catalog.col_description ( a.attrelid, a.attnum ) AS description
                FROM pg_catalog.pg_attribute a
                JOIN pg_catalog.pg_type t
                    ON a.attrelid = t.typrelid
                JOIN pg_catalog.pg_namespace n
                    ON ( n.oid = t.typnamespace )
                JOIN types
                    ON ( types.nspname = n.nspname
                        AND types.obj_name = pg_catalog.format_type ( t.oid, NULL ) )
                WHERE a.attnum > 0
                    AND NOT a.attisdropped
        )
        SELECT cols.schema_name,
                cols.obj_name,
                cols.column_name,
                cols.data_type,
                cols.ordinal_position,
                cols.is_required,
                coalesce ( cols.description, '' ) AS description
            FROM cols
            where schema_name = 'public' ORDER BY cols.schema_name,
                    cols.obj_name,
                    cols.ordinal_position ;",
        &[],
    )?;
    Ok(rows)
}
