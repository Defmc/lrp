use std::collections::HashMap;

use lrp::Span;

use crate::{Ast, Gramem};

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
                Ast::Rule(rule_name, rule) => self.rule_decl(rule_name, rule),
                Ast::Import(decl) => self.use_decl(*decl),
                Ast::Alias(tk, alias) => self.token_decl(*tk, *alias, src),
                c => unreachable!("unexpected {c:?} in code builder"),
            }
        }
    }

    fn rule_decl(&mut self, rule_name: &Span, rule: &[Span]) {
        // let rule_name = rule[0];
        // let rule: Vec<_> = rule.iter().skip(2).cloned().collect();
        todo!()
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
