use std::collections::HashMap;

use lrp::Span;

use crate::{Ast, Gramem, Sym};

pub type SrcRef = Span;

#[derive(Debug, Default)]
pub struct Builder {
    pub aliases: HashMap<String, SrcRef>,
    pub gramems: HashMap<String, Vec<Vec<SrcRef>>>,
    pub imports: Vec<SrcRef>,
}

#[derive(Debug)]
pub enum GramemEntry {
    Normal,
    Variadic,
    Optional,
    Repeated,
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
        rule.iter()
            .for_each(|r| self.extend_complete_rule(&[], &mut prods, r, src));

        assert!(
            self.gramems
                .insert(rule_name.from_source(src).to_string(), prods)
                .is_none(),
            "rule {} was already defined",
            rule_name.from_source(src)
        );
    }

    fn extend_complete_rule(
        &self,
        prefix: &[SrcRef],
        buf: &mut Vec<Vec<SrcRef>>,
        rule: &[Gramem],
        src: &str,
    ) {
        // TODO: Impl sub prods
        let mut prod = prefix.to_vec();
        for g in rule {
            match g.ty {
                Sym::Ident => {
                    let p = self
                        .aliases
                        .get(g.item.span.from_source(src))
                        .unwrap_or(&g.item.span);
                    prod.push(*p);
                }
                Sym::StrLit => {
                    let p = self
                        .aliases
                        .get(g.item.span.from_source(src))
                        .unwrap_or_else(|| {
                            panic!("no entry for {:?} literal", g.item.span.from_source(src))
                        });
                    prod.push(*p);
                }
                _ => unreachable!("{:?}", g.ty),
            }
        }
        buf.push(prod);
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
        out
    }
}
