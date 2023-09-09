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
    map.insert(Sym::Program, vec![program_rec, program_rec]);
    map.insert(Sym::Declaration, vec![decl, decl, decl]);
    map.insert(Sym::TokenDecl, vec![tokendecl, tokendecl]);
    map.insert(Sym::IdentPath, vec![identpath]);
    map.insert(Sym::UseDecl, vec![usedecl]);
    map.insert(Sym::AssignOp, vec![assignop]);
    map
}

fn entry_point(program: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let program = program[0].item.clone();
    let span = (program.start, program.end);
    Meta::new(program.item, span)
}

fn program_rec(expressions: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = expressions.first().unwrap().item.start;
    let end = expressions.last().unwrap().item.end;
    let program = expressions.into_iter().cloned().collect();
    Meta::new(Ast::Program(program), (start, end))
}

fn decl(decl: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (decl[0].item.start, decl[0].item.end);
    Meta::new(Ast::Declaration(decl[0].clone().into()), span)
}

fn tokendecl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let token = toks[1].clone();
    let ident = toks[2].clone();
    let span = (token.item.start, ident.item.end);
    Meta::new(Ast::TokenDecl(token.into(), ident.into()), span)
}

fn identpath(path: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = path.first().unwrap().item.start;
    let end = path.last().unwrap().item.end;
    let program = path.into_iter().cloned().collect();
    Meta::new(Ast::Program(program), (start, end))
}

fn usedecl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[1].item.end);
    Meta::new(Ast::UseDecl(toks[1].clone().into()), span)
}

fn assignop(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[0].item.end);
    Meta::new(Ast::AssignOp(toks[0].clone().into()), span)
}
