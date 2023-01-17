use std::fmt;

mod grammars_tests {
    use lrp::{grammar, Grammar, Set};
    include!("../grammars_tests.rs");
}

use grammars_tests::GRAMMARS;
use hermes_bench::{BenchSize, Bencher, ClassicBench, IterBench};
use lrp::{dfa::Error, to_tokens, Clr, Dfa, Lalr, Parser, Slr, Tabler};

const BENCH_SIZE: BenchSize = BenchSize::Iters(100);

fn test_table_parser_prod<P: Parser<&'static str> + PartialEq + fmt::Debug>(name: &str) {
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

fn test_dfa<P: Parser<&'static str>>(name: &str) {
    println!("\n{name}'s DFA:");
    for (grammar, inputs, grammar_name) in GRAMMARS {
        let parser = P::new(grammar());
        let actions_copy = || parser.tables().actions.clone();
        let iter = inputs
            .into_iter()
            .cycle()
            .map(|&i| to_tokens(i.into_iter().cloned()))
            .map(|i| (i, actions_copy()));
        let assert = |r| {
            assert!(
                matches!(r, Ok(_) | Err(Error::Conflict(_, _))),
                "rased {r:?}"
            )
        };
        let mut bench = IterBench::new(iter, &|(input, table)| {
            let mut dfa = Dfa::new(input, table, "$");
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
    test_table_parser_prod::<Clr<&'static str>>("Canonical LR");
    test_table_parser_prod::<Lalr<&'static str>>("LALR(1)");
    test_table_parser_prod::<Slr<&'static str>>("SLR");
    test_dfa::<Clr<&'static str>>("Canonical LR");
    test_dfa::<Lalr<&'static str>>("LALR(1)");
    test_dfa::<Slr<&'static str>>("SLR");
}
