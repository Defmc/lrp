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
    debug_assert!(matches!(program[0].ty, Sym::Program));
    let program = program[0].item.clone();
    let span = (program.start, program.end);
    Meta::new(program.item, span)
}

fn program_rec(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let mut program = toks[0].item.clone();
    let start = program.start;
    let end = toks[1].item.end;
    match program.item {
        Ast::Program(ref mut vec) => {
            debug_assert!(matches!(toks[1].ty, Sym::Declaration));
            vec.push(toks[1].clone())
        }
        _ => unreachable!(),
    };
    Meta::new(program.item, (start, end))
}

fn program(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    debug_assert!(matches!(toks[0].ty, Sym::Declaration));
    let program = toks[0].clone();
    let span = (program.item.start, program.item.end);
    Meta::new(Ast::Program(vec![program]), span)
}

fn decl(decl: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    debug_assert!(matches!(
        decl[0].ty,
        Sym::TokenDecl | Sym::UseDecl | Sym::RuleDecl
    ));
    let span = (decl[0].item.start, decl[0].item.end);
    Meta::new(Ast::Declaration(decl[0].clone().into()), span)
}

fn token_decl(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let token = toks[1].clone();
    let ident = toks[2].clone();
    let span = (token.item.start, ident.item.end);
    Meta::new(Ast::TokenDecl(token.into(), ident.into()), span)
}

fn ident_path_rec(path: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let mut program = path[0].item.clone();
    let start = program.start;
    let end = path[2].item.end;
    match program.item {
        Ast::IdentPath(ref mut vec) => {
            debug_assert!(matches!(path[2].ty, Sym::Ident));
            vec.push(path[2].clone())
        }
        _ => unreachable!(),
    };
    Meta::new(program.item, (start, end))
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

fn attr_prefix_rec(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let mut program = toks[1].item.clone();
    let end = program.end;
    let start = toks[0].item.start;
    match program.item {
        Ast::AttrPrefix(ref mut vec) => {
            debug_assert!(matches!(toks[1].ty, Sym::MetaAttr | Sym::BoxAttr));
            let sym = toks[0].ty;
            let span = (toks[0].item.start, toks[0].item.end);
            vec.push(Meta::new(sym, span))
        }
        _ => unreachable!(),
    };
    Meta::new(program.item, (start, end))
}

fn attr_prefix(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let start = toks.first().unwrap().item.start;
    let end = toks.last().unwrap().item.end;
    let program = toks
        .into_iter()
        .map(|t| Meta::new(t.ty, (t.item.start, t.item.end)))
        .collect();
    Meta::new(Ast::AttrPrefix(program), (start, end))
}

fn attr_suffix(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[0].item.end);
    Meta::new(Ast::AttrSuffix(toks[0].ty), span)
}

fn var_pipe(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let span = (toks[0].item.start, toks[0].item.end);
    Meta::new(Ast::VarPipe(toks[0].ty), span)
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
    let elm = toks[0].clone();
    let span = (elm.item.start, elm.item.end);
    Meta::new(Ast::Elm(None, elm.into(), None), span)
}

fn elm_with_prefix(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let prefix = toks[0].clone();
    let elm = toks[1].clone();
    let span = (prefix.item.start, elm.item.end);
    Meta::new(Ast::Elm(Some(prefix.into()), elm.into(), None), span)
}

fn elm_with_suffix(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let elm = toks[0].clone();
    let suffix = toks[1].clone();
    let span = (elm.item.start, suffix.item.end);
    Meta::new(Ast::Elm(None, elm.into(), Some(suffix.into())), span)
}

fn elm_with_all(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let prefix = toks[0].clone();
    let elm = toks[1].clone();
    let suffix = toks[2].clone();
    let span = (prefix.item.start, suffix.item.end);
    Meta::new(
        Ast::Elm(Some(prefix.into()), elm.into(), Some(suffix.into())),
        span,
    )
}

fn prod_rec(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let mut rec = toks[0].item.clone();
    let start = rec.start;
    let elm = toks[1].clone();
    let end = elm.item.end;
    match rec.item {
        Ast::Prod(ref mut vec) => {
            vec.push((elm, None));
        }
        _ => unreachable!(),
    }
    Meta::new(rec.item, (start, end))
}
fn prod(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let elm = toks[0].clone();
    let start = elm.item.start;
    let end = elm.item.end;
    Meta::new(Ast::Prod(vec![(elm, None)]), (start, end))
}

fn prod_expr_rec(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let mut rec = toks[0].item.clone();
    let start = rec.start;
    let elm = toks[1].clone();
    let code_expr = toks[2].clone();
    let end = code_expr.item.end;
    match rec.item {
        Ast::Prod(ref mut vec) => {
            vec.push((elm, Some(code_expr.into())));
        }
        _ => unreachable!(),
    }
    Meta::new(rec.item, (start, end))
}

fn prod_expr(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let elm = toks[0].clone();
    let start = elm.item.start;
    let code_expr = toks[1].clone();
    let end = code_expr.item.end;
    Meta::new(Ast::Prod(vec![(elm, Some(code_expr.into()))]), (start, end))
}

fn rule_pipe_repeater_rec(toks: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let mut program = toks[0].item.clone();
    let start = program.start;
    let end = toks[1].item.end;
    match program.item {
        Ast::RulePipeRepeater(ref mut vec) => {
            debug_assert!(matches!(toks[1].ty, Sym::Prod));
            vec.push(toks[1].clone())
        }
        _ => unreachable!(),
    };
    Meta::new(program.item, (start, end))
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
