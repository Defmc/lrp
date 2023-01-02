use lrp::{grammar, Grammar, Set};

pub fn dragon_book() -> Grammar {
    /*
        S -> C C.
        C -> c C.
        C -> d.
    */
    let grammar = grammar! {
        "S" -> "C" "C",
        "C" -> "c" "C"
            | "d"
    };
    Grammar::new("S", grammar, Set::from(["c", "d"]))
}

pub fn serokell() -> Grammar {
    /*
        Start -> Add.
        Add -> Add + Factor.
        Add -> Factor.
        Factor -> Factor * Term.
        Factor -> Term.
        Term -> ( Add ).
        Term -> int.
        Term -> ident.
    */
    let grammar = grammar! {
        "Start" -> "Add",
        "Add" -> "Add" "+" "Factor"
            | "Factor",
        "Factor" -> "Factor" "*" "Term"
            | "Term",
        "Term" -> "(" "Add" ")"
            | "int"
            | "ident"
    };

    Grammar::new(
        "Start",
        grammar,
        Set::from(["int", "ident", "(", ")", "+", "*"]),
    )
}

pub fn ucalgary_uni_oth_lr1() -> Grammar {
    /*
        S -> E.
        E -> d D.
        E -> D.
        E -> F.
        F -> e C.
        F -> C.
        D -> d e B b.
        D -> e A c.
        C -> e d B c.
        C -> d A b.
        B -> a.
        A -> a.
    */
    let grammar = grammar! {
        "S" -> "E",
        "E" ->	"d" "D"
            |	"D"
            |	"F",
        "F" ->	"e" "C"
            |	"C",
        "D" ->	"d" "e" "B" "b"
            |	"e" "A" "c",
        "C" ->	"e" "d" "B" "c"
            |	"d" "A" "b",
        "B" ->	"a",
        "A" ->	"a"
    };

    Grammar::new("S", grammar, Set::from(["a", "b", "c", "d", "e"]))
}

pub fn wikipedia() -> Grammar {
    /*
        S -> E.
        E -> E * B.
        E -> E + B.
        E -> B.
        B -> 0.
        B -> 1.
    */
    let grammar = grammar! {
        "S" -> "E",
        "E" -> "E" "*" "B"
            | "E" "+" "B"
            | "B",
        "B" -> "0" | "1"
    };

    Grammar::new("S", grammar, Set::from(["0", "1", "+", "*"]))
}

pub fn puncs() -> Grammar {
    /*
        S -> ( ).
        S -> ( S ).
        S -> [ ].
        S -> [ S ].
        S -> { }.
        S -> { S }.
    */
    let grammar = grammar! {
        "S" -> "(" ")"
            | "(" "S" ")"
            | "[" "]"
            | "[" "S" "]"
            | "{" "}"
            | "{" "S" "}"
    };

    Grammar::new("S", grammar, Set::from(["(", ")", "[", "]", "{", "}"]))
}
