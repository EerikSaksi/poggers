extern crate test;

pub fn add_two(a: i32) -> i32 {
    a + 2
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
}
