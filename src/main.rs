use lrp::{Action, Dfa, Map, Position, Set, Tabler};

macro_rules! rule {
    ($grammar:tt, $rule:literal -> $($($terms:literal)*)|*) => {
        $grammar.insert($rule, vec![$(vec![$($terms),*]),*]);
    }
}

macro_rules! grammar {
    ($($rule:literal -> $($($terms:literal)*)|*),*) => {{
        let mut hmp = Map::new();
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
        // "Program" -> "Start" "$",
        // "Start" -> "Add",
        // "Add" -> "Add" "+" "Factor"
        //     | "Factor",
        // "Factor" -> "Factor" "*" "Term"
        //     | "Term",
        // "Term" -> "(" "Add" ")" | "int" | "ident"
        "S" -> "C" "C",
        "C" -> "c" "C"
            | "d"
    };
    let terminals = Set::from(["$", "c", "d"]);

    println!("grammar: {grammar:?}");
    println!("terminals: {terminals:?}\n");

    let mut parser = Tabler::new(grammar, terminals.clone());

    println!("FIRST table: {:?}", parser.first);
    println!("FOLLOW table: {:?}", parser.follow);

    for rule in parser.grammar.keys() {
        for choice in parser.pos(rule, 0, Set::new()) {
            println!("closures for {choice}:");
            let closures = parser.closure(Set::from([choice]));
            println!("\ttotal:");
            for closure in closures {
                println!("\t\t{closure}");
            }
        }
    }

    let test = Position::new("S", vec!["C", "C"], 0, Set::from(["$"]));
    println!("calculating table {test}");
    parser.proc_closures(test);
    for (i, closures) in parser.states.iter().enumerate() {
        println!("state {i}:");
        for closure in closures {
            println!("\t{closure}");
        }
    }

    parser.proc_actions("S");

    let terms: Vec<_> = parser.syms.iter().chain(["$"].iter()).copied().collect();
    print!("  | ");
    for term in &terms {
        print!("{term}  ");
    }
    for (i, line) in parser.actions.iter().enumerate() {
        print!("\n{i} | ");
        for term in &terms {
            if let Some(a) = line.get(term) {
                let p = match a {
                    Action::Conflict(..) => "cn".into(),
                    Action::Acc => "ac".into(),
                    Action::Reduce(..) => "re".into(),
                    Action::Shift(s) => format!("s{s}"),
                    Action::Goto(g) => format!("g{g}"),
                };
                print!("{p}");
            } else {
                print!("  ");
            }
            print!(" ");
        }
    }
    println!("\n{:?}", parser.actions);

    let mut dfa = Dfa::new(vec!["c", "d", "d", "$"], parser.actions);
    dfa.start()
}
