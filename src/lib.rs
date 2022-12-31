use std::collections::{BTreeMap, BTreeSet};

pub const EOF: &str = unsafe { std::str::from_utf8_unchecked(&[0x03]) };
pub const INTERNAL_START_RULE: &str = "LRP'START";

pub mod grammar;
pub use grammar::*;
pub mod parser;
pub use parser::*;
pub mod clr;
pub use clr::Clr;

/// grammars.
/// rule:
///     prod1: [term1, term2, ...],
///     prod2: [term1, term2, ...],
///     ...,
/// rule2...
///
pub type Map<K, V> = BTreeMap<K, V>;
pub type Set<T> = BTreeSet<T>;

pub type ActTable = Vec<Map<Term, Action>>;

pub type Rule = &'static str;
pub type Term = &'static str;
pub type State = Set<Position>;

/// Terms table,
/// in FIRST:
/// A = a . . . . -> {A: a}
/// A = a . | b . -> {A: a, b}
/// A = B . . . . -> {A: FIRST(B)}
///
/// in FOLLOW:
/// A = . . . T a -> {T: a}
/// A = . . . T B -> {T: FIRST(B)}
/// A = . . . . T -> {T: FOLLOW(A)}
pub type Table = Map<Rule, TermSet>;

/// terminal sets
pub type TermSet = Set<Term>;

/// for a given f(x), processes `x` until f(x) = f(f(x)) -> f(f(f(f....f(x)))) = f(x)
pub fn transitive<T>(seed: T, map: impl Fn(T) -> T) -> T
where
    T: Clone + PartialEq,
{
    let mut val = seed;
    loop {
        let new = map(val.clone());
        if new == val {
            return val;
        }
        val = new;
    }
}

pub mod dfa;
pub use dfa::*;
pub mod tabler;
pub use tabler::*;
pub mod pos;
pub use pos::*;

#[macro_export]
macro_rules! grammar {
    ($($rule:literal -> $($($terms:literal)*)|*),*) => {{
        let mut hmp = $crate::Map::new();
        $($crate::grammar!(hmp, $rule -> $($($terms)*)|*);)*
        hmp
    }};
    ($grammar:tt, $rule:literal -> $($($terms:literal)*)|*) => {{
        let rule = $crate::grammar::Rule::new($rule, vec![$(vec![$($terms),*]),*]);
        $grammar.insert($rule, rule);
    }}
}

#[cfg(test)]
pub mod grammars_tests {
    use crate::{grammar, Grammar, Set};

    pub fn dragon_book() -> Grammar {
        /*
            S -> C C.
            C -> c C.
            C -> d.
        */
        let grammar = grammar! {
            "S" -> "C" "C",
            "C" -> "c" "C"
                | "d"
        };
        Grammar::new("S", grammar, Set::from(["c", "d"]))
    }

    pub fn serokell() -> Grammar {
        /*
            Start -> Add.
            Add -> Add + Factor.
            Add -> Factor.
            Factor -> Factor * Term.
            Factor -> Term.
            Term -> ( Add ).
            Term -> int.
            Term -> ident.
        */
        let grammar = grammar! {
            "Start" -> "Add",
            "Add" -> "Add" "+" "Factor"
                | "Factor",
            "Factor" -> "Factor" "*" "Term"
                | "Term",
            "Term" -> "(" "Add" ")"
                | "int"
                | "ident"
        };

        Grammar::new(
            "Start",
            grammar,
            Set::from(["int", "ident", "(", ")", "+", "*"]),
        )
    }

    pub fn ucalgary_uni_oth_lr1() -> Grammar {
        /*
            S -> E.
            E -> d D.
            E -> D.
            E -> F.
            F -> e C.
            F -> C.
            D -> d e B b.
            D -> e A c.
            C -> e d B c.
            C -> d A b.
            B -> a.
            A -> a.
        */
        let grammar = grammar! {
            "S" -> "E",
            "E" ->	"d" "D"
                |	"D"
                |	"F",
            "F" ->	"e" "C"
                |	"C",
            "D" ->	"d" "e" "B" "b"
                |	"e" "A" "c",
            "C" ->	"e" "d" "B" "c"
                |	"d" "A" "b",
            "B" ->	"a",
            "A" ->	"a"
        };

        Grammar::new("S", grammar, Set::from(["a", "b", "c", "d", "e"]))
    }

    pub fn wikipedia() -> Grammar {
        /*
            S -> E.
            E -> E * B.
            E -> E + B.
            E -> B.
            B -> 0.
            B -> 1.
        */
        let grammar = grammar! {
            "S" -> "E",
            "E" -> "E" "*" "B"
                | "E" "+" "B"
                | "B",
            "B" -> "0" | "1"
        };

        Grammar::new("S", grammar, Set::from(["0", "1", "+", "*"]))
    }
}
