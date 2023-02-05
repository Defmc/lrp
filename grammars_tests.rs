pub type GrammarProd = fn() -> Grammar<&'static str>;

pub const GRAMMARS: &[(GrammarProd, &[&[&str]], &str)] = &[
    (dragon_book, DRAGON_BOOK_INPUTS, "dragon's book"),
    (serokell, SEROKELL_INPUTS, "serokell"),
    (
        ucalgary_uni_oth_lr1,
        UCALGARY_UNI_OTH_LR1_INPUTS,
        "ucalgary_uni_oth_lr1",
    ),
    (wikipedia, WIKIPEDIA_INPUTS, "wikipedia"),
    (puncs, PUNCS_INPUTS, "punctuations"),
    (scanner, SCANNER_INPUTS, "scanner"),
];

pub const DRAGON_BOOK_INPUTS: &[&[&str]] = &[
    &["d", "d"],
    &["d", "c", "d"],
    &["c", "d", "d"],
    &["d", "c", "c", "d"],
    &["c", "d", "c", "d"],
    &["c", "c", "d", "d"],
    &["d", "c", "c", "c", "d"],
    &["c", "d", "c", "c", "d"],
    &["c", "c", "d", "c", "d"],
    &["c", "c", "c", "d", "d"],
    &["d", "c", "c", "c", "c", "d"],
    &["c", "d", "c", "c", "c", "d"],
    &["c", "c", "d", "c", "c", "d"],
    &["c", "c", "c", "d", "c", "d"],
    &["c", "c", "c", "c", "d", "d"],
    &["d", "c", "c", "c", "c", "c", "d"],
    &["c", "d", "c", "c", "c", "c", "d"],
    &["c", "c", "d", "c", "c", "c", "d"],
    &["c", "c", "c", "d", "c", "c", "d"],
    &["c", "c", "c", "c", "d", "c", "d"],
    &["c", "c", "c", "c", "c", "d", "d"],
    &["d", "c", "c", "c", "c", "c", "c", "c", "c", "d"],
    &["c", "d", "c", "c", "c", "c", "c", "c", "c", "d"],
    &["c", "c", "d", "c", "c", "c", "c", "c", "c", "d"],
    &["c", "c", "c", "d", "c", "c", "c", "c", "c", "d"],
    &["c", "c", "c", "c", "c", "c", "d", "c", "c", "d"],
    &["c", "c", "c", "c", "c", "c", "c", "d", "c", "d"],
    &["c", "c", "c", "c", "c", "c", "c", "c", "d", "d"],
    &["d", "c", "c", "c", "c", "c", "c", "c", "d"],
    &["c", "d", "c", "c", "c", "c", "c", "c", "d"],
];

pub fn dragon_book() -> Grammar<&'static str> {
    /*
        S -> C C.
        C -> c C.
        C -> d.
    */
    let grammar = grammar_map! {
        "S" -> "C" "C",
        "C" -> "c" "C"
            | "d"
    };
    Grammar::new("S", grammar, "$")
}

pub const SEROKELL_INPUTS: &[&[&str]] = &[
    &["int"],
    &["int", "*", "int"],
    &["ident", "*", "int"],
    &["(", "int", ")"],
    &["int", "+", "int"],
    &["ident", "+", "int"],
    &["int", "*", "int", "*", "int"],
    &["int", "*", "ident", "*", "int"],
    &["ident", "*", "int", "*", "int"],
    &["ident", "*", "ident", "*", "int"],
    &["int", "*", "(", "int", ")"],
    &["ident", "*", "(", "int", ")"],
    &["int", "*", "int", "+", "int"],
    &["int", "*", "(", "ident", "+", "int", ")"],
    &["ident", "*", "int", "+", "int"],
    &[
        "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(",
        "(", "(", "(", "(", "int", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
        ")", ")", ")", ")", ")", ")", ")", ")", ")",
    ],
];

