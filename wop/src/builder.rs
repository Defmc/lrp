use crate::{Ast, Gramem, RulePipe, Sym};
use std::{collections::HashMap, fmt::Write, str::FromStr};

pub type SrcRef = lrp::Span;

pub const GRAMMAR_LINTS: &str = "unused_imports, clippy::enum_glob_use";
pub const REDUCTOR_LINTS: &str =
    "non_snake_case, clippy::enum_glob_use, unused_braces, unused_imports";

#[derive(Debug, Default)]
pub struct Builder {
    pub aliases: HashMap<String, SrcRef>,
    pub rules: HashMap<String, RuleBuild>,
    pub imports: Vec<SrcRef>,
}

#[derive(Debug, Clone)]
pub struct ItemAlias {
    pub alias: SrcRef,
    pub optional: Option<bool>,
    pub index: usize,
    pub final_index: Option<usize>,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct ProductionBuild {
    pub production: Vec<SrcRef>,
    pub codeblock: SrcRef,
    /// return type of `self.codeblock`
    pub ty: SrcRef,
    pub aliases: Vec<ItemAlias>,
}

pub type RuleBuild = Vec<ProductionBuild>;
impl ItemAlias {
    pub fn dump(&self, src: &str) -> String {
        let mut s = String::new();
        self.write(&mut s, src).unwrap();
        s
    }

    pub fn write(&self, out: &mut impl Write, src: &str) -> Result<(), std::fmt::Error> {
        if !self.active {
            return Ok(());
        }
        let alias = self.alias.from_source(src);
        let index = if let Some(findex) = self.final_index {
            format!("{:?}", (self.index..findex))
        } else {
            self.index.to_string()
        };
        match self.optional {
            Some(true) => writeln!(out, "let {alias} = Some(&toks[{index}]);"),
            Some(false) => writeln!(out, "let {alias} = None;"),
            None => writeln!(out, "let {alias} = &toks[{index}];"),
        }
    }
}

impl Builder {
    #[must_use]
    pub fn get_program_instructions(ast: &Gramem) -> &Vec<Gramem> {
        if let Ast::Program(p) = &ast.item.item {
            p
        } else {
            unreachable!();
        }
    }

    pub fn process(&mut self, ast: &Gramem, src: &str) {
        let program = Self::get_program_instructions(ast);
        for decl in program {
            match &decl.item.item {
                Ast::RuleDecl((rule_ident, rule_ty, rule)) => {
                    self.rule_decl(*rule_ident, *rule_ty, rule, src);
                }
                Ast::Import(decl) => self.use_decl(*decl),
                Ast::Alias(tk, alias) => self.token_decl(*tk, *alias, src),
                c => unreachable!("unexpected {c:?} in code builder"),
            }
        }
    }

    fn rule_decl(&mut self, rule_ident: SrcRef, rule_ty: SrcRef, rule: &[RulePipe], src: &str) {
        let mut rules = Vec::new();
        for prod in rule {
            assert_ne!(
                prod.1.from_source(src),
                "",
                "missing code block for {:?}",
                rule_ident.from_source(src)
            );
            let base = ProductionBuild {
                production: Vec::new(),
                codeblock: prod.1,
                ty: rule_ty,
                aliases: Vec::new(),
            };
            rules.extend(self.get_production(&[base], &prod.0, src));
        }

        assert!(
            self.rules
                .insert(rule_ident.from_source(src).to_string(), rules)
                .is_none(),
            "rule {} was already defined",
            rule_ident.from_source(src)
        );
    }

    fn get_production(&self, origin: &[ProductionBuild], pipe: &[Gramem], src: &str) -> RuleBuild {
        if pipe.is_empty() {
            origin.to_vec()
        } else {
            let mut out = Vec::new();
            for origin in origin {
                out.extend(self.get_from_single_production(origin, pipe, src));
            }
            out
        }
    }

