use postgres::Row;
use std::ops::Range;

pub trait ParentPkChecker {
    fn same_parent(&self, row: &Row, parent_pks_range: &Range<usize>, parent_pks: &[i32]) -> bool;
}

pub struct NoNull {}

impl ParentPkChecker for NoNull {
    //this function iterates over the primary keys of the parent and checks whether they are the same
    //as parent_pks (the previous parent.) Pks are assumed to be i32 and not Option<i32> (this should
    //be used when building the root table's children (as the root parent cannot be null))
    fn same_parent(&self, row: &Row, parent_pks_range: &Range<usize>, parent_pks: &[i32]) -> bool {
        for i in 0..parent_pks_range.end {
            let pk_val: i32 = row.get(i);
            if pk_val != *parent_pks.get(i).unwrap() {
                return false;
            };
        }
        true
    }
}

pub struct WithNull {}
impl ParentPkChecker for WithNull {
    //this function iterates over the primary keys of the parent and checks whether they are the same
    //as parent_pks (the previous parent.) Pks are assumed to be i32 and not Option<i32> (this should
    //be used when building the root table's children (as the root parent cannot be null))
    fn same_parent(&self, row: &Row, parent_pks_range: &Range<usize>, parent_pks: &[i32]) -> bool {
        let first_pk: Option<i32> = row.get(parent_pks_range.start);
        match first_pk {
            Some(first_pk) => {
                if first_pk != *parent_pks.get(0).unwrap() {
                    return false;
                };
                let mut i = 0;
                while i + parent_pks_range.start < parent_pks_range.end {
                    let pk_val: i32 = row.get(i + parent_pks_range.start);
                    if pk_val != *parent_pks.get(i).unwrap() {
                        return false;
                    };
                    i += 1;
                }
            }
            None => return false,
        };
        true
    }
}
