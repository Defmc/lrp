use std::{
    collections::{BTreeMap, BTreeSet},
    ops::{Index, Range},
};

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

/// Terms table.
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Sym<T, U> {
    Term(T),
    NoTerm(U),
}

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Meta<T> {
    pub item: T,
    pub span: Span,
}

impl<T> Meta<T> {
    pub fn new(item: T, span: Span) -> Self {
        Self { item, span }
    }
}

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

impl<M> Token<(), M> {
    pub const fn empty(ty: M) -> Self {
        Self::new((), ty)
    }
}

pub fn to_tokens<T: Clone>(it: impl IntoIterator<Item = T>) -> impl Iterator<Item = Token<(), T>> {
    it.into_iter().map(|i| Token::new((), i))
}

/// A exclusive Span struct, for indexing metadata on Tokens.
/// start is where it BEGINS, end is UNTIL collect.
/// ```
/// use lrp::Span;
/// let i: Span = Span::new(0, 2);
/// let src = [0, 1, 2, 3, 4];
/// assert_eq!(i.from_source(&src), &[0, 1]);
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn from_source<'a, T: Index<Range<usize>> + ?Sized>(
        &'a self,
        slice: &'a T,
    ) -> &'a T::Output {
        slice.index(self.start..self.end)
    }
}

impl From<(usize, usize)> for Span {
    fn from(value: (usize, usize)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

#[macro_export]
macro_rules! grammar_map {
    ($($rule:literal -> $($($terms:literal)*)|*),*) => {{
        let mut hmp = $crate::Map::new();
        $($crate::grammar_map!(hmp, $rule -> $($($terms)*)|*);)*
        hmp
    }};
    ($grammar:tt, $rule:literal -> $($($terms:literal)*)|*) => {{
        let rule = $crate::grammar::Rule::new($rule, vec![$(vec![$($terms),*]),*]);
        $grammar.insert($rule, rule);
    }};
    ($($rule:ident -> $($($terms:ident)*)|*),*) => {{
        let mut hmp = $crate::Map::new();
        $($crate::grammar_map!(hmp, $rule -> $($($terms)*)|*);)*
        hmp
    }};
    ($grammar:tt, $rule:ident -> $($($terms:ident)*)|*) => {{
        let rule = $crate::grammar::Rule::new($rule, vec![$(vec![$($terms),*]),*]);
        $grammar.insert($rule, rule);
    }}
}

#[cfg(test)]
pub mod grammars_tests {
    use crate::Grammar;
    include!("../grammars_tests.rs");
}
