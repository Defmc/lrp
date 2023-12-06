use crate::{Ast, Attr, Gramem, Sym};
use lrp::Span;
use std::collections::HashMap;

pub type SrcRef = Span;

#[derive(Debug, Default)]
pub struct Builder {
    pub aliases: HashMap<String, SrcRef>,
    pub rules: HashMap<String, Vec<Vec<SrcRef>>>,
    pub imports: Vec<SrcRef>,
}

impl Builder {
    pub fn get_program_instructions(ast: &Gramem) -> &Vec<Gramem> {
        if let Ast::Program(p) = &ast.item.item {
            p
        } else {
            unreachable!();
        }
    }

    pub fn process(&mut self, ast: &Gramem, src: &str) {
        let program = Self::get_program_instructions(&ast);
        for decl in program.iter() {
            match &decl.item.item {
                Ast::RuleDecl(rule_name, rule) => self.rule_decl(rule_name, rule, src),
                Ast::Import(decl) => self.use_decl(*decl),
                Ast::Alias(tk, alias) => self.token_decl(*tk, *alias, src),
                c => unreachable!("unexpected {c:?} in code builder"),
            }
        }
    }

    fn rule_decl(&mut self, rule_name: &Span, rule: &[Vec<Gramem>], src: &str) {
        let mut prods = Vec::new();
        rule.iter().for_each(|prod| {
            for p in self.get_complete_rule(prod, src) {
                prods.push(p);
            }
        });

        assert!(
            self.rules
                .insert(rule_name.from_source(src).to_string(), prods)
                .is_none(),
            "rule {} was already defined",
            rule_name.from_source(src)
        );
    }

    fn get_complete_rule(&self, pipe: &[Gramem], src: &str) -> Vec<Vec<SrcRef>> {
        let mut prods = vec![vec![]];
        fn push_all(prods: &mut [Vec<Span>], item: SrcRef) {
            for prod in prods.iter_mut() {
                prod.push(item);
            }
        }
        fn clone_all(prods: &mut Vec<Vec<Span>>) -> &mut [Vec<Span>] {
            let len = prods.len();
            prods.reserve(prods.len());
            for i in 0..prods.len() {
                prods.push(prods[i].clone());
            }
            &mut prods[len..]
        }
        for g in pipe {
            let (g, attr) = g.item.item.as_rule_item().unwrap();
            match g.ty {
                Sym::Ident => {
                    let prods = if attr == &Attr::Optional {
                        clone_all(&mut prods)
                    } else {
                        &mut prods
                    };
                    push_all(
                        prods,
                        *self
                            .aliases
                            .get(g.item.span.from_source(src))
                            .unwrap_or(&g.item.span),
                    )
                }
                Sym::StrLit => {
                    let prods = if attr == &Attr::Optional {
                        clone_all(&mut prods)
                    } else {
                        &mut prods
                    };
                    push_all(
                        prods,
                        *self
                            .aliases
                            .get(g.item.span.from_source(src))
                            .unwrap_or_else(|| {
                                panic!(
                                    "missing alias for {:?} literal",
                                    g.item.span.from_source(src)
                                )
                            }),
                    )
                } // TODO: Impl optional
                Sym::Rule => match g.item.item {
                    Ast::Rule(ref v) => {
                        let mut new_prods = Vec::new();
                        for prod in &prods {
                            for var in v {
                                for new_var in self.get_complete_rule(var, src) {
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
                _ => unreachable!(),
            }
        }
        prods
    }

    fn token_decl(&mut self, tk: Span, alias: Span, src: &str) {
        assert!(
            self.aliases
                .insert(tk.from_source(src).to_string(), alias)
                .is_none(),
            "overriding an already defined alias: {} to {} ",
            tk.from_source(src).to_string(),
            alias.from_source(src).to_string(),
        );
    }

    fn use_decl(&mut self, decl: Span) {
        self.imports.push(decl);
    }

    pub fn dump(&self, src: &str) -> String {
        let mut out = String::new();
        self.imports.iter().for_each(|i| {
            out.push_str("use ");
            out.push_str(i.from_source(src));
            out.push_str(";\n");
        });
        out.push_str("fn grammar() -> Grammar<Ast> {");
        out.push_str("\n\tuse Ast::*");
        out.push_str("\n\tlet mut grammar = lrp::RuleMap::new();");
        for (r_name, impls) in &self.rules {
            out.push_str("\n\tgrammar.insert(");
            out.push_str(r_name);
            out.push_str(", vec![");
            for i in 0..impls.len() {
                out.push_str("\n\t\tvec![");
                for gramem in &impls[i] {
                    out.push_str(&gramem.from_source(src));
                    out.push_str(", ");
                }
                out.push_str("]")
            }
            out.push_str("]);");
        }
        out.push_str("\n\tGrammar::new(Token::EntryPoint, grammar, Token::Eof)\n}");
        out
    }
}
