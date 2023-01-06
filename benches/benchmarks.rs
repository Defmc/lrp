use std::time::Duration;

use hermes_bench::{BenchSize, Bencher, IterBench};
use lrp::{Clr, Lalr, Parser};

mod grammars;

const BENCH_SIZE: BenchSize = BenchSize::Time(Duration::from_secs(1));

macro_rules! bench_grammar_prod {
    ($parser:tt, $($grammar:tt),*) => {{
        println!("{}:", stringify!($parser).to_uppercase());
        $(
            let parser = $parser::new(grammars::$grammar());
            let assert_prod = &|new_parser| assert_eq!(new_parser, parser);
            let mut bench = hermes_bench::ClassicBench::new(&grammars::$grammar, &|g| $parser::new(g))
                .with_size(BENCH_SIZE)
                .with_post(&assert_prod)
                .with_name(concat!(stringify!($grammar), " grammar production"));
            bench.run();
            println!("\t{bench}");
        )*
    }};
}

macro_rules! with_grammars {
    ($macro:tt, $($args:tt),*) => {
        $macro!($($args),*, dragon_book, serokell, ucalgary_uni_oth_lr1, wikipedia, puncs);
    }
}

fn main() {
    with_grammars!(bench_grammar_prod, Lalr);
    with_grammars!(bench_grammar_prod, Clr);
}
