
use crate::{Ast, Gramem, Meta, Sym};
use lrp::{Grammar, ReductMap, Span};

#[allow(unused_imports, clippy::enum_glob_use)]
#[must_use]
pub fn grammar() -> Grammar<Sym> {
    Grammar::new(Sym::EntryPoint, {
	use crate::Sym::*;
	use crate::Ast;
	let mut map = lrp::RuleMap::new();
	map.insert(RuleItem, lrp::grammar::Rule::new(RuleItem, vec![
		vec![IdentPath, Optional, TwoDots, Clone, Ident, ],
		vec![IdentPath, Optional, TwoDots, Ident, ],
		vec![IdentPath, Optional, ],
		vec![IdentPath, TwoDots, Clone, Ident, ],
		vec![IdentPath, TwoDots, Ident, ],
		vec![IdentPath, ],
		vec![StrLit, Optional, TwoDots, Clone, Ident, ],
		vec![StrLit, Optional, TwoDots, Ident, ],
		vec![StrLit, Optional, ],
		vec![StrLit, TwoDots, Clone, Ident, ],
		vec![StrLit, TwoDots, Ident, ],
		vec![StrLit, ],
		vec![OpenParen, Rule, CloseParen, Optional, TwoDots, Clone, Ident, ],
		vec![OpenParen, Rule, CloseParen, Optional, TwoDots, Ident, ],
		vec![OpenParen, Rule, CloseParen, Optional, ],
		vec![OpenParen, Rule, CloseParen, TwoDots, Clone, Ident, ],
		vec![OpenParen, Rule, CloseParen, TwoDots, Ident, ],
		vec![OpenParen, Rule, CloseParen, ],

	]));
	map.insert(IdentPath, lrp::grammar::Rule::new(IdentPath, vec![
		vec![IdentPath, PathAccess, Ident, ],
		vec![Ident, ],

	]));
	map.insert(EntryPoint, lrp::grammar::Rule::new(EntryPoint, vec![
		vec![Program, ],

	]));
	map.insert(Rule, lrp::grammar::Rule::new(Rule, vec![
		vec![Rule, Pipe, RulePipe, CodeBlock, ],
		vec![Rule, Pipe, RulePipe, ],
		vec![RulePipe, CodeBlock, ],
		vec![RulePipe, ],

	]));
	map.insert(RulePipe, lrp::grammar::Rule::new(RulePipe, vec![
		vec![RulePipe, RuleItem, ],
		vec![RuleItem, ],

	]));
	map.insert(Import, lrp::grammar::Rule::new(Import, vec![
		vec![UseWord, IdentPath, PathAccess, Glob, ],
		vec![UseWord, IdentPath, ],

	]));
	map.insert(Alias, lrp::grammar::Rule::new(Alias, vec![
		vec![AliasWord, Ident, IdentPath, ],
		vec![AliasWord, StrLit, IdentPath, ],

	]));
	map.insert(Program, lrp::grammar::Rule::new(Program, vec![
		vec![Program, Import, Sc, ],
		vec![Program, Alias, Sc, ],
		vec![Program, RuleDecl, Sc, ],
		vec![Alias, Sc, ],
		vec![Import, Sc, ],
		vec![RuleDecl, Sc, ],

	]));
	map.insert(RuleDecl, lrp::grammar::Rule::new(RuleDecl, vec![
		vec![IdentPath, TwoDots, IdentPath, Assign, Rule, ],

	]));

	map
}, Sym::Eof)
}

#[allow(non_snake_case, clippy::enum_glob_use, unused_braces, unused_imports, unused_assignments, clippy::unnecessary_literal_unwrap)]
pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
	use crate::Sym::*;
	use crate::Ast;
	let mut map = lrp::ReductMap::new();

	fn lrp_wop_RuleItem_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let opt = Some(toks[1].clone());
