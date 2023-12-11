use crate::{Ast, Gramem, RulePipe, Sym};
use std::{collections::HashMap, fmt::Write, str::FromStr};

pub type SrcRef = lrp::Span;

pub const GRAMMAR_LINTS: &str = "unused_imports, clippy::enum_glob_use";
pub const REDUCTOR_LINTS: &str =
    "non_snake_case, clippy::enum_glob_use, unused_braces, unused_imports";

#[derive(Debug, Default)]
pub struct Builder {
    pub aliases: HashMap<String, SrcRef>,
    pub rules: HashMap<String, Vec<Vec<SrcRef>>>,
    pub reductors: HashMap<String, (SrcRef, Vec<SrcRef>)>,
    pub item_aliases: HashMap<String, Vec<Vec<ItemAlias>>>,
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
        let mut prods = Vec::new();
        let mut reductors = Vec::new();
        let mut item_aliases = Vec::new();
        for prod in rule {
            assert_ne!(
                prod.1.from_source(src),
                "",
                "missing code block for {:?}",
                rule_ident.from_source(src)
            );
            let (p, r, a) = self.get_complete_rule(prod, src);
            prods.extend(p);
            reductors.extend(r);
            item_aliases.extend(a);
        }

        assert!(
            self.rules
                .insert(rule_ident.from_source(src).to_string(), prods)
                .is_none(),
            "rule {} was already defined",
            rule_ident.from_source(src)
        );
        assert!(
            self.reductors
                .insert(
                    rule_ident.from_source(src).to_string(),
                    (rule_ty, reductors)
                )
                .is_none(),
            "rule {} was already defined",
            rule_ident.from_source(src)
        );
        assert!(
            self.item_aliases
                .insert(rule_ident.from_source(src).to_string(), item_aliases)
                .is_none(),
            "rule {} was already defined",
            rule_ident.from_source(src)
        );
    }

    fn get_complete_rule(
        &self,
        pipe: &RulePipe,
        src: &str,
    ) -> (Vec<Vec<SrcRef>>, Vec<SrcRef>, Vec<Vec<ItemAlias>>) {
        fn push_all_prods(prods: &mut [Vec<SrcRef>], item: SrcRef) {
            for prod in prods.iter_mut() {
                prod.push(item);
            }
        }
        let mut prods = vec![vec![]];
        let mut item_aliases = vec![vec![]];
        for (index, g) in pipe.0.iter().enumerate() {
            let (g, optional, alias) = if let Ast::RuleItem(ref i, o, a) = g.item.item {
                (i, o, a)
            } else {
                unreachable!()
            };
            let item_alias = ItemAlias {
                alias: alias.unwrap_or(SrcRef::new(0, 0)),
                optional: if optional { Some(true) } else { None },
                index,
                final_index: None,
                active: alias.is_some(),
            };
            item_aliases
                .iter_mut()
                .for_each(|v| v.push(item_alias.clone()));

            let clones = if optional { prods.clone() } else { Vec::new() };
            match g.ty {
                Sym::StrLit => push_all_prods(
                    &mut prods,
                    *self
                        .aliases
                        .get(g.item.span.from_source(src))
                        .unwrap_or_else(|| {
                            eprintln!(
                                "using literal string for {} (maybe is missing an alias?)",
                                g.item.span.from_source(src)
                            );
                            &g.item.span
                        }),
                ),
                Sym::IdentPath => push_all_prods(
                    &mut prods,
                    *self
                        .aliases
                        .get(g.item.span.from_source(src))
                        .unwrap_or(&g.item.span),
                ),
                Sym::Rule => match g.item.item {
                    Ast::Rule(ref v) => {
                        let mut new_prods = Vec::new();
                        for prod in &prods {
                            for var in v {
                                for new_var in self.get_complete_rule(var, src).0 {
                                    let mut prod = prod.clone();
                                    prod.extend(new_var);
                                    new_prods.push(prod);
                                }
                            }
                        }
                        prods = new_prods;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!("{g:?}"),
            }
            prods.extend(clones);
            if optional {
                let disables: Vec<_> = item_aliases
                    .iter()
                    .cloned()
                    .map(|v| {
                        let mut v = v;
                        v.last_mut().unwrap().optional = Some(false);
                        v
                    })
                    .collect();
                item_aliases.extend(disables);
            }
        }
        let prods_size_it = 0..prods.len();
        (prods, prods_size_it.map(|_| pipe.1).collect(), item_aliases)
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
            writeln!(
                out,
                "\tmap.insert({r_name}, lrp::grammar::Rule::new({r_name}, vec!["
            )
            .unwrap();
            for imp in impls {
                out.push_str("\t\tvec![");
                for gramem in imp {
                    write!(out, "{}, ", gramem.from_source(src)).unwrap();
                }
                out.push_str("],\n");
            }
            out.push_str("\n\t]));\n");
        }
        out.push_str("\n\tmap\n}");
        out
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
        for (r_name, (ty, impls)) in &self.reductors {
            let ty = ty.from_source(src);
            for (i, imp) in impls.iter().enumerate() {
                let mut item_aliases = String::new();
                for alias in &self.item_aliases[r_name][i] {
                    alias.write(&mut item_aliases, src);
                }
                writeln!(
                    out,
                    "\tfn lrp_wop_{r_name}_{i}(toks: &[Gramem]) -> lrp::Meta<{ty}> {{\n\t\tlrp::Meta::new({{ {item_aliases} {}}}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))\n\t}}",
                    imp.from_source(src).strip_prefix("->").unwrap().strip_suffix('%').unwrap()
                )
                .unwrap();
            }
            write!(out, "\tmap.insert({r_name}, vec![").unwrap();
            for (i, _) in impls.iter().enumerate() {
                write!(out, "lrp_wop_{r_name}_{i}, ").unwrap();
            }
            out.push_str("\t]);\n\n");
        }

        out.push_str("\tmap\n}");
        out
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
