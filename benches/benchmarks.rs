use std::{fmt, time::Duration};

use hermes_bench::{BenchSize, Bencher, ClassicBench, IterBench};
use lrp::{dfa::Error, Clr, Dfa, Grammar, Parser, Slr, Tabler};

mod grammars;

const BENCH_SIZE: BenchSize = BenchSize::Iters(100);

const GRAMMARS: &[(fn() -> Grammar, &[&[&str]], &'static str)] = &[
    (
        grammars::dragon_book,
        grammars::DRAGON_BOOK_INPUTS,
        "dragon's book",
    ),
    (grammars::serokell, grammars::SEROKELL_INPUTS, "serokell"),
    (
        grammars::ucalgary_uni_oth_lr1,
        grammars::UCALGARY_UNI_OTH_LR1_INPUTS,
        "ucalgary_uni_oth_lr1",
    ),
    (grammars::wikipedia, grammars::WIKIPEDIA_INPUTS, "wikipedia"),
    (grammars::puncs, grammars::PUNCS_INPUTS, "punctuations"),
];

fn test_table_parser_prod<P: Parser + PartialEq + fmt::Debug>(name: &str) {
    println!("\n{name} productions:");
    for (grammar, _, grammar_name) in GRAMMARS {
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
    for (grammar, _, grammar_name) in GRAMMARS {
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

fn test_dfa<P: Parser>(name: &str) {
    println!("\n{name}'s DFA:");
    for (grammar, inputs, grammar_name) in GRAMMARS {
        let parser = P::new(grammar());
        let actions_copy = || parser.tables().actions.clone();
        let iter = inputs.into_iter().cycle().map(|i| (i, actions_copy()));
        let assert = |r| assert!(matches!(r, Ok(_) | Err(Error::Conflict(_, _))));
        let mut bench = IterBench::new(iter, &|(input, table)| {
            let mut dfa = Dfa::new(input.into_iter().copied(), table);
            dfa.start()
        })
        .with_name(format!("{grammar_name} parsing inputs"))
        .with_post(&assert)
        .with_size(BENCH_SIZE);
        bench.run();
        println!("\t{bench}");
    }
}

fn main() {
    test_table_gen();
    test_table_parser_prod::<Clr>("Canonical LR");
    test_table_parser_prod::<Slr>("SLR");
    test_dfa::<Clr>("Canonical LR");
    test_dfa::<Slr>("SLR");
}
