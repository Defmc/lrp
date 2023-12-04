use logos::Logos;
use lrp::{Meta, Span, Token};
use std::{fs, time::Instant};
use wop::{builder::Builder, Ast, Gramem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap();
    let file = fs::read_to_string(path)?;
    let mut copy = Vec::new();

    let lexer = wop::Sym::lexer(&file).spanned().map(|(m, s)| {
        println!("\"{}\" ({m:?}) [{s:?}]", &file[s.clone()]);
        copy.push(&file[s.clone()]);
        Token::new(Meta::new(Ast::Token(m), Span::new(s.start, s.end)), m)
    });

    let mut dfa = wop::build_parser(lexer);

    let res = match dfa.trace(|st| println!("{:?}", st.stack_fmt())) {
        Err(e) => {
            println!("FATAL: {e}");
            println!("source:");
            // copy.into_iter().for_each(|a| print!("{a} "));
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "impossible to parse").into())
        }
        Ok(p) => {
            println!("OUTPUT: {p:?}");
            Ok(())
        }
    };
    println!("PARSING OUTPUT:");
    print_nested(&dfa.items[0], "", 0, &file);

    println!("BUILDING OUTPUT");
    let mut builder = Builder::default();
    let start = Instant::now();
    builder.process(&dfa.items[0], &file);
    println!(
        "CODE BUILDER OUTPUT (after {:?}): {builder:?}",
        start.elapsed()
    );

    println!("PRODUCED CODE:\n\x1B[1;33m{}\x1B[0;m", builder.dump(&file));
    res
}

const TAB_C: &str = "  ";

fn print_nested(tok: &Gramem, prefix: &str, lvl: usize, txt: &str) {
    fn print_iter_nested<'a>(
        iter: impl Iterator<Item = &'a Gramem>,
        prefix: &str,
        lvl: usize,
        txt: &str,
    ) {
        iter.for_each(|g| print_nested(g, prefix, lvl, txt));
    }

    println!(
        "{}{prefix}{:?}: \x1B[1;33m\"{}\"\x1B[0;m",
        TAB_C.repeat(lvl),
        tok.ty,
        tok.item.span.from_source(txt)
    );
    let lvl = lvl + 1;
    let tab_spc = TAB_C.repeat(lvl);
    match &tok.item.item {
        Ast::Token(_) => (),
        Ast::EntryPoint(g) => print_nested(g.as_ref(), "", lvl, txt),
        Ast::Program(gs) => print_iter_nested(gs.iter(), "- ", lvl, txt),
        Ast::RuleDecl(g, gs) => {
            println!(
                "{tab_spc}|> rule_name: \x1B[1;33m\"{}\"\x1B[0;m",
                g.from_source(txt)
            );
            for (i, gss) in gs.iter().enumerate() {
                println!("{tab_spc}|> production {i}:");
                print_iter_nested(gss.iter(), "- ", lvl + 1, txt);
            }
        }
        Ast::Rule(gs) => {
            for (i, gss) in gs.iter().enumerate() {
                println!("{tab_spc}|> production {i}:");
                print_iter_nested(gss.iter(), "- ", lvl + 1, txt);
            }
        }
        Ast::RulePipe(gs) => {
            print_iter_nested(gs.iter(), "- ", lvl, txt);
        }
        Ast::Import(g) => println!("{tab_spc}|> path: {}", g.from_source(txt)),
        Ast::Alias(g, h) => {
            println!("{tab_spc}|> alias: {}", g.from_source(txt));
            println!("{tab_spc}|> definition: {}", h.from_source(txt));
        }
        Ast::IdentPath(g) => println!("{tab_spc} {}", g.from_source(txt)),
        // _ => unreachable!(),
    }
}
