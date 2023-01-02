use criterion::criterion_main;

pub mod grammars;
pub mod parsers;

#[macro_export]
macro_rules! gen_bench_table_construction {
    ($parser:tt, $($grammar:tt),+) => {
        $(pub fn $grammar(c: &mut criterion::Criterion) {
            let parsed = $parser::new($crate::grammars::$grammar());
            c.bench_function(
                concat!(
                    stringify!($parser),
                    " grammar parsing",
                    stringify!($grammar)
                ),
                |b| {
                    b.iter(|| {
                        assert_eq!(
                            parsed,
                            $parser::new(criterion::black_box($crate::grammars::$grammar()))
                        )
                    })
                },
            );
        })+
    };
}

criterion_main!(parsers::table_gens);
