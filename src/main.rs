use lrp::{Action, Dfa, Grammar, Set, Tabler};

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
    let grammar = lrp::grammar! {
        // "S" -> "C" "C",
        // "C" -> "c" "C"
        //     | "d"
       "Start" -> "Add",
       "Add" -> "Add" "+" "Factor"
           | "Factor",
       "Factor" -> "Factor" "*" "Term"
           | "Term",
       "Term" -> "(" "Add" ")"
           | "int"
           | "ident"
    };
    let terminals = Set::from(["*", "+", "int", "ident", "(", ")"]);

    println!("grammar: {grammar:?}");
    println!("terminals: {terminals:?}\n");

    let grammar = Grammar::new("Start", grammar, terminals.clone());
    let mut parser = Tabler::new(grammar);

    println!("FIRST table: {:?}", parser.first);
    println!("FOLLOW table: {:?}", parser.follow);

    parser.proc_closures();
    for (kernel, i) in &parser.kernels {
        println!("state {i}: {:?}", kernel);
        for closure in &parser.states[*i] {
            println!("\t{closure}");
        }
    }

    parser.proc_actions();

    let terms: Vec<_> = parser
        .grammar
        .symbols()
        .chain([lrp::EOF].iter().copied())
        .collect();
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

    let mut dfa = Dfa::new(["ident", "*","(", "int", "+", "ident", ")"].into_iter(), parser.actions);
    dfa.start()
}
