use postgres::{Client, NoTls, Row};

pub fn read_tables() -> Result<Vec<Row>, postgres::Error> {
    let mut client = Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls)?;
    //https://stackoverflow.com/questions/1152260/how-to-list-table-foreign-keys
    let query_res = client.query(
        "
            select cols.table_name, cols.column_name, cols.data_type, ccu.table_name as foreign_table_name,
            case is_nullable
                when 'NO' then '!'
                when 'YES' then ''
            end as nullable
            from information_schema.columns as cols 
            JOIN information_schema.table_constraints AS tc on cols.table_name = tc.table_name and cols.table_schema = tc.table_schema 
            full JOIN information_schema.key_column_usage AS kcu
              ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
            full JOIN information_schema.constraint_column_usage AS ccu
              ON ccu.constraint_name = tc.constraint_name
              AND ccu.table_schema = tc.table_schema
              AND ccu.column_name = cols.column_name 
              AND ccu.table_schema = 'public'
            where cols.table_schema = 'public'
            group by cols.table_name, cols.column_name, cols.data_type, nullable, foreign_table_name;

",
            //WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = 'public' AND ccu.table_schema = 'public'
        &[],
    )?;
    Ok(query_res)
}

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
