use super::create;
use postgres::{Client, NoTls};
fn connect_create_schema(query: &str) -> Result<(), postgres::Error> {
    let mut client = Client::connect(
        "postgres://eerik:Postgrizzly@localhost:5432/poggers_testing",
        NoTls,
    )?;
    client.query("drop schema public cascade", &[])?;
    client.query(query, &[])?;
    Ok(())
}

fn check_single_connection() {
    connect_create_schema(
    "
        create table parent_table(id integer primary key generated always as identity);
        grant all on parent_table to public;
        create table foreign_table(
          id integer primary key generated always as identity,
          parent_table_id integer not null references parent_table(id) on delete cascade
        );
        create index if not exists foreign_table_parent_table_idx on \"foreign_table\"(parent_table_id);
        grant all on foreign_table to public;
    ",
    )
    .unwrap();
}
