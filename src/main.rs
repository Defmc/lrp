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
}
