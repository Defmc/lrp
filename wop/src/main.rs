use std::fs;

use logos::Logos;
use lrp::Token;
use wop::{Ast, Gramem, Meta};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap();
    let file = fs::read_to_string(path)?;
    let mut copy = Vec::new();

    let lexer = wop::Sym::lexer(&file).spanned().map(|(m, s)| {
        println!("\"{}\" ({m:?}) [{s:?}]", &file[s.clone()]);
        copy.push(&file[s.clone()]);
        Token::new(Meta::new(Ast::Token(m), (s.start, s.end)), m)
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
    //println!("{:#?}", dfa.items[0]);
    print_nested(&dfa.items[0], "", 0, &file);
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
        "{}{prefix}{:?}: \x1B[1;33m{:?}\x1B[0;m",
        TAB_C.repeat(lvl),
        tok.ty,
        txt.get(tok.item.start..tok.item.end).unwrap()
    );
    let lvl = lvl + 1;

    match &tok.item.item {
        Ast::Token(_) => (),
        Ast::EntryPoint(g) => print_nested(g.as_ref(), "", lvl, txt),
        Ast::Program(gs) => print_iter_nested(gs.iter(), "", lvl, txt),
        Ast::Declaration(g) => print_nested(g.as_ref(), "", lvl, txt),
        Ast::TokenDecl(g, h) => {
            print_nested(g.as_ref(), "", lvl, txt);
            print_nested(h.as_ref(), "", lvl, txt);
        }
        Ast::IdentPath(gs) => print_iter_nested(gs.iter(), "", lvl, txt),
        Ast::UseDecl(g) => print_nested(g.as_ref(), "", lvl, txt),
        Ast::AssignOp(op) => println!("{}{op:?}", TAB_C.repeat(lvl)),
        Ast::AttrPrefix(ss) => {
            let tabs = TAB_C.repeat(lvl);
            ss.iter().for_each(|s| println!("{tabs}{s:?}"));
        }
        Ast::AttrSuffix(s) => println!("{}{s:?}", TAB_C.repeat(lvl)),
        Ast::VarPipe(s) => println!("{}{s:?}", TAB_C.repeat(lvl)),
        Ast::TypeDecl(g) => print_nested(g.as_ref(), "", lvl, txt),
        Ast::ElmBase(gs) => print_iter_nested(gs.iter(), "", lvl, txt),
        Ast::Elm(g, h, j) => {
            let tabs = TAB_C.repeat(lvl);
            if let Some(g) = g {
                print_nested(g.as_ref(), "", lvl, txt);
            } else {
                println!("{tabs}none");
            }
            print_nested(h.as_ref(), "", lvl, txt);
            if let Some(j) = j {
                print_nested(j.as_ref(), "", lvl, txt);
            } else {
                println!("{tabs}none");
            }
        }
        Ast::Prod(v) => {
            for (tk, opt) in v {
                print_nested(&tk, "", lvl, txt);
                if let Some(opt) = opt {
                    print_nested(&opt, "", lvl, txt);
                }
            }
        }
        Ast::RulePipeRepeater(gs) => print_iter_nested(gs.iter(), "", lvl, txt),
        Ast::RulePipe(gs) => print_iter_nested(gs.iter(), "", lvl, txt),
        Ast::RuleDecl(gs) => print_iter_nested(gs.iter(), "", lvl, txt),
    }
}
