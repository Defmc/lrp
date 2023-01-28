use std::{fmt::Display, rc::Rc};

use crate::{Map, Position, Set};

/// Production Rule + Index In Declaration
pub type Production<T> = (Vec<T>, usize);
pub type RuleMap<T> = Map<T, Rule<T>>;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rule<T> {
    pub name: T,
    pub prods: Vec<Rc<Production<T>>>,
}

impl<T> Rule<T> {
    #[must_use]
    pub fn new<I>(name: T, prods: I) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        Self {
            name,
            prods: prods
                .into_iter()
                .enumerate()
                .map(|(i, p)| Rc::new((p, i)))
                .collect(),
        }
    }

    #[must_use]
    pub fn single(name: T, prod: Vec<T>) -> Self {
        Self::new(name, std::iter::once(prod))
    }

    pub fn prods(&self) -> impl Iterator<Item = Rc<Production<T>>> + '_ {
        self.prods.iter().cloned()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Grammar<T>
where
    T: Ord + Clone + Display,
{
    pub rules: RuleMap<T>,
    pub terminals: Set<T>,
    pub symbols: Set<T>,
    pub basis: Position<T>,
}

impl<T> Grammar<T>
where
    T: Ord + Clone + Display,
{
    #[must_use]
    pub fn new(start: T, rules: RuleMap<T>, eof: T) -> Self {
        let mut terminals = Set::new();
        for rule in rules.values() {
            for rc in rule.prods() {
                for term in &rc.0 {
                    if !rules.contains_key(term) {
                        terminals.insert(term.clone());
                    }
                }
            }
        }
        terminals.insert(eof.clone());
        let symbols = rules.keys().chain(terminals.iter()).cloned().collect();

        let prods = &rules[&start].prods;
        debug_assert_eq!(prods.len(), 1, "there's more than one possible entry");
        let basis = Position::new(start, prods[0].clone(), 0, Set::from([eof]));
        Self {
            rules,
            terminals,
            symbols,
            basis,
        }
    }

    #[must_use]
    pub fn basis(&self) -> Position<T> {
        self.basis.clone()
    }

    #[must_use]
    pub fn is_terminal(&self, term: &T) -> bool {
        !self.rules.contains_key(term)
    }

    pub fn rules(&self) -> impl Iterator<Item = &Rule<T>> {
        self.rules.values()
    }

    pub fn symbols(&self) -> impl Iterator<Item = T> + '_ {
        self.symbols.iter().cloned()
    }
}
