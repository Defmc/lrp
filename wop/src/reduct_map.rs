use crate::{Ast, Meta, Sym};
use lrp::{ReductMap, Token};

pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
    // pub type ReductFn<T, M> = fn(&[Token<T, M>]) -> T;
    // pub type ReductMap<T, M> = Map<M, Vec<ReductFn<T, M>>>;
    /* ReductMap::from([(
        Sym,
        vec![
            fn2_with_ref_to_tokens,
            fn3_with_ref_to_tokens,
        ],
    )]); */

    let mut map = ReductMap::new();
    map.insert(Sym::EntryPoint, vec![entry_point]);
    map.insert(Sym::Program, vec![program]);
    map.insert(Sym::Declaration, vec![decl, decl, decl]);
    map.insert(Sym::TokenDecl, vec![tokendecl, tokendecl]);
    map.insert(Sym::IdentPath, vec![identpath]);
    map
}

fn entry_point(program: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let program = program[0].item.clone();
    let (start, end) = (program.start, program.end);
    Meta::new(program.item, (start, end))
}

fn program(expressions: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = expressions.first().unwrap().item.start;
    let end = expressions.last().unwrap().item.end;
    let program = expressions.into_iter().map(|e| e.clone()).collect();
    Meta::new(Ast::Program(program), (start, end))
}

fn decl(decl: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = decl[0].item.start;
    let end = decl[0].item.end;
    Meta::new(Ast::Declaration(decl[0].clone().into()), (start, end))
}

fn tokendecl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let token = toks[1];
    let ident = toks[2];
    let (start, end) = (token.item.start, ident.item.end);
    Meta::new(Ast::TokenDecl(token.into(), ident.into()), (start, end))
}

fn identpath(path: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = path.first().unwrap().item.start;
    let end = path.last().unwrap().item.end;
    let program = path.into_iter().map(|e| e.clone()).collect();
    Meta::new(Ast::Program(program), (start, end))
}
