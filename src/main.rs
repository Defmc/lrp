use std::collections::{HashMap, HashSet};

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
/// A = . . . T B -> {T: FOLLOW(B)}
/// A = . . . . T -> {T: FOLLOW(A)}
pub type Table = HashMap<&'static str, HashSet<&'static str>>;

/// terminal sets
pub type TermSet = HashSet<&'static str>;

#[derive(Debug)]
pub struct Parser {
    pub grammar: Grammar,
    pub terminals: TermSet,
    pub first: Table,
    pub follow: Table,
}

impl Parser {
    #[must_use]
    pub fn new(grammar: Grammar, terminals: TermSet) -> Self {
        let first = Self::gen_first(&grammar);
        let follow = Self::gen_follow(&grammar, &terminals);
        Self {
            grammar,
            terminals,
            first,
            follow,
        }
    }

    /// # Panics
    /// Never.
    #[must_use]
    pub fn gen_first(grammar: &Grammar) -> Table {
        let mut table = Table::new();
        for (name, rules) in grammar {
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
    pub fn gen_follow(grammar: &Grammar, terminals: &TermSet) -> Table {
        let mut table = Table::new();
        for (name, rules) in grammar {
            for rule in rules {
                for term_idx in 0..rule.len() - 1 {
                    if !terminals.contains(rule[term_idx]) {
                        table
                            .entry(rule[term_idx])
                            .or_insert_with(HashSet::new)
                            .insert(rule[term_idx + 1]);
                    }
                }
                let last = rule.last().unwrap();
                if !terminals.contains(last) {
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
}

fn main() {
    /*
    * BNF grammar:
       Start -> Add
       Add -> Add + Factor
       Add -> Factor
       Factor -> Factor * Term
       Factor -> Term
       Term -> Expr
       Term -> Lvalue
       Expr -> ( Add )
       Lvalue -> int
       Lvalue -> ident
    */
    let grammar = grammar! {
        "Start" -> "Add",
        "Add" -> "Add" "+" "Factor"
            | "Factor",
        "Factor" -> "Factor" "*" "Term"
            | "Term",
        "Term" -> "Expr" | "Lvalue",
        "Expr" -> "(" "Add" ")",
        "Lvalue" -> "int" | "ident"
    };
    let terminals = HashSet::from(["int", "ident", "(", "*", "+", ")"]);

    println!("grammar: {grammar:?}");
    println!("terminals: {terminals:?}\n");

    let mut parser = Parser::new(grammar, terminals);
    println!("first-step FIRST table: {:?}", parser.first);
    parser.proc_first();
    println!("final FIRST table: {:?}\n", parser.first);

    println!("first-step FOLLOW table: {:?}", parser.follow);
    parser.proc_follow();
    println!("final FOLLOW talbe: {:?}", parser.follow);
}