    fn get_from_single_production(
        &self,
        origin: &ProductionBuild,
        pipe: &[Gramem],
        src: &str,
    ) -> RuleBuild {
        let mut prod = origin.clone();
        for (i, g) in pipe.iter().enumerate() {
            let (item, is_optional, alias) = if let Ast::RuleItem(ref i, o, a) = g.item.item {
                (i.as_ref(), o, a)
            } else {
                unreachable!()
            };
            let get_definition = |should_have: bool| {
                self.aliases
                    .get(item.item.span.from_source(src))
                    .unwrap_or_else(|| {
                        if should_have {
                            eprintln!(
                                "using literal string for {} (maybe is missing an alias?)",
                                item.item.span.from_source(src)
                            );
                        }
                        &item.item.span
                    })
            };
            match item.ty {
                Sym::StrLit => prod.production.push(*get_definition(true)),
                Sym::IdentPath => prod.production.push(*get_definition(true)),
                Sym::Rule => match item.item.item {
                    Ast::Rule(ref variants) => {
                        let mut prods = Vec::new();
                        for (variant, a) in variants {
                            assert_eq!(
                                a,
                                &SrcRef::new(0, 0),
                                "sub-rules like {} shouldn't have a codeblock",
                                g.item.span.from_source(src)
                            );
                            let news = self.get_from_single_production(&prod, variant, src);
                            prods.extend(news);
                        }
                        if i < pipe.len() {
                            prods = self.get_production(&prods, &pipe[i + 1..], src);
                        }
                        return prods;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!("{item:?}"),
            }
            if is_optional {
                let mut ignored_prod = prod.clone();
                ignored_prod.production.pop();
                if i < pipe.len() {
                    return self.get_production(&[prod, ignored_prod], &pipe[i + 1..], src);
                } else {
                    return vec![prod, ignored_prod];
                }
            }
        }
        vec![prod]
    }

    fn token_decl(&mut self, tk: SrcRef, alias: SrcRef, src: &str) {
        assert!(
            self.aliases
                .insert(tk.from_source(src).to_string(), alias)
                .is_none(),
            "overriding an already defined alias: {} to {} ",
            tk.from_source(src),
            alias.from_source(src),
        );
    }

    fn use_decl(&mut self, decl: SrcRef) {
        self.imports.push(decl);
    }

    /// Returns an expressions that returns a `RuleMap`
    /// # Panics
    /// Never.
    #[must_use]
    pub fn dump_grammar(&self, src: &str) -> String {
        let mut out = "{\n".to_string();
        self.imports.iter().for_each(|i| {
            writeln!(out, "\tuse {};", i.from_source(src)).unwrap();
        });
        writeln!(out, "\tlet mut map = lrp::RuleMap::new();").unwrap();
        for (r_name, impls) in &self.rules {
            Self::write_rule(&mut out, r_name, &impls, src);
        }
        out.push_str("\n\tmap\n}");
        out
    }

    /// Writes the rule production
    /// # Panics
    /// Never.
    pub fn write_rule(out: &mut String, name: &str, prods: &RuleBuild, src: &str) {
        writeln!(
            out,
            "\tmap.insert({name}, lrp::grammar::Rule::new({name}, vec!["
        )
        .unwrap();
        for prod in prods {
            out.push_str("\t\tvec![");
            for gramem in &prod.production {
                write!(out, "{}, ", gramem.from_source(src)).unwrap();
            }
            out.push_str("],\n");
        }
        out.push_str("\n\t]));\n");
    }

    /// Dumps an expression that returns a `ReductMap`
    /// # Panics
    /// Never.
    #[must_use]
    pub fn dump_reductor(&self, src: &str) -> String {
        let mut out = "{\n".to_string();
        self.imports.iter().for_each(|i| {
            writeln!(out, "\tuse {};", i.from_source(src)).unwrap();
        });
        writeln!(out, "\tlet mut map = lrp::ReductMap::new();\n").unwrap();
        for (r_name, prods) in &self.rules {
            Self::write_rule_reduction(&mut out, r_name, prods, src);
        }

        out.push_str("\tmap\n}");
        out
    }

    pub fn write_rule_reduction(out: &mut String, name: &str, prods: &RuleBuild, src: &str) {
        let ty = prods[0].ty.from_source(src);
        for (i, prod) in prods.iter().enumerate() {
            let mut item_aliases = String::new();
            for alias in &prod.aliases {
                alias.write(&mut item_aliases, src).unwrap();
            }
            writeln!(
                    out,
                    "\tfn lrp_wop_{name}_{i}(toks: &[Gramem]) -> lrp::Meta<{ty}> {{\n\t\tlrp::Meta::new({{ {item_aliases} {}}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))\n\t}}",
                    prod.codeblock.from_source(src).strip_prefix("->").unwrap().strip_suffix('%').unwrap()
                )
                .unwrap();
        }
        write!(out, "\tmap.insert({name}, vec![").unwrap();
        for (i, _) in prods.iter().enumerate() {
            write!(out, "lrp_wop_{name}_{i}, ").unwrap();
        }
        out.push_str("\t]);\n\n");
    }
}

impl FromStr for Builder {
    type Err = lrp::Error<Sym>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lexer = crate::lexer(s);
        let mut dfa = crate::build_parser(lexer);
        dfa.start()?;
        let mut builder = Self::default();
        builder.process(&dfa.items[0], s);
        Ok(builder)
    }
}
