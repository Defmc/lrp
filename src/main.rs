use std::iter::Peekable;

use lrp::{Clr, Dfa, Grammar, Parser, Set, Tabler, Term};
use prettytable::{row, Cell, Row, Table};

fn main() {
    let grammar = lrp::grammar! {
        "S" -> "C" "C",
        "C" -> "c" "C"
            | "d"
    };

    let grammar = Grammar::new("S", grammar, Set::from(["c", "d"]));
    let inputs: &[&[&str]] = &[&["c", "d", "d"], &["d", "d"]];
    let parser = Clr::new(grammar);

    let tables = parser.tables();

    print_tokens_table(tables);

    print_states_table(tables);
    print_actions_table(tables);

    let mut dfa = parser.dfa((&[]).into_iter().copied());
    for input in inputs.into_iter().copied() {
        dfa.reset();
        dfa.buffer = input.into_iter().copied().peekable();
        print_proc_dfa(&mut dfa);
    }
}

fn print_tokens_table(table: &Tabler) {
    let mut out = Table::new();
    out.set_titles(row!["Non terminal", "First tokens", "Follow tokens"]);

    let iter = table.first.iter().zip(table.follow.iter().map(|(_, f)| f));

    for ((k, first), follow) in iter {
        out.add_row(row![k, format!("{first:?}"), format!("{follow:?}")]);
    }

    out.printstd();
}

fn print_states_table(table: &Tabler) {
    let mut out = Table::new();
    out.set_titles(row!["Goto(Idx, Term)", "Kernel", "State", "Closure"]);

    let internal = table.basis_pos();
    let start = &table.states[0];
    let syms: Vec<_> = table
        .grammar
        .symbols()
        .chain(["LRP'START", lrp::EOF].into_iter())
        .collect();

    out.add_row(row!["", format!("{internal:?}"), "0", format!("{start:?}")]);

    for (i, state) in table.states.iter().enumerate() {
        for sym in &syms {
            let kernel = Tabler::sym_filter(state, &sym);
            if kernel.is_empty() {
                continue;
            }
            let state_id = table.kernels[&kernel];
            out.add_row(row![
                format!("goto({i}, {sym})"),
                format!("{kernel:?}"),
                format!("{state_id}"),
                format!("{:?}", table.states[state_id])
            ]);
        }
    }

    out.printstd();
}

fn print_actions_table(table: &Tabler) {
    let mut out = Table::new();

    let (terminals, nonterminals) = table
        .grammar
        .symbols()
        .chain(["LRP'START", lrp::EOF].into_iter())
        .fold((Set::new(), Set::new()), |(mut ts, mut nts), s| {
            if table.grammar.is_terminal(&s) {
                ts.insert(s);
            } else {
                nts.insert(s);
            }
            (ts, nts)
        });

    let rows: Vec<_> = terminals
        .iter()
        .chain(["State"].iter())
        .chain(nonterminals.iter())
        .map(|t| {
            if t == &lrp::EOF {
                Cell::new("EOF (\\0x03)")
            } else {
                Cell::new(t)
            }
        })
        .collect();

    out.set_titles(Row::new(rows));

    let mut row_buf = Vec::new();
    for (state_idx, state) in table.actions.iter().enumerate() {
        row_buf.clear();
        for t in &terminals {
            let item = if let Some(act) = state.get(t) {
                format!("{act:?}")
            } else {
                String::new()
            };
            row_buf.push(Cell::new(&item));
        }
        row_buf.push(Cell::new(&format!("{state_idx}")));

        for nt in &nonterminals {
            let item = if let Some(act) = state.get(nt) {
                format!("{act:?}")
            } else {
                String::new()
            };
            row_buf.push(Cell::new(&item));
        }

        out.add_row(Row::new(row_buf.clone()));
    }

    out.printstd();
}

fn print_proc_dfa<I: Iterator<Item = Term>>(dfa: &mut Dfa<I>)
where
    Peekable<I>: Clone,
{
    let mut out = Table::new();
    out.set_titles(row!["Step", "Stack", "Buffer", "Action Address", "Action"]);

    dfa.trace(|state| {
        let step = out.len();
        let stack = state.stack_fmt();
        let buffer = format!(
            "{:?}",
            state
                .buffer
                .clone()
                .chain(["EOF"].into_iter())
                .collect::<Vec<_>>()
        );
        let symbol = state.buffer.peek().unwrap_or(&lrp::EOF);
        let action = format!("{:?}", state.table[state.top][symbol]);
        let action_adr = format!("{}:{:?}", state.top, symbol);

        out.add_row(row![step, stack, buffer, action_adr, action]);
    });

    out.printstd();
}