let cl = Some(toks[3].clone());
let id = Some(toks[4].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let opt = Some(toks[1].clone());
let cl = Some(toks[3].clone());
let id = Some(toks[3].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_2(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let opt = Some(toks[1].clone());
let mut cl = Some(&toks[0]); cl = None; let cl = cl;
let mut id = Some(&toks[0]); id = None; let id = id;
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_3(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let cl = Some(toks[2].clone());
let id = Some(toks[3].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_4(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let cl = Some(toks[2].clone());
let id = Some(toks[2].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_5(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let mut cl = Some(&toks[0]); cl = None; let cl = cl;
let mut id = Some(&toks[0]); id = None; let id = id;
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_6(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let opt = Some(toks[1].clone());
let cl = Some(toks[3].clone());
let id = Some(toks[4].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_7(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let opt = Some(toks[1].clone());
let cl = Some(toks[3].clone());
let id = Some(toks[3].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_8(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let opt = Some(toks[1].clone());
let mut cl = Some(&toks[0]); cl = None; let cl = cl;
let mut id = Some(&toks[0]); id = None; let id = id;
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_9(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let cl = Some(toks[2].clone());
let id = Some(toks[3].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_10(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let cl = Some(toks[2].clone());
let id = Some(toks[2].clone());
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_11(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = Some(toks[0].clone());
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let mut cl = Some(&toks[0]); cl = None; let cl = cl;
let mut id = Some(&toks[0]); id = None; let id = id;
  {
    Ast::RuleItem(Box::new(item.unwrap()), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_12(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let rule = toks[1].clone();
let opt = Some(toks[3].clone());
let cl = Some(toks[5].clone());
let id = Some(toks[6].clone());
  {
    Ast::RuleItem(Box::new(rule), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_13(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let rule = toks[1].clone();
let opt = Some(toks[3].clone());
let cl = Some(toks[5].clone());
let id = Some(toks[5].clone());
  {
    Ast::RuleItem(Box::new(rule), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_14(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let rule = toks[1].clone();
let opt = Some(toks[3].clone());
let mut cl = Some(&toks[0]); cl = None; let cl = cl;
let mut id = Some(&toks[0]); id = None; let id = id;
  {
    Ast::RuleItem(Box::new(rule), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_15(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let rule = toks[1].clone();
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let cl = Some(toks[4].clone());
let id = Some(toks[5].clone());
  {
    Ast::RuleItem(Box::new(rule), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_16(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let rule = toks[1].clone();
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let cl = Some(toks[4].clone());
let id = Some(toks[4].clone());
  {
    Ast::RuleItem(Box::new(rule), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_17(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let rule = toks[1].clone();
let mut opt = Some(toks[0].clone()); opt = None; let opt = opt;
let mut cl = Some(&toks[0]); cl = None; let cl = cl;
let mut id = Some(&toks[0]); id = None; let id = id;
  {
    Ast::RuleItem(Box::new(rule), opt.is_some(), id.map(|id| id.item.span), cl.is_some())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(RuleItem, vec![lrp_wop_RuleItem_0, lrp_wop_RuleItem_1, lrp_wop_RuleItem_2, lrp_wop_RuleItem_3, lrp_wop_RuleItem_4, lrp_wop_RuleItem_5, lrp_wop_RuleItem_6, lrp_wop_RuleItem_7, lrp_wop_RuleItem_8, lrp_wop_RuleItem_9, lrp_wop_RuleItem_10, lrp_wop_RuleItem_11, lrp_wop_RuleItem_12, lrp_wop_RuleItem_13, lrp_wop_RuleItem_14, lrp_wop_RuleItem_15, lrp_wop_RuleItem_16, lrp_wop_RuleItem_17, 	]);

	fn lrp_wop_IdentPath_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let ip = toks[0].clone();
let ext = toks[2].clone();
  {
    let span = Span::new(ip.item.span.start, ext.item.span.end);
    Ast::IdentPath(span)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_IdentPath_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({   { Ast::IdentPath(toks[0].item.span) }}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(IdentPath, vec![lrp_wop_IdentPath_0, lrp_wop_IdentPath_1, 	]);

	fn lrp_wop_EntryPoint_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let p = toks[0].clone();
  { Ast::EntryPoint(Box::new(p)) }}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(EntryPoint, vec![lrp_wop_EntryPoint_0, 	]);

	fn lrp_wop_Rule_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let cb = Some(toks[3].clone());
  {
    let mut rule_vec = match toks[0].item.item {
        Ast::Rule(ref vv) => vv.clone(),
        _ => unreachable!(),
    };
    match toks[2].item.item {
        Ast::RulePipe(ref v) => rule_vec.push((v.clone(), cb.map_or_else(|| Span::new(0, 0), |g| g.item.span))),
        _ => unreachable!(),
    };
    Ast::Rule(rule_vec)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Rule_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let mut cb = Some(toks[0].clone()); cb = None; let cb = cb;
  {
    let mut rule_vec = match toks[0].item.item {
        Ast::Rule(ref vv) => vv.clone(),
        _ => unreachable!(),
    };
    match toks[2].item.item {
        Ast::RulePipe(ref v) => rule_vec.push((v.clone(), cb.map_or_else(|| Span::new(0, 0), |g| g.item.span))),
        _ => unreachable!(),
    };
    Ast::Rule(rule_vec)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Rule_2(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let cb = Some(toks[1].clone());
  {
    let Ast::RulePipe(ref prod) = toks[0].item.item else {
        unreachable!()
    };
    Ast::Rule(vec![(prod.clone(), cb.map_or_else(|| Span::new(0, 0), |g| g.item.span))])
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Rule_3(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let mut cb = Some(toks[0].clone()); cb = None; let cb = cb;
  {
    let Ast::RulePipe(ref prod) = toks[0].item.item else {
        unreachable!()
    };
    Ast::Rule(vec![(prod.clone(), cb.map_or_else(|| Span::new(0, 0), |g| g.item.span))])
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Rule, vec![lrp_wop_Rule_0, lrp_wop_Rule_1, lrp_wop_Rule_2, lrp_wop_Rule_3, 	]);

	fn lrp_wop_RulePipe_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let item = toks[1].clone();
  {
    let mut v = match toks[0].item.item {
        Ast::RulePipe(ref v) => v.clone(),
        _ => unreachable!(),
    };
    v.push(item);
    Ast::RulePipe(v)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RulePipe_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({   {
    Ast::RulePipe(toks[..1].to_vec())
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(RulePipe, vec![lrp_wop_RulePipe_0, lrp_wop_RulePipe_1, 	]);

	fn lrp_wop_Import_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({   {
    Ast::Import(Span::new(toks[1].item.span.start, toks.last().unwrap().item.span.end))
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Import_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({   {
    Ast::Import(Span::new(toks[1].item.span.start, toks.last().unwrap().item.span.end))
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Import, vec![lrp_wop_Import_0, lrp_wop_Import_1, 	]);

	fn lrp_wop_Alias_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({   {
    Ast::Alias(toks[1].item.span, toks[2].item.span)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Alias_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({   {
    Ast::Alias(toks[1].item.span, toks[2].item.span)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Alias, vec![lrp_wop_Alias_0, lrp_wop_Alias_1, 	]);

	fn lrp_wop_Program_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let ext = Some(toks[1].clone());
  { 
    let program = &toks[0];
    let mut program_vec = match program.item.item {
        Ast::Program(ref v) => v.clone(),
        _ => unreachable!(),
    };
    program_vec.push(ext.unwrap());
    Ast::Program(program_vec)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let ext = Some(toks[1].clone());
  { 
    let program = &toks[0];
    let mut program_vec = match program.item.item {
        Ast::Program(ref v) => v.clone(),
        _ => unreachable!(),
    };
    program_vec.push(ext.unwrap());
    Ast::Program(program_vec)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_2(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let ext = Some(toks[1].clone());
  { 
    let program = &toks[0];
    let mut program_vec = match program.item.item {
        Ast::Program(ref v) => v.clone(),
        _ => unreachable!(),
    };
    program_vec.push(ext.unwrap());
    Ast::Program(program_vec)
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_3(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let p = toks[0..1].to_vec();
  { Ast::Program(p) }}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_4(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let p = toks[0..1].to_vec();
  { Ast::Program(p) }}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_5(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({ let p = toks[0..1].to_vec();
  { Ast::Program(p) }}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Program, vec![lrp_wop_Program_0, lrp_wop_Program_1, lrp_wop_Program_2, lrp_wop_Program_3, lrp_wop_Program_4, lrp_wop_Program_5, 	]);

	fn lrp_wop_RuleDecl_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new({   {
    let ident = toks[0].item.item.get_src_ref().unwrap();
    let ty = toks[2].item.item.get_src_ref().unwrap();
    let rule_vec = match toks[4].item.item {
        Ast::Rule(ref v) => v.clone(),
        _ => unreachable!(),
    };
    Ast::RuleDecl((ident, ty, rule_vec))
}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(RuleDecl, vec![lrp_wop_RuleDecl_0, 	]);

	map
}
