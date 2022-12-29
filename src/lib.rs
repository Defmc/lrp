use std::collections::{BTreeMap, BTreeSet};

pub const EOF: &str = unsafe { std::str::from_utf8_unchecked(&[0x03]) };
pub const INTERNAL_START_RULE: &str = "LRP'START";

pub mod grammar;
pub use grammar::*;

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
    T: Clone + PartialEq + std::fmt::Debug,
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
