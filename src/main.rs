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

type Grammar = HashMap<&'static str, Vec<Vec<&'static str>>>;
type Table = HashMap<&'static str, HashSet<&'static str>>;
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
}
