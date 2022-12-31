use crate::{ActTable, Action, Grammar, Map, Parser, Tabler, Term};

pub struct Clr {
    pub table: Tabler,
}

impl Parser for Clr {
    fn new(grammar: Grammar) -> Self {
        let mut parser = Clr {
            table: Tabler::new(grammar),
        };
        parser.proc_actions();
        parser
    }

    fn gotos(&self) -> &ActTable {
        &self.table.actions
    }
    fn actions(&self) -> &ActTable {
        &self.table.actions
    }

    fn proc_actions(&mut self) {
        self.table.proc_closures();
        let start = self.table.basis_pos().rule;
        for row in self.table.states.iter() {
            let mut map: Map<Term, Action> = Map::new();
            for item in row {
                for (term, act) in self.table.decision(start, item, row) {
                    if map.contains_key(term) && map[term] != act {
                        *map.get_mut(term).unwrap() =
                            Action::Conflict(Box::new(map.get(term).unwrap().clone()), act.into());
                    } else {
                        map.insert(term, act);
                    }
                }
            }
            self.table.actions.push(map);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{grammars_tests, Clr, Parser};

    #[test]
    pub fn dragon_book() {
        let clr = Clr::new(grammars_tests::dragon_book());

        clr.parse(["d", "d"]);
        clr.parse(["d", "c", "d"]);
        clr.parse(["c", "d", "d"]);
        clr.parse(["d", "c", "c", "d"]);
        clr.parse(["c", "d", "c", "d"]);
        clr.parse(["c", "c", "d", "d"]);
        clr.parse(["d", "c", "c", "c", "d"]);
        clr.parse(["c", "d", "c", "c", "d"]);
        clr.parse(["c", "c", "d", "c", "d"]);
        clr.parse(["c", "c", "c", "d", "d"]);
        clr.parse(["d", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "d", "c", "c", "c", "d"]);
        clr.parse(["c", "c", "d", "c", "c", "d"]);
        clr.parse(["c", "c", "c", "d", "c", "d"]);
        clr.parse(["c", "c", "c", "c", "d", "d"]);
        clr.parse(["d", "c", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "d", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "c", "d", "c", "c", "c", "d"]);
        clr.parse(["c", "c", "c", "d", "c", "c", "d"]);
        clr.parse(["c", "c", "c", "c", "d", "c", "d"]);
        clr.parse(["c", "c", "c", "c", "c", "d", "d"]);
        clr.parse(["d", "c", "c", "c", "c", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "d", "c", "c", "c", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "c", "d", "c", "c", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "c", "c", "d", "c", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "c", "c", "c", "c", "c", "d", "c", "c", "d"]);
        clr.parse(["c", "c", "c", "c", "c", "c", "c", "d", "c", "d"]);
        clr.parse(["c", "c", "c", "c", "c", "c", "c", "c", "d", "d"]);
        clr.parse(["d", "c", "c", "c", "c", "c", "c", "c", "d"]);
        clr.parse(["c", "d", "c", "c", "c", "c", "c", "c", "d"]);
    }

    #[test]
    fn wikipedia() {
        let clr = Clr::new(grammars_tests::wikipedia());

        clr.parse(["0"]);
        clr.parse(["1"]);
        clr.parse(["0", "*", "0"]);
        clr.parse(["0", "*", "1"]);
        clr.parse(["1", "*", "0"]);
        clr.parse(["1", "*", "1"]);
        clr.parse(["0", "+", "0"]);
        clr.parse(["0", "+", "1"]);
        clr.parse(["1", "+", "0"]);
        clr.parse(["1", "+", "1"]);
        clr.parse(["0", "*", "0", "*", "0"]);
        clr.parse(["0", "*", "0", "*", "1"]);
        clr.parse(["0", "*", "1", "*", "0"]);
        clr.parse(["0", "*", "1", "*", "1"]);
        clr.parse(["1", "*", "0", "*", "0"]);
        clr.parse(["1", "*", "0", "*", "1"]);
        clr.parse(["1", "*", "1", "*", "0"]);
        clr.parse(["1", "*", "1", "*", "1"]);
        clr.parse(["0", "+", "0", "*", "0"]);
        clr.parse(["0", "+", "0", "*", "1"]);
        clr.parse(["0", "+", "1", "*", "0"]);
        clr.parse(["0", "+", "1", "*", "1"]);
        clr.parse(["1", "+", "0", "*", "0"]);
        clr.parse(["1", "+", "0", "*", "1"]);
        clr.parse(["1", "+", "1", "*", "0"]);
        clr.parse(["1", "+", "1", "*", "1"]);
        clr.parse(["0", "*", "0", "+", "0"]);
        clr.parse(["0", "*", "0", "+", "1"]);
        clr.parse(["0", "*", "1", "+", "0"]);
        clr.parse(["0", "*", "1", "+", "1"]);
    }

    // https://smlweb.cpsc.ucalgary.ca/
    #[test]
    fn ucalgary_uni_oth_lr1() {
        let clr = Clr::new(grammars_tests::ucalgary_uni_oth_lr1());

        clr.parse(["e", "a", "c"]);
        clr.parse(["d", "a", "b"]);
        clr.parse(["d", "e", "a", "c"]);
        clr.parse(["d", "e", "a", "b"]);
        clr.parse(["e", "d", "a", "b"]);
        clr.parse(["e", "d", "a", "c"]);
        clr.parse(["d", "d", "e", "a", "b"]);
        clr.parse(["e", "e", "d", "a", "c"]);
    }

    #[test]
    fn serokell() {
        let clr = Clr::new(grammars_tests::serokell());

        clr.parse(["int"]);
        clr.parse(["int", "*", "int"]);
        clr.parse(["ident", "*", "int"]);
        clr.parse(["(", "int", ")"]);
        clr.parse(["int", "+", "int"]);
        clr.parse(["ident", "+", "int"]);
        clr.parse(["int", "*", "int", "*", "int"]);
        clr.parse(["int", "*", "ident", "*", "int"]);
        clr.parse(["ident", "*", "int", "*", "int"]);
        clr.parse(["ident", "*", "ident", "*", "int"]);
        clr.parse(["int", "*", "(", "int", ")"]);
        clr.parse(["ident", "*", "(", "int", ")"]);
        clr.parse(["int", "*", "int", "+", "int"]);
        clr.parse(["int", "*", "(", "ident", "+", "int", ")"]);
        clr.parse(["ident", "*", "int", "+", "int"]);
        clr.parse([
            "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(", "(",
            "(", "(", "(", "(", "(", "int", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
            ")", ")", ")", ")", ")", ")", ")", ")", ")", ")", ")",
        ]);
    }

    #[test]
    pub fn puncs() {
        let clr = Clr::new(grammars_tests::puncs());

        clr.parse(["(", ")"]);
        clr.parse(["[", "]"]);
        clr.parse(["{", "}"]);
        clr.parse(["(", "(", ")", ")"]);
        clr.parse(["(", "[", "]", ")"]);
        clr.parse(["(", "{", "}", ")"]);
        clr.parse(["[", "(", ")", "]"]);
        clr.parse(["[", "[", "]", "]"]);
        clr.parse(["[", "{", "}", "]"]);
        clr.parse(["{", "(", ")", "}"]);
        clr.parse(["{", "[", "]", "}"]);
        clr.parse(["{", "{", "}", "}"]);
        clr.parse(["(", "(", "(", ")", ")", ")"]);
        clr.parse(["(", "(", "[", "]", ")", ")"]);
        clr.parse(["(", "(", "{", "}", ")", ")"]);
        clr.parse(["(", "[", "(", ")", "]", ")"]);
        clr.parse(["(", "[", "[", "]", "]", ")"]);
        clr.parse(["(", "[", "{", "}", "]", ")"]);
        clr.parse(["(", "{", "(", ")", "}", ")"]);
        clr.parse(["(", "{", "[", "]", "}", ")"]);
        clr.parse(["(", "{", "{", "}", "}", ")"]);
        clr.parse(["[", "(", "(", ")", ")", "]"]);
        clr.parse(["[", "(", "[", "]", ")", "]"]);
        clr.parse(["[", "(", "{", "}", ")", "]"]);
        clr.parse(["[", "[", "(", ")", "]", "]"]);
        clr.parse(["[", "[", "[", "]", "]", "]"]);
        clr.parse(["[", "[", "{", "}", "]", "]"]);
        clr.parse(["[", "{", "(", ")", "}", "]"]);
        clr.parse(["[", "{", "[", "]", "}", "]"]);
        clr.parse(["[", "{", "{", "}", "}", "]"]);
    }
}
