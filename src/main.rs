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

/* FIRST table:
 * S = B
 * A = ab
 * B = Ac
 *
 * {
 * A: a,
 * B: FIRST(A) = a,
 * S: FIRST(B) = a
 * }
 */

/* FOLLOW table:
 * S = B
 * A = ab
 * B = Ac
 *
 * A = ... R a -> {R: a}
 * A = ... R B -> {R: FOLLOW(B)}
 * A = ... R -> {R: FOLLOW(A)}
 * {
 * A: c,
 * }
 */

type Grammar = HashMap<&'static str, Vec<Vec<&'static str>>>;
type Table = HashMap<&'static str, HashSet<&'static str>>;
type TermSet = HashSet<&'static str>;

fn transitive<T>(seed: T, map: impl Fn(T) -> T) -> T
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

fn gen_first_table(grammar: &Grammar) -> Table {
    let mut table = Table::new();
    for (name, rules) in grammar {
        table.insert(name, HashSet::new());
        for rule in rules.iter().filter(|r| &r[0] != name) {
            table.get_mut(name).unwrap().insert(rule[0]);
        }
    }
    table
}

fn gen_follow_table(grammar: &Grammar, terminals: &TermSet) -> Table {
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

fn first_step(grammar: &Table, terminals: &TermSet) -> Table {
    let mut table = Table::new();
    for (name, firsts) in grammar {
        table.insert(name, HashSet::new());
        for first in firsts {
            if terminals.contains(first) {
                table.get_mut(name).unwrap().insert(first);
            } else {
                merge(table.get_mut(name).unwrap(), &grammar[first]);
            }
        }
    }
    table
}

fn merge(rhs: &mut HashSet<&'static str>, lhs: &HashSet<&'static str>) {
    for item in lhs {
        if !rhs.contains(item) {
            rhs.insert(item);
        }
    }
}

fn main() {
    let rules = grammar! {
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

    println!("rules: {rules:?}");
    println!("terminals: {terminals:?}\n");

    let first_table = gen_first_table(&rules);
    println!("first-step FIRST table: {first_table:?}");

    let final_first = transitive(first_table, |t| first_step(&t, &terminals));
    println!("final FIRST table: {final_first:?}\n");

    let follow_table = gen_follow_table(&rules, &terminals);
    println!("first-step FOLLOW table: {follow_table:?}");

}
