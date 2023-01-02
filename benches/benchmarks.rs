pub mod grammars;
pub mod parsers;

#[macro_export]
macro_rules! gen_bench_table_construction {
    ($parser:tt, $($grammar:tt),+) => {
        $(pub fn $grammar(b: &mut bencher::Bencher) {
            let parsed = $parser::new($crate::grammars::$grammar());
            let table = lrp::Tabler::new($crate::grammars::$grammar());
            b.bench_n(10, |b| b.iter(|| {
                assert_eq!(
                    parsed,
                    $parser::with_table(std::hint::black_box(table.clone()))
                );
            }));
        })+
    };
}

bencher::benchmark_main!(parsers::table_gens);
