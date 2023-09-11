use std::fs;

use logos::Logos;
use lrp::{Clr, Parser, Token};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap();
    let file = fs::read_to_string(path)?;
    let mut copy = Vec::new();

    let lexer = wop::Sym::lexer(&file).spanned().map(|(m, s)| {
        println!("\"{}\" ({m:?}) [{s:?}]", &file[s.clone()]);
        copy.push(&file[s.clone()]);
        Token::empty(m)
    });
    let clr = Clr::new(wop::grammar());

    let mut dfa = clr.simple_dfa(lexer);

    match dfa.trace(|_| ()) {
        Err(e) => {
            println!("FATAL: {e}");
            println!("source:");
            copy.into_iter().for_each(|a| print!("{a} "));
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "impossible to parse").into())
        }
        Ok(p) => {
            println!("{p:?}");
            Ok(())
        }
    }
}
