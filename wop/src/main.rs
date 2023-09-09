use std::fs;

use logos::Logos;
use lrp::{Clr, Parser, Token};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().skip(1).next().unwrap();
    let file = fs::read_to_string(path)?;

    let mut copy = Vec::new();

    let lexer = wop::Sym::lexer(&file).spanned().map(|(m, s)| {
        println!("\"{}\" ({m:?}) [{s:?}]", &file[s.clone()]);
        copy.push(&file[s.clone()]);
        Token::empty(m)
    });
    let clr = Clr::new(wop::grammar());

    let mut dfa = clr.simple_dfa(lexer);

    if let Err(e) = dfa.trace(|st| /* println!("{:?}", st.stack_fmt()) */()) {
        println!("FATAL: {e}");
        println!("source:");
        copy.into_iter().for_each(|a| print!("{a} "));
    }

    Ok(())
}
