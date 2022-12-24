use std::collections::{HashMap, HashSet};

use lrp::{Dfa, State, Tabler};

macro_rules! rule {
    ($grammar:tt, $rule:literal -> $($($terms:literal)*)|*) => {
        $grammar.insert($rule, vec![$(vec![$($terms),*]),*]);
    }
}

macro_rules! grammar {
    ($($rule:literal -> $($($terms:literal)*)|*),*) => {{
        let mut hmp = HashMap::new();
        $(rule!(hmp, $rule -> $($($terms)*)|*);)*
        hmp
    }}
}

/* FOLLOW table:
 * S = B
 * A = ab
 * B = Ac
 *
 * {
 * A: c,
 * }
 */

fn main() {
    /*
    * BNF grammar:
       Start -> Add
       Add -> Add + Factor
       Add -> Factor
       Factor -> Factor * Term
       Factor -> Term
       Term -> ( Add )
       Term -> int
       Term -> ident
    */
    let grammar = grammar! {
        "Start" -> "Add",
        "Add" -> "Add" "+" "Factor"
            | "Factor",
        "Factor" -> "Factor" "*" "Term"
            | "Term",
        "Term" -> "(" "Add" ")" | "int" | "ident"
    };
    let terminals = HashSet::from(["int", "ident", "(", "*", "+", ")", "$", "a", "b", "d", "x"]);

    println!("grammar: {grammar:?}");
    println!("terminals: {terminals:?}\n");

    let parser = Tabler::new(grammar, terminals.clone());

    println!("FIRST table: {:?}", parser.first);
    println!("FOLLOW table: {:?}", parser.follow);

    let actions = vec![
        HashMap::from([
            ("$", State::Reduce(0, "program", "empty")),
            ("varDecl", State::Reduce(0, "program", "empty")),
            ("constDecl", State::Reduce(0, "program", "empty")),
            ("statement", State::Reduce(0, "program", "empty")),
            ("program", State::Shift(1)),
        ]),
        HashMap::from([
            ("$", State::Acc),
            ("declaration", State::Shift(2)),
            ("varDecl", State::Shift(3)),
            ("constDecl", State::Shift(4)),
            ("statement", State::Shift(5)),
        ]),
        HashMap::from([
            ("$", State::Reduce(2, "program", "sequence")),
            ("varDecl", State::Reduce(2, "program", "sequence")),
            ("constDecl", State::Reduce(2, "program", "sequence")),
            ("statement", State::Reduce(2, "program", "sequence")),
        ]),
        HashMap::from([
            ("$", State::Reduce(1, "declaration", "var")),
            ("varDecl", State::Reduce(1, "declaration", "var")),
            ("constDecl", State::Reduce(1, "declaration", "var")),
            ("statement", State::Reduce(1, "declaration", "var")),
        ]),
        HashMap::from([
            ("$", State::Reduce(1, "declaration", "const")),
            ("varDecl", State::Reduce(1, "declaration", "const")),
            ("constDecl", State::Reduce(1, "declaration", "const")),
            ("statement", State::Reduce(1, "declaration", "const")),
        ]),
        HashMap::from([
            ("$", State::Reduce(1, "declaration", "statement")),
            ("varDecl", State::Reduce(1, "declaration", "statement")),
            ("constDecl", State::Reduce(1, "declaration", "statement")),
            ("statement", State::Reduce(1, "declaration", "statement")),
        ]),
    ];

    let mut dfa = Dfa::new(vec!["var", "abc", "$"], actions);
    dfa.start("varDecl");
    println!("items: {:?}", dfa.forest);
}
