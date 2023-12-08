use crate::{Ast, Gramem, Meta, Sym};
use lrp::{ReductMap, Span};

pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
    // pub type ReductFn<T, M> = fn(&[Token<T, M>]) -> T;
    // pub type ReductMap<T, M> = Map<M, Vec<ReductFn<T, M>>>;

    let mut map = ReductMap::new();
    map.insert(Sym::EntryPoint, vec![entry_point]);
    map.insert(
        Sym::Program,
        vec![
            program_extend,
            program_extend,
            program_extend,
            program_origin,
            program_origin,
            program_origin,
        ],
    );
    map.insert(Sym::IdentPath, vec![ident_path_extend, ident_path_origin]);
    map.insert(Sym::Import, vec![import, import]);
    map.insert(Sym::Alias, vec![alias; 2]);
    map.insert(Sym::RulePipe, vec![rule_pipe_extend, rule_pipe]);
    map.insert(Sym::RuleDecl, vec![rule_decl]);
    map.insert(Sym::Rule, vec![rule_extend, rule_origin]);
    map.insert(Sym::RuleItem, vec![rule_item, rule_item]);
    map
}

fn entry_point(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::Program);
    Meta::new(
        Ast::EntryPoint(Box::new(toks[0].clone())),
        toks[0].item.span,
    )
}

fn program_origin(toks: &[Gramem]) -> Meta<Ast> {
    let first = toks[0].clone();
    debug_assert!(matches!(first.ty, Sym::Rule | Sym::Alias | Sym::Import));
    debug_assert_eq!(toks[1].ty, Sym::Sc);
    Meta::new(Ast::Program(toks[..1].to_vec()), toks[0].item.span)
}

fn program_extend(toks: &[Gramem]) -> Meta<Ast> {
    let program = &toks[0];
    debug_assert_eq!(toks[0].ty, Sym::Program);
    let extension = toks[1].clone();
    debug_assert!(matches!(
        extension.ty,
        Sym::RuleDecl | Sym::Alias | Sym::Import
    ),);
    debug_assert_eq!(toks[2].ty, Sym::Sc);
    let mut program_vec = match program.item.item {
        Ast::Program(ref v) => v.clone(),
        _ => unreachable!(),
    };
    let span = Span::new(program.item.span.start, extension.item.span.end);
    program_vec.push(extension);

    Meta::new(Ast::Program(program_vec), span)
}

fn ident_path_origin(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::Ident);
    Meta::new(Ast::IdentPath(toks[0].item.span), toks[0].item.span)
}

fn ident_path_extend(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::IdentPath);
    let ip = toks[0].clone();
    debug_assert_eq!(toks[1].ty, Sym::PathAccess);
    debug_assert_eq!(toks[2].ty, Sym::Ident);
    let extension = toks[2].clone();
    let span = Span::new(ip.item.span.start, extension.item.span.end);
    Meta::new(Ast::IdentPath(span), span)
}

fn import(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::UseWord);
    debug_assert_eq!(toks[1].ty, Sym::IdentPath);
    let end = if let Some(tk) = toks.get(2) {
        debug_assert_eq!(tk.ty, Sym::PathAccess);
        debug_assert_eq!(toks[3].ty, Sym::Glob);
        toks[3].item.span.end
    } else {
        toks[1].item.span.end
    };

    Meta::new(
        Ast::Import(Span::new(toks[1].item.span.start, end)),
        Span::new(toks[0].item.span.start, end),
    )
}

fn alias(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::AliasWord);
    debug_assert!(matches!(toks[1].ty, Sym::Ident | Sym::StrLit));
    debug_assert_eq!(toks[2].ty, Sym::IdentPath);
    Meta::new(
        Ast::Alias(toks[1].item.span, toks[2].item.span),
        Span::new(toks[0].item.span.start, toks[2].item.span.end),
    )
}

fn rule_pipe_extend(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::RulePipe);
    debug_assert_eq!(toks[1].ty, Sym::RuleItem);
    let mut v = match toks[0].item.item {
        Ast::RulePipe(ref v) => v.clone(),
        _ => unreachable!(),
    };
    v.push(toks[1].clone());
    Meta::new(
        Ast::RulePipe(v),
        Span::new(toks[0].item.span.start, toks[1].item.span.end),
    )
}
fn rule_pipe(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::RuleItem);
    Meta::new(Ast::RulePipe(toks[..1].to_vec()), toks[0].item.span)
}

fn rule_origin(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::RulePipe);
    debug_assert_eq!(toks[1].ty, Sym::CodeBlock);
    let Ast::RulePipe(ref prod) = toks[0].item.item else {
        unreachable!()
    };
    Meta::new(
        Ast::Rule(vec![(prod.clone(), toks[1].item.span)]),
        toks[0].item.span,
    )
}

fn rule_extend(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::Rule);
    let mut rule_vec = match toks[0].item.item {
        Ast::Rule(ref vv) => vv.clone(),
        _ => unreachable!(),
    };
    debug_assert_eq!(toks[1].ty, Sym::Pipe);
    debug_assert_eq!(toks[2].ty, Sym::RulePipe);
    debug_assert_eq!(toks[3].ty, Sym::CodeBlock);
    match toks[2].item.item {
        Ast::RulePipe(ref v) => rule_vec.push((v.clone(), toks[3].item.span)),
        _ => unreachable!(),
    };
    Meta::new(
        Ast::Rule(rule_vec),
        Span::new(toks[0].item.span.start, toks[1].item.span.end),
    )
}

fn rule_decl(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert_eq!(toks[0].ty, Sym::IdentPath);
    let ident = toks[0].item.item.get_src_ref().unwrap();
    debug_assert_eq!(toks[1].ty, Sym::TwoDots);
    debug_assert_eq!(toks[2].ty, Sym::IdentPath);
    let ty = toks[2].item.item.get_src_ref().unwrap();
    debug_assert_eq!(toks[3].ty, Sym::Assign);
    debug_assert_eq!(toks[4].ty, Sym::Rule);
    let rule_vec = match toks[4].item.item {
        Ast::Rule(ref v) => v.clone(),
        _ => unreachable!(),
    };
    Meta::new(
        Ast::RuleDecl((ident, ty, rule_vec)),
        Span::new(toks[0].item.span.start, toks[4].item.span.end),
    )
}

fn rule_item(toks: &[Gramem]) -> Meta<Ast> {
    debug_assert!(matches!(toks[0].ty, Sym::IdentPath | Sym::StrLit));
    Meta::new(Ast::RuleItem(toks[0].item.span), toks[0].item.span)
}
