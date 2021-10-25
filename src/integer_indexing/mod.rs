extern crate test;
use postgres::{Client, NoTls, Row};

fn get_rows() -> Vec<Row> {
    let mut client = Client::connect(
        "postgres://eerik:Postgrizzly@localhost:5432/benchmarks",
        NoTls,
    )
    .unwrap();

    let query = "
        select field1 as  __t0_c1__,  
        field2 as    __t0_c2__,  
        field3 as    __t0_c3__,  
        field4 as    __t0_c4__,  
        field5 as    __t0_c5__,  
        field6 as    __t0_c6__,  
        field7 as    __t0_c7__,  
        field8 as    __t0_c8__,
        field9 as    __t0_c9__,
        field10 as   __t0_c10__,
        field11 as   __t0_c11__,
        field12 as   __t0_c12__,
        field13 as   __t0_c13__,
        field14 as   __t0_c14__,
        field15 as   __t0_c15__,
        field16 as   __t0_c16__,
        field17 as   __t0_c17__,
        field18 as   __t0_c18__,
        field19 as   __t0_c19__,
        field20 as   __t0_c20__,
        field21 as   __t0_c21__,
        field22 as   __t0_c22__,
        field23 as   __t0_c23__,
        field24 as   __t0_c24__,
        field25 as   __t0_c25__,
        field26 as   __t0_c26__,
        field27 as   __t0_c27__,
        field28 as   __t0_c28__,
        field29 as   __t0_c29__,
        field30 as   __t0_c30__,
        field31 as   __t0_c31__,
        field32 as   __t0_c32__,
        field33 as   __t0_c33__,
        field34 as   __t0_c34__,
        field35 as   __t0_c35__,
        field36 as   __t0_c36__,
        field37 as   __t0_c37__,
        field38 as   __t0_c38__,
        field39 as   __t0_c39__,
        field40 as   __t0_c40__,
        field41 as   __t0_c41__,
        field42 as   __t0_c42__,
        field43 as   __t0_c43__,
        field44 as   __t0_c44__,
        field45 as   __t0_c45__,
        field46 as   __t0_c46__,
        field47 as   __t0_c47__,
        field48 as   __t0_c48__,
        field49 as   __t0_c49__,
        field50 as   __t0_c50__,
        field51 as   __t0_c51__,
        field52 as   __t0_c52__,
        field53 as   __t0_c53__,
        field54 as   __t0_c54__,
        field55 as   __t0_c55__,
        field56 as   __t0_c56__,
        field57 as   __t0_c57__,
        field58 as   __t0_c58__,
        field59 as   __t0_c59__,
        field60 as   __t0_c60__,
        field61 as   __t0_c61__,
        field62 as   __t0_c62__,
        field63 as   __t0_c63__,
        field64 as   __t0_c64__,
        field65 as   __t0_c65__,
        field66 as   __t0_c66__,
        field67 as   __t0_c67__,
        field68 as   __t0_c68__,
        field69 as   __t0_c69__,
        field70 as   __t0_c70__,
        field71 as   __t0_c71__,
        field72 as   __t0_c72__,
        field73 as   __t0_c73__,
        field74 as   __t0_c74__,
        field75 as   __t0_c75__,
        field76 as   __t0_c76__,
        field77 as   __t0_c77__,
        field78 as   __t0_c78__,
        field79 as   __t0_c79__,
        field80 as   __t0_c80__,
        field81 as   __t0_c81__,
        field82 as   __t0_c82__,
        field83 as   __t0_c83__,
        field84 as   __t0_c84__,
        field85 as   __t0_c85__,
        field86 as   __t0_c86__,
        field87 as   __t0_c87__,
        field88 as   __t0_c88__,
        field89 as   __t0_c89__,
        field90 as   __t0_c90__,
        field91 as   __t0_c91__,
        field92 as   __t0_c92__,
        field93 as   __t0_c93__,
        field94 as   __t0_c94__,
        field95 as   __t0_c95__,
        field96 as   __t0_c96__,
        field97 as   __t0_c97__,
        field98 as   __t0_c98__,
        field99 as   __t0_c99__,
        field100 as  __t0_c100__ from test_table";
    client.query(query, &[]).unwrap()
}
#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    #[bench]
    fn construct_ids(b: &mut Bencher) {
        b.iter(|| {
            for table in 0..100 {
                for col in 0..100 {
                    let index = test::black_box(table * 100 + col);
                }
            }
        })
    }
    #[bench]
    fn construct_string_ids(b: &mut Bencher) {
        b.iter(|| {
            for table in 0..100 {
                for col in 0..100 {
                    let index = test::black_box(
                        &["__t", &table.to_string(), "_c", &col.to_string(), "__"].concat(),
                    );
                }
            }
        })
    }

    #[bench]
    fn test_postgres_integer_indexing(b: &mut Bencher) {
        let rows = get_rows();
        b.iter(|| {
            for row in &rows {
                for i in 0..100 {
                    let col_val: i32 = test::black_box(row.get(i));
                }
            }
        })
    }

    #[bench]
    fn test_postgres_string_indexing(b: &mut Bencher) {
        let rows = get_rows();

        let table_index = 0;
        b.iter(|| {
            for row in &rows {
                for i in 1..101 {
                    let col_val: i32 = test::black_box(row.get(
                        &*["__t", &table_index.to_string(), "_c", &i.to_string(), "__"].concat(),
                    ));
                }
            }
        })
    }
}