pub fn serokell() -> Grammar<&'static str> {
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
    let grammar = grammar_map! {
        "Start" -> "Add",
        "Add" -> "Add" "+" "Factor"
            | "Factor",
        "Factor" -> "Factor" "*" "Term"
            | "Term",
        "Term" -> "(" "Add" ")"
            | "int"
            | "ident"
    };

    Grammar::new("Start", grammar, "$")
}

pub const NON_LALR_UCALGARY_UNI_OTH_LR1_INPUTS: &[&[&str]] = &[
    &["d", "e", "a", "c"],
    &["d", "e", "a", "b"],
    &["e", "d", "a", "b"],
    &["e", "d", "a", "c"],
];

pub const NON_SLR_UCALGARY_UNI_OTH_LR1_INPUTS: &[&[&str]] = &[
    &["d", "e", "a", "c"],
    &["d", "e", "a", "b"],
    &["e", "d", "a", "b"],
    &["e", "d", "a", "c"],
];

pub const UCALGARY_UNI_OTH_LR1_INPUTS: &[&[&str]] = &[
    &["e", "a", "c"],
    &["d", "a", "b"],
    &["d", "e", "a", "c"],
    &["d", "e", "a", "b"],
    &["e", "d", "a", "b"],
    &["e", "d", "a", "c"],
    &["d", "d", "e", "a", "b"],
    &["e", "e", "d", "a", "c"],
];

pub fn ucalgary_uni_oth_lr1() -> Grammar<&'static str> {
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
    let grammar = grammar_map! {
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

    Grammar::new("S", grammar, "$")
}

pub const WIKIPEDIA_INPUTS: &[&[&str]] = &[
    &["0"],
    &["1"],
    &["0", "*", "0"],
    &["0", "*", "1"],
    &["1", "*", "0"],
    &["1", "*", "1"],
    &["0", "+", "0"],
    &["0", "+", "1"],
    &["1", "+", "0"],
    &["1", "+", "1"],
    &["0", "*", "0", "*", "0"],
    &["0", "*", "0", "*", "1"],
    &["0", "*", "1", "*", "0"],
    &["0", "*", "1", "*", "1"],
    &["1", "*", "0", "*", "0"],
    &["1", "*", "0", "*", "1"],
    &["1", "*", "1", "*", "0"],
    &["1", "*", "1", "*", "1"],
    &["0", "+", "0", "*", "0"],
    &["0", "+", "0", "*", "1"],
    &["0", "+", "1", "*", "0"],
    &["0", "+", "1", "*", "1"],
    &["1", "+", "0", "*", "0"],
    &["1", "+", "0", "*", "1"],
    &["1", "+", "1", "*", "0"],
    &["1", "+", "1", "*", "1"],
    &["0", "*", "0", "+", "0"],
    &["0", "*", "0", "+", "1"],
    &["0", "*", "1", "+", "0"],
    &["0", "*", "1", "+", "1"],
];

pub fn wikipedia() -> Grammar<&'static str> {
    /*
        S -> E.
        E -> E * B.
        E -> E + B.
        E -> B.
        B -> 0.
        B -> 1.
    */
    let grammar = grammar_map! {
        "S" -> "E",
        "E" -> "E" "*" "B"
            | "E" "+" "B"
            | "B",
        "B" -> "0" | "1"
    };

    Grammar::new("S", grammar, "$")
}

pub const PUNCS_INPUTS: &[&[&str]] = &[
    &["(", ")"],
    &["[", "]"],
    &["{", "}"],
    &["(", "(", ")", ")"],
    &["(", "[", "]", ")"],
    &["(", "{", "}", ")"],
    &["[", "(", ")", "]"],
    &["[", "[", "]", "]"],
    &["[", "{", "}", "]"],
    &["{", "(", ")", "}"],
    &["{", "[", "]", "}"],
    &["{", "{", "}", "}"],
    &["(", "(", "(", ")", ")", ")"],
    &["(", "(", "[", "]", ")", ")"],
    &["(", "(", "{", "}", ")", ")"],
    &["(", "[", "(", ")", "]", ")"],
    &["(", "[", "[", "]", "]", ")"],
    &["(", "[", "{", "}", "]", ")"],
    &["(", "{", "(", ")", "}", ")"],
    &["(", "{", "[", "]", "}", ")"],
    &["(", "{", "{", "}", "}", ")"],
    &["[", "(", "(", ")", ")", "]"],
    &["[", "(", "[", "]", ")", "]"],
    &["[", "(", "{", "}", ")", "]"],
    &["[", "[", "(", ")", "]", "]"],
    &["[", "[", "[", "]", "]", "]"],
    &["[", "[", "{", "}", "]", "]"],
    &["[", "{", "(", ")", "}", "]"],
    &["[", "{", "[", "]", "}", "]"],
    &["[", "{", "{", "}", "}", "]"],
];

