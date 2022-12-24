use std::collections::{HashMap, HashSet};

/// grammars.
/// rule:
///     choice1: [term1, term2, ...],
///     choice2: [term1, term2, ...],
///     ...,
/// rule2...
pub type Grammar = HashMap<&'static str, Vec<Vec<&'static str>>>;

pub type ActTable = Vec<HashMap<&'static str, State>>;

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
pub type Table = HashMap<&'static str, HashSet<&'static str>>;

/// terminal sets
pub type TermSet = HashSet<&'static str>;

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
