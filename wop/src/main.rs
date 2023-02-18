use std::{fs, io};

use logos::Logos;
use lrp::{Parser, Slr, Token};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().skip(1).next().unwrap();
    let file = fs::read_to_string(path)?;

    let lexer = wop::Sym::lexer(&file).map(|m| Token::empty(m));
    let slr = Slr::new(wop::grammar());

    let mut dfa = slr.simple_dfa(lexer);

    if let Err(e) = dfa.trace(|st| println!("{:?}", st.stack_fmt())) {
        println!("FATAL: {e}");
    }

    Ok(())
}
