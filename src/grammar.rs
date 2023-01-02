use std::rc::Rc;

use crate::{Map, Set, Term};

pub type Production = Vec<Term>;
pub type RuleMap = Map<RuleName, Rule>;
pub type RuleName = &'static str;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rule {
    pub name: RuleName,
    pub prods: Vec<Rc<Production>>,
}

impl Rule {
    #[must_use]
    pub fn new(name: RuleName, prods: Vec<Production>) -> Self {
        Self {
            name,
            prods: prods.into_iter().map(Rc::new).collect(),
        }
    }

    #[must_use]
    pub fn single(name: RuleName, prod: Production) -> Self {
        Self::new(name, vec![prod])
    }

    pub fn prods(&self) -> impl Iterator<Item = Rc<Production>> + '_ {
        self.prods.iter().cloned()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Grammar {
    pub rules: RuleMap,
    pub terminals: Set<Term>,
    pub symbols: Set<Term>,
}

impl Grammar {
    #[must_use]
    pub fn new(start: RuleName, mut rules: RuleMap, mut terminals: Set<Term>) -> Self {
        let symbols = rules.keys().chain(terminals.iter()).copied().collect();
        let start = Rule::single("LRP'START", vec![start]);
        terminals.insert(crate::EOF);
        let lrp_start_old = rules.insert(crate::INTERNAL_START_RULE, start);
        debug_assert!(lrp_start_old.is_none(), "`LRP'START` already declared");
        Self {
            rules,
            terminals,
            symbols,
        }
    }

    #[must_use]
    pub fn basis(&self) -> Rc<Production> {
        self.rules[crate::INTERNAL_START_RULE].prods[0].clone()
    }

    #[must_use]
    pub fn is_terminal(&self, term: &Term) -> bool {
        self.terminals.contains(term)
    }

    pub fn rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.values()
    }

    pub fn symbols(&self) -> impl Iterator<Item = Term> + '_ {
        self.symbols.iter().copied()
    }
}
