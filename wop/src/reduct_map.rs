use crate::{Ast, Meta, Sym};
use lrp::{ReductMap, Token};

pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
    // pub type ReductFn<T, M> = fn(&[Token<T, M>]) -> T;
    // pub type ReductMap<T, M> = Map<M, Vec<ReductFn<T, M>>>;

    let mut map = ReductMap::new();
    map.insert(Sym::EntryPoint, vec![entry_point]);
    map.insert(Sym::Program, vec![program_rec, program]);
    map.insert(Sym::Declaration, vec![decl; 3]);
    map.insert(Sym::TokenDecl, vec![token_decl; 2]);
    map.insert(Sym::IdentPath, vec![ident_path_rec, ident_path]);
    map.insert(Sym::UseDecl, vec![use_decl]);
    map.insert(Sym::AssignOp, vec![assign_op; 4]);
    map.insert(
        Sym::AttrPrefix,
        vec![attr_prefix, attr_prefix, attr_prefix_rec, attr_prefix_rec],
    );
    map.insert(Sym::AttrSuffix, vec![attr_suffix; 3]);
    map.insert(Sym::VarPipe, vec![var_pipe]);
    map.insert(Sym::TypeDecl, vec![type_decl]);
    map.insert(Sym::ElmBase, vec![elm_base; 6]);
    map.insert(
        Sym::Elm,
        vec![elm_with_all, elm_with_suffix, elm_with_prefix, elm],
    );
    map.insert(Sym::Prod, vec![prod_rec, prod_expr_rec, prod_expr, prod]);
    map.insert(
        Sym::RulePipeRepeater,
        vec![rule_pipe_repeater_rec, rule_pipe_repeater],
    );
    map.insert(Sym::RulePipe, vec![rule_pipe; 4]);
    map.insert(Sym::RuleDecl, vec![rule_decl; 2]);
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

fn token_decl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let token = toks[1].clone();
    let ident = toks[2].clone();
    let span = (token.item.start, ident.item.end);
    Meta::new(Ast::TokenDecl(token.into(), ident.into()), span)
}

fn ident_path(path: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = path.first().unwrap().item.start;
    let end = path.last().unwrap().item.end;
    let program = path.into_iter().cloned().collect();
    Meta::new(Ast::IdentPath(program), (start, end))
}

fn use_decl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[1].item.end);
    Meta::new(Ast::UseDecl(toks[1].clone().into()), span)
}

fn assign_op(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[0].item.end);
    Meta::new(Ast::AssignOp(toks[0].clone().into()), span)
}

fn attr_prefix(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks
        .into_iter()
        .map(|t| Meta::new(t.item.item.as_sym(), (t.item.start, t.item.end)))
        .collect();
    Meta::new(Ast::AttrPrefix(program), (start, end))
}

fn attr_suffix(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[0].item.end);
    Meta::new(
        Ast::AttrSuffix(
            toks[0]
                .item /* TODO: couldn't it be `ty` directly */
                .item
                .as_sym(),
        ),
        span,
    )
}

fn var_pipe(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[0].item.end);
    Meta::new(
        Ast::VarPipe(
            toks[0]
                .item /* TODO: couldn't it be `ty` directly */
                .item
                .as_sym(),
        ),
        span,
    )
}

fn type_decl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[0].item.end);
    Meta::new(Ast::TypeDecl(toks[0].clone().into()), span)
}

fn elm_base(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks.into_iter().cloned().collect();
    Meta::new(Ast::ElmBase(program), (start, end))
}

fn elm(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks.into_iter().cloned().collect();
    Meta::new(Ast::Elm(program), (start, end))
}

fn prod(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks.into_iter().cloned().collect();
    Meta::new(Ast::Prod(program), (start, end))
}

fn rule_pipe_repeater(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks.into_iter().cloned().collect();
    Meta::new(Ast::RulePipeRepeater(program), (start, end))
}

fn rule_pipe(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks.into_iter().cloned().collect();
    Meta::new(Ast::RulePipe(program), (start, end))
}

fn rule_decl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks.into_iter().cloned().collect();
    Meta::new(Ast::RuleDecl(program), (start, end))
}
