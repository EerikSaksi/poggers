use super::*;

#[test]
fn validate_no_duplicate_constraints() {
    let mut client =
        Client::connect("postgres://eerik:Postgrizzly@localhost:5432/rpgym", NoTls).unwrap();
    let rows = client.query(constraint_subquery(), &[]).unwrap();
    for row in &rows {
        let mut exact_matches = 0;
        for inner_row in &rows {
            //check if all the column values match
            if row.columns().iter().all(|col| {
                let left: &str = row.get(col.name());
                let right: &str = inner_row.get(col.name());
                left == right
            }) {
                exact_matches += 1
            }
        }
        //there should be one exact match for each row (itself)
        if exact_matches != 1 {
            panic!("{:?}", row);
        }
    }
}
