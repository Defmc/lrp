use std::fs;

use logos::Logos;
use lrp::Token;
use wop::{Ast, Meta};

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
    res
}
