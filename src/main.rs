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
