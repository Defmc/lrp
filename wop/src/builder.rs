use crate::{Ast, Gramem, RulePipe};
use std::{collections::HashMap, fmt::Write};

pub type SrcRef = lrp::Span;

#[derive(Debug, Default)]
pub struct Builder {
    pub aliases: HashMap<String, SrcRef>,
    pub rules: HashMap<String, Vec<Vec<SrcRef>>>,
    pub reductors: HashMap<String, (SrcRef, Vec<SrcRef>)>,
    pub imports: Vec<SrcRef>,
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
        for prod in rule {
            let (p, r) = self.get_complete_rule(prod, src);
            prods.extend(p);
            reductors.extend(r);
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
    }

    fn get_complete_rule(&self, pipe: &RulePipe, src: &str) -> (Vec<Vec<SrcRef>>, Vec<SrcRef>) {
        fn push_all(prods: &mut [Vec<SrcRef>], item: SrcRef) {
            for prod in prods.iter_mut() {
                prod.push(item);
            }
        }
        let mut prods = vec![vec![]];
        // fn clone_all(prods: &mut Vec<Vec<SrcRef>>) -> &mut [Vec<Span>] {
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
        let prods_size_it = 0..prods.len();
        (prods, prods_size_it.map(|_| pipe.1).collect())
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
                out.push_str("]),\n");
            }
            out.push_str("\n\t]);\n");
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
                writeln!(
                    out,
                    "\tfn lrp_wop_{r_name}_{i}(toks: &[Gramem]) -> lrp::Meta<{ty}> {{\n\t\tlrp::Meta::new({}, lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end))\n\t}}",
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
