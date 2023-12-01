use std::collections::HashMap;

use lrp::Span;

use crate::{Ast, Gramem};

pub type SrcRef = Span;

#[derive(Debug, Default)]
pub struct Builder {
    pub aliases: HashMap<String, String>,
    pub variants: HashMap<String, Vec<(GramemEntry, Vec<SrcRef>)>>,
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
        for decl in program.iter().map(|e| {
            if let Ast::Declaration(ref decl) = e.item.item {
                decl
            } else {
                unreachable!()
            }
        }) {
            match &decl.item.item {
                Ast::RuleDecl(rule) => {}
                Ast::UseDecl(decl) => self.use_decl(decl),
                Ast::TokenDecl(tk, alias) => self.token_decl(tk, alias, src),
                c => unreachable!("unexpected {c:?} in code builder"),
            }
        }
    }

    fn token_decl(&mut self, tk: &Gramem, alias: &Gramem, src: &str) {
        assert!(
            self.aliases
                .insert(
                    tk.item.span.from_source(src).to_string(),
                    alias.item.span.from_source(src).to_string(),
                )
                .is_none(),
            "overriding an already defined alias: {} to {} ",
            tk.item.span.from_source(src).to_string(),
            alias.item.span.from_source(src).to_string(),
        );
    }

    fn use_decl(&mut self, decl: &Gramem) {
        self.imports.push(decl.item.span);
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