pub fn puncs() -> Grammar<&'static str> {
    /*
        S -> ( ).
        S -> ( S ).
        S -> [ ].
        S -> [ S ].
        S -> { }.
        S -> { S }.
    */
    let grammar = grammar_map! {
        "S'" -> "S",
        "S" -> "(" ")"
            | "(" "S" ")"
            | "[" "]"
            | "[" "S" "]"
            | "{" "}"
            | "{" "S" "}"
    };

    Grammar::new("S'", grammar, "$")
}

pub const SCANNER_INPUTS: &[&[&str]] = &[
    &[
        "l", "o", "r", "e", "m", "_", "i", "p", "s", "u", "m", "_", "d", "o", "l", "o", "r", "_",
        "s", "i", "t", "_", "a", "m", "e", "t",
    ],
    &[
        "i", "n", "_", "v", "i", "n", "o", "_", "v", "e", "r", "i", "t", "a", "s", "_", "b", "e",
        "f", "o", "r", "e", "_", "7", "9", "_", "a", "c",
    ],
    &[
        "1", "2", "_", "t", "i", "m", "e", "s", "_", "3", "_", "i", "s", "_", "e", "q", "u", "a",
        "l", "_", "t", "o", "_", "4", "0", "_", "m", "i", "n", "u", "s", "_", "4",
    ],
    &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
    &[
        "f", "i", "n", "a", "l", "_", "d", "e", "_", "s", "e", "m", "a", "n", "a", "_", "e", "l",
        "a", "_", "v", "a", "i", "_", "p", "r", "a", "_", "r", "u", "a",
    ],
];

pub fn scanner() -> Grammar<&'static str> {
    /*
    Phrase -> Item Space Phrase.
    Phrase -> Item.

    Item -> Word.
    Item -> Num.

    Word -> Alpha Word.
    Word -> Alpha.

    Num -> Digit Num.
    Num -> Digit.

    Alpha -> a.
    Alpha -> b.
    Alpha -> c.
    Alpha -> d.
    Alpha -> e.
    Alpha -> f.
    Alpha -> g.
    Alpha -> h.
    Alpha -> i.
    Alpha -> j.
    Alpha -> k.
    Alpha -> l.
    Alpha -> m.
    Alpha -> n.
    Alpha -> o.
    Alpha -> p.
    Alpha -> q.
    Alpha -> r.
    Alpha -> t.
    Alpha -> s.
    Alpha -> u.
    Alpha -> v.
    Alpha -> w.
    Alpha -> x.
    Alpha -> y.
    Alpha -> z.

    Digit -> 0.
    Digit -> 1.
    Digit -> 2.
    Digit -> 3.
    Digit -> 4.
    Digit -> 5.
    Digit -> 6.
    Digit -> 7.
    Digit -> 8.
    Digit -> 9.

    Space -> _.
     */
    let grammar = grammar_map! {
        "S" -> "Phrase",

        "Phrase" -> "Item" "Space" "Phrase"
            | "Item",

        "Item" -> "Word" | "Num",

        "Word" -> "Alpha" "Word"
            | "Alpha",

        "Num" -> "Digit" "Num"
            | "Digit",

        "Alpha" -> "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "t" | "s" | "u" | "v" | "w" | "x" | "y" | "z",

        "Digit" -> "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9",

        "Space" -> "_"
    };

    Grammar::new("S", grammar, "$")
}
