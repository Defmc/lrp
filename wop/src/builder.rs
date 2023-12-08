use crate::{Ast, Gramem, RulePipe};
use lrp::Span;
use std::{collections::HashMap, fmt::Write};

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
                Ast::RuleDecl((rule_ident, rule_ty, rule)) => {
                    self.rule_decl(*rule_ident, *rule_ty, rule, src)
                }
                Ast::Import(decl) => self.use_decl(*decl),
                Ast::Alias(tk, alias) => self.token_decl(*tk, *alias, src),
                c => unreachable!("unexpected {c:?} in code builder"),
            }
        }
    }

    fn rule_decl(&mut self, rule_ident: Span, rule_ty: Span, rule: &[RulePipe], src: &str) {
        let mut prods = Vec::new();
        rule.iter().for_each(|prod| {
            for p in self.get_complete_rule(rule_ident, rule_ty, prod, src) {
                prods.push(p);
            }
        });

        assert!(
            self.rules
                .insert(rule_ident.from_source(src).to_string(), prods)
                .is_none(),
            "rule {} was already defined",
            rule_ident.from_source(src)
        );
    }

    fn get_complete_rule(
        &self,
        rule_ident: Span,
        rule_ty: Span,
        pipe: &RulePipe,
        src: &str,
    ) -> Vec<Vec<SrcRef>> {
        let mut prods = vec![vec![]];
        fn push_all(prods: &mut [Vec<Span>], item: SrcRef) {
            for prod in prods.iter_mut() {
                prod.push(item);
            }
        }
        // fn clone_all(prods: &mut Vec<Vec<Span>>) -> &mut [Vec<Span>] {
        //     let len = prods.len();
        //     prods.reserve(prods.len());
        //     for i in 0..prods.len() {
        //         prods.push(prods[i].clone());
        //     }
        //     &mut prods[len..]
        // }
        for g in &pipe.0 {
            println!("{g:?}");
            let src_ref = g.item.item.get_src_ref().unwrap();
            push_all(
                &mut prods,
                *self
                    .aliases
                    .get(src_ref.from_source(src))
                    .unwrap_or(&g.item.span),
            );
            // Sym::Rule => match g.item.item {
            //     Ast::Rule(ref v) => {
            //         let mut new_prods = Vec::new();
            //         for prod in &prods {
            //             for var in v {
            //                 for new_var in self.get_complete_rule(var, src) {
            //                     let mut prod = prod.clone();
            //                     prod.extend(new_var);
            //                     new_prods.push(prod);
            //                 }
            //             }
            //         }
            //         prods = new_prods;
            //     }
            //     _ => unreachable!(),
            // },
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

    pub fn dump_grammar(&self, src: &str) -> String {
        let mut out = "{\n".to_string();
        self.imports.iter().for_each(|i| {
            writeln!(out, "\tuse {};", i.from_source(src)).unwrap();
        });
        writeln!(out, "\tlet mut map = lrp::RuleMap::new();").unwrap();
        for (r_name, impls) in &self.rules {
            writeln!(out, "\tmap.insert({r_name}, vec![").unwrap();
            for i in 0..impls.len() {
                out.push_str("\t\tvec![");
                for gramem in &impls[i] {
                    write!(out, "{}, ", gramem.from_source(src)).unwrap();
                }
                out.push_str("],\n");
            }
            out.push_str("\n\t]);\n");
        }
        out.push_str("\n}");
        out
    }
}
