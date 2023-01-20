use std::collections::{BTreeMap, BTreeSet};

pub mod grammar;
pub use grammar::*;

pub mod parser;
pub use parser::*;

pub mod clr;
pub use clr::Clr;

pub mod lalr;
pub use lalr::Lalr;

pub mod slr;
pub use slr::Slr;

pub type Map<K, V> = BTreeMap<K, V>;
pub type Set<T> = BTreeSet<T>;

pub type ActTable<T> = Vec<Map<T, Action<T>>>;

pub type Rule = &'static str;
pub type State<T> = Set<Position<T>>;

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
pub type Table<T> = Map<T, Set<T>>;

/// For a given f(x), processes `x` until f(x) = f(f(x)) -> f(f(f(f....f(x)))) = f(x)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token<T, M> {
    pub item: T,
    pub ty: M,
}

impl<T, M> Token<T, M> {
    pub const fn new(item: T, ty: M) -> Self {
        Self { item, ty }
    }
}

pub fn to_tokens<T: Clone>(it: impl IntoIterator<Item = T>) -> impl Iterator<Item = Token<(), T>> {
    it.into_iter().map(|i| Token::new((), i))
}

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
    include!("../grammars_tests.rs");
}
