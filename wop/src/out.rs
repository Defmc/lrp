
use crate::{Ast, Gramem, Meta, Sym};
use lrp::{Grammar, ReductMap, Span};

#[allow(clippy::enum_glob_use)]
#[allow(unused_imports)]
#[must_use]
pub fn grammar() -> Grammar<Sym> {
    Grammar::new(EntryPoint, {
	use crate::Sym::*;
	use crate::Ast;
	let mut map = lrp::RuleMap::new();
	map.insert(Program, lrp::grammar::Rule::new(Program, vec![
		vec![Program, Alias, Sc, ],
		vec![Program, Import, Sc, ],
		vec![Program, RuleDecl, Sc, ],
		vec![Alias, Sc, ],
		vec![Import, Sc, ],
		vec![RuleDecl, Sc, ],

	]));
	map.insert(RuleDecl, lrp::grammar::Rule::new(RuleDecl, vec![
		vec![IdentPath, TwoDots, IdentPath, Assign, Rule, ],

	]));
	map.insert(Import, lrp::grammar::Rule::new(Import, vec![
		vec![UseWord, IdentPath, PathAccess, Glob, ],
		vec![UseWord, IdentPath, ],

	]));
	map.insert(IdentPath, lrp::grammar::Rule::new(IdentPath, vec![
		vec![IdentPath, PathAccess, Ident, ],
		vec![Ident, ],

	]));
	map.insert(Rule, lrp::grammar::Rule::new(Rule, vec![
		vec![Rule, Pipe, RulePipe, CodeBlock, ],
		vec![RulePipe, CodeBlock, ],

	]));
	map.insert(Alias, lrp::grammar::Rule::new(Alias, vec![
		vec![AliasWord, Ident, IdentPath, ],
		vec![AliasWord, StrLit, IdentPath, ],

	]));
	map.insert(RuleItem, lrp::grammar::Rule::new(RuleItem, vec![
		vec![IdentPath, ],
		vec![StrLit, ],

	]));
	map.insert(RulePipe, lrp::grammar::Rule::new(RulePipe, vec![
		vec![RulePipe, RuleItem, ],
		vec![RuleItem, ],

	]));
	map.insert(EntryPoint, lrp::grammar::Rule::new(EntryPoint, vec![
		vec![Program, ],

	]));

	map
}, Eof)
}

#[allow(non_snake_case)]
pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
	use crate::Sym::*;
	use crate::Ast;
	let mut map = lrp::ReductMap::new();

	fn lrp_wop_Alias_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    Ast::Alias(toks[1].item.span, toks[2].item.span)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Alias_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    Ast::Alias(toks[1].item.span, toks[2].item.span)
    }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Alias, vec![lrp_wop_Alias_0, lrp_wop_Alias_1, 	]);

	fn lrp_wop_Import_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    Ast::Import(Span::new(toks[1].item.span.start, toks[3].item.span.end))
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Import_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    Ast::Import(toks[1].item.span)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Import, vec![lrp_wop_Import_0, lrp_wop_Import_1, 	]);

	fn lrp_wop_Program_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( { 
    let program = &toks[0];
    let extension = toks[1].clone();
    let mut program_vec = match program.item.item {
        Ast::Program(ref v) => v.clone(),
        _ => unreachable!(),
    };
    program_vec.push(extension);
    Ast::Program(program_vec)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    let program = &toks[0];
    let extension = toks[1].clone();
    let mut program_vec = match program.item.item {
        Ast::Program(ref v) => v.clone(),
        _ => unreachable!(),
    };
    program_vec.push(extension);
    Ast::Program(program_vec)
    }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_2(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    let program = &toks[0];
    let extension = toks[1].clone();
    let mut program_vec = match program.item.item {
        Ast::Program(ref v) => v.clone(),
        _ => unreachable!(),
    };
    program_vec.push(extension);
    Ast::Program(program_vec)
    }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_3(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( { Ast::Program(toks[..1].to_vec()) }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_4(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( { Ast::Program(toks[..1].to_vec()) }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Program_5(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( { Ast::Program(toks[..1].to_vec()) }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Program, vec![lrp_wop_Program_0, lrp_wop_Program_1, lrp_wop_Program_2, lrp_wop_Program_3, lrp_wop_Program_4, lrp_wop_Program_5, 	]);

	fn lrp_wop_IdentPath_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    let ip = toks[0].clone();
    let extension = toks[2].clone();
    let span = Span::new(ip.item.span.start, extension.item.span.end);
    Ast::IdentPath(span)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_IdentPath_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( { Ast::IdentPath(toks[0].item.span) }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(IdentPath, vec![lrp_wop_IdentPath_0, lrp_wop_IdentPath_1, 	]);

	fn lrp_wop_Rule_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    let mut rule_vec = match toks[0].item.item {
        Ast::Rule(ref vv) => vv.clone(),
        _ => unreachable!(),
    };
    match toks[2].item.item {
        Ast::RulePipe(ref v) => rule_vec.push((v.clone(), toks[3].item.span)),
        _ => unreachable!(),
    };
    Ast::Rule(rule_vec)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_Rule_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    let Ast::RulePipe(ref prod) = toks[0].item.item else {
        unreachable!()
    };
    Ast::Rule(vec![(prod.clone(), toks[1].item.span)])
    }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(Rule, vec![lrp_wop_Rule_0, lrp_wop_Rule_1, 	]);

	fn lrp_wop_RuleDecl_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    let ident = toks[0].item.item.get_src_ref().unwrap();
    let ty = toks[2].item.item.get_src_ref().unwrap();
    let rule_vec = match toks[4].item.item {
        Ast::Rule(ref v) => v.clone(),
        _ => unreachable!(),
    };
    Ast::RuleDecl((ident, ty, rule_vec))
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(RuleDecl, vec![lrp_wop_RuleDecl_0, 	]);

	fn lrp_wop_RulePipe_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    let mut v = match toks[0].item.item {
        Ast::RulePipe(ref v) => v.clone(),
        _ => unreachable!(),
    };
    v.push(toks[1].clone());
    Ast::RulePipe(v)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RulePipe_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    Ast::RulePipe(toks[..1].to_vec())
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(RulePipe, vec![lrp_wop_RulePipe_0, lrp_wop_RulePipe_1, 	]);

	fn lrp_wop_EntryPoint_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( { Ast::EntryPoint(Box::new(toks[0].clone())) }, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(EntryPoint, vec![lrp_wop_EntryPoint_0, 	]);

	fn lrp_wop_RuleItem_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    Ast::RuleItem(toks[0].item.span)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	fn lrp_wop_RuleItem_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
		lrp::Meta::new( {
    Ast::RuleItem(toks[0].item.span)
}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))
	}
	map.insert(RuleItem, vec![lrp_wop_RuleItem_0, lrp_wop_RuleItem_1, 	]);

	map
}
