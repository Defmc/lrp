use std::{fmt, time::Duration};

use hermes_bench::{BenchSize, Bencher, ClassicBench};
use lrp::{Clr, Grammar, Lalr, Parser, Tabler};

mod grammars;

const BENCH_SIZE: BenchSize = BenchSize::Time(Duration::from_secs(1));

const GRAMMARS: &[(fn() -> Grammar, &'static str)] = &[
    (grammars::dragon_book, "dragon's book"),
    (grammars::serokell, "serokell"),
    (grammars::ucalgary_uni_oth_lr1, "ucalgary_uni_oth_lr1"),
    (grammars::wikipedia, "wikipedia"),
    (grammars::puncs, "punctuations"),
];

fn test_table_parser_prod<P: Parser + PartialEq + fmt::Debug>(name: &str) {
    println!("\n{name} productions:");
    for (grammar, grammar_name) in GRAMMARS {
        let parser = P::new(grammar());
        let assert = |p| assert_eq!(p, parser);
        let table_copy = || Tabler::new(grammar());
        let mut bench = ClassicBench::new(&table_copy, &|t| P::with_table(t))
            .with_name(format!("{grammar_name} table production"))
            .with_post(&assert)
            .with_size(BENCH_SIZE);
        bench.run();
        println!("\t{bench}");
    }
}

fn test_table_gen() {
    println!("\nTabler setup:");
    for (grammar, grammar_name) in GRAMMARS {
        let table = Tabler::new(grammar());
        let assert = |p| assert_eq!(p, table);
        let grammar_copy = || grammar();
        let mut bench = ClassicBench::new(&grammar_copy, &|g| Tabler::new(g))
            .with_name(format!("{grammar_name} setup"))
            .with_post(&assert)
            .with_size(BENCH_SIZE);
        bench.run();
        println!("\t{bench}");
    }
}

fn main() {
    test_table_gen();
    test_table_parser_prod::<Clr>("Canonical LR");
    test_table_parser_prod::<Lalr>("LALR(1)");
}
