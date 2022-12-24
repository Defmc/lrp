use std::collections::{HashMap, HashSet};
use std::iter::Extend;

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

/// grammars.
/// rule:
///     choice1: [term1, term2, ...],
///     choice2: [term1, term2, ...],
///     ...,
/// rule2...
pub type Grammar = HashMap<&'static str, Vec<Vec<&'static str>>>;

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

pub type State = HashMap<&'static str, HashSet<Position>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Position {
    pub stack: Vec<&'static str>,
    pub top: isize,
    pub lookahead: Vec<&'static str>,
}

impl Position {
    pub fn new(stack: Vec<&'static str>, top: isize, lookahead: Vec<&'static str>) -> Self {
        Self {
            stack,
            top,
            lookahead,
        }
    }

    pub fn adv(&mut self) {
        self.top += 1;
    }

    pub fn peek(&self, idx: isize) -> Option<&'static str> {
        let idx: usize = (self.top + idx).try_into().ok()?;
        self.stack.get(idx).copied()
    }

    pub fn locus(&self) -> Option<&'static str> {
        self.peek(1)
    }

    pub fn top(&self) -> Option<&'static str> {
        self.peek(0)
    }
}

impl From<Vec<&'static str>> for Position {
    fn from(value: Vec<&'static str>) -> Self {
        Self {
            stack: value,
            top: -1,
            lookahead: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Parser {
    pub grammar: Grammar,
    pub terminals: TermSet,
    pub first: Table,
    pub follow: Table,
}

impl Parser {
    #[must_use]
    pub fn new(grammar: Grammar, terminals: TermSet) -> Self {
        let mut buf = Self {
            grammar,
            terminals,
            ..Default::default()
        };
        buf.first = dbg!(buf.gen_first());
        buf.proc_first();
        buf.follow = dbg!(buf.gen_follow());
        buf.proc_follow();
        buf
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn gen_first(&self) -> Table {
        let mut table = Table::new();
        for (name, rules) in &self.grammar {
            table.insert(name, HashSet::new());
            for rule in rules.iter().filter(|r| &r[0] != name) {
                table.get_mut(name).unwrap().insert(rule[0]);
            }
        }
        table
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn gen_follow(&self) -> Table {
        let mut table = Table::new();
        for (name, rules) in &self.grammar {
            for rule in rules {
                for term_idx in 0..rule.len() - 1 {
                    if !self.is_terminal(rule[term_idx]) {
                        let entry = table.entry(rule[term_idx]).or_insert_with(HashSet::new);
                        if self.is_terminal(rule[term_idx + 1]) {
                            // A = . . . T a -> {T: a}
                            entry.insert(rule[term_idx + 1]);
                        } else {
                            // A = . . . T B -> {T: FIRST(B)}
                            entry.extend(self.first[rule[term_idx + 1]].clone())
                        }
                    }
                }
                let last = rule.last().unwrap();
                if !self.is_terminal(last) {
                    // A = . . . . T -> {T: FOLLOW(A)}
                    table.entry(last).or_insert_with(HashSet::new).insert(name);
                }
            }
        }
        table
    }

    pub fn proc_first(&mut self) {
        self.first = transitive(self.first.clone(), |t| self.first_step(&t));
    }

    pub fn proc_follow(&mut self) {
        self.follow = transitive(self.follow.clone(), |t| self.follow_step(&t));
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn first_step(&self, input: &Table) -> Table {
        let mut table = Table::new();
        for (name, firsts) in input {
            table.insert(name, HashSet::new());
            for first in firsts {
                if self.is_terminal(first) {
                    table.get_mut(name).unwrap().insert(first);
                } else {
                    table.get_mut(name).unwrap().extend(&input[first]);
                }
            }
        }
        table
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn follow_step(&self, input: &Table) -> Table {
        let mut table = Table::new();
        for (noterm, terms) in input {
            table.insert(noterm, HashSet::new());
            for term in terms {
                if self.is_terminal(term) {
                    table.get_mut(noterm).unwrap().insert(term);
                } else if let Some(entry) = input.get(term) {
                    table.get_mut(noterm).unwrap().extend(entry);
                }
            }
        }
        table
    }

    #[must_use]
    pub fn is_terminal(&self, term: &str) -> bool {
        self.terminals.contains(term)
    }

    #[must_use]
    pub fn state(&self, rule: &'static str) -> State {
        let mut set = HashSet::new();
        for choice in &self.grammar[rule] {
            set.insert(Position::from(choice.clone()));
        }
        State::from([(rule, set)])
    }

    #[must_use]
    pub fn gen_closure(&self, state: State) -> State {
        let mut new_state = State::new();
        for (name, poss) in state.into_iter() {
            new_state.insert(name, poss.clone());
            for mut pos in poss.into_iter() {
                while let Some(locus) = pos.locus() {
                    if !self.is_terminal(locus) {
                        let lookahead: Vec<&str> = if let Some(next) = pos.peek(2) {
                            if self.is_terminal(next) {
                                vec![next]
                            } else {
                                self.first[next].iter().copied().collect()
                            }
                        } else {
                            self.follow[locus].iter().copied().collect()
                        };
                        let poss: HashSet<_> = self.grammar[locus]
                            .iter()
                            .map(|r: &Vec<&str>| Position::new(r.clone(), -1, lookahead.clone()))
                            .collect();
                        new_state
                            .entry(locus)
                            .or_insert_with(HashSet::new)
                            .extend(poss);
                    }
                    pos.adv();
                }
            }
        }
        new_state
    }
}

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

    let parser = Parser::new(grammar, terminals.clone());

    println!("FIRST table: {:?}", parser.first);
    println!("FOLLOW table: {:?}", parser.follow);

    let test_g = grammar! {
        "S" -> "a" "A" "b" "$"
            | "a" "B" "d" "$",
        "A" -> "x",
        "B" -> "x"
    };
    let test = Parser::new(test_g, terminals);
    println!("closure {:?}", test.gen_closure(test.state("S")));
}
