use std::{
    fmt,
    io::{self, Write},
    iter::Peekable,
};

use lrp::{Dfa, Grammar, Lalr, Parser, Set, Tabler, Token};
use prettytable::{row, Cell, Row, Table};

fn main() {
    let grammar = lrp::grammar_map! {
        "'S" -> "S",
        "S" -> "(" ")"
            | "(" "S" ")"
            | "[" "]"
            | "[" "S" "]"
            | "{" "}"
            | "{" "S" "}"
    };
    let terminals = Set::from(["[", "]", "(", ")", "{", "}", "d", "c"]);
    let grammar = Grammar::new("'S", grammar, "$");

    let parser = Lalr::new(grammar);

    let tables = parser.tables();

    print_tokens_table(tables);

    print_states_table(tables, &parser);
    print_actions_table(tables);

    loop {
        print!("input: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        #[allow(clippy::needless_collect)]
        let input: Vec<_> = input.trim().chars().map(|c| c.to_string()).collect();
        let mut dfa = parser.simple_dfa(
            input
                .into_iter()
                .map(|t| Token::new((), *terminals.get(t.as_str()).unwrap())),
        );
        print_proc_dfa(&mut dfa);
    }
}

fn print_tokens_table<T>(table: &Tabler<T>)
where
    T: Clone + Ord + fmt::Display + fmt::Debug,
{
    let mut out = Table::new();
    out.set_titles(row!["non terminal", "first tokens", "follow tokens"]);

    let iter = table.first.iter().zip(table.follow.values());

    for ((k, first), follow) in iter {
        out.add_row(row![k, format!("{first:?}"), format!("{follow:?}")]);
    }

    out.printstd();
}

fn print_states_table<T>(table: &Tabler<T>, parser: &impl Parser<T>)
where
    T: Clone + Ord + fmt::Display + fmt::Debug,
{
    let mut out = Table::new();
    out.set_titles(row!["goto(idx, term)", "kernel", "state", "closure"]);

    let internal = table.basis_pos();
    let start = &table.states[0];
    let syms: Vec<_> = table.grammar.symbols().collect();

    out.add_row(row![
        format!("last state: {}", table.states.len() - 1),
        format!("{internal:?}"),
        "0",
        format!("{start:?}")
    ]);

    for (i, state) in table.states.iter().enumerate() {
        for sym in &syms {
            let kernel = Tabler::sym_filter(state, sym);
            if kernel.is_empty() {
                continue;
            }
            let kernel = parser.final_kernel(&kernel).unwrap();
            let state_id = parser.state_from_kernel(kernel).unwrap();
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

fn print_actions_table<T>(table: &Tabler<T>)
where
    T: Clone + Ord + fmt::Display + fmt::Debug,
{
    let mut out = Table::new();

    let (terminals, nonterminals) =
        table
            .grammar
            .symbols()
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
        .chain(nonterminals.iter())
        .map(|t| Cell::new(&format!("{t:?}")))
        .collect();

    out.set_titles(Row::new(rows));

    let mut row_buf = Vec::new();
    for (state_idx, state) in table.actions.iter().enumerate() {
        row_buf.clear();
        for t in &terminals {
            let item = state
                .get(t)
                .map_or_else(String::new, |act| format!("{act:?}"));
            row_buf.push(Cell::new(&item));
        }
        row_buf.push(Cell::new(&format!("{state_idx}")));

        for nt in &nonterminals {
            let item = state
                .get(nt)
                .map_or_else(String::new, |act| format!("{act:?}"));
            row_buf.push(Cell::new(&item));
        }

        out.add_row(Row::new(row_buf.clone()));
    }

    out.printstd();
}

fn print_proc_dfa<M, T, I: Iterator<Item = Token<M, T>>>(dfa: &mut Dfa<M, T, I>)
where
    Peekable<I>: Clone,
    M: Clone + fmt::Debug,
    T: fmt::Display + fmt::Debug + Clone + Ord,
{
    let mut out = Table::new();
    out.set_titles(row!["step", "stack", "buffer", "action address", "action"]);
    let eof = dfa.eof.clone();

    let res = dfa.trace(|state| {
        let step = out.len() + 1;
        let stack = state.stack_fmt();
        let buffer = format!(
            "{:?}",
            state
                .buffer
                .clone()
                .map(|Token { ty, .. }| ty)
                .chain(std::iter::once(eof.clone()))
                .collect::<Vec<_>>()
        );
        let symbol = state
            .buffer
            .peek()
            .map_or_else(|| eof.clone(), |t| t.ty.clone());
        let action = state
            .table
            .get(state.top)
            .and_then(|t| t.get(&symbol))
            .map_or_else(|| "n/a".to_string(), |a| format!("{a:?}"));
        let action_adr = format!("{}:{:?}", state.top, symbol);

        out.add_row(row![step, stack, buffer, action_adr, action]);
    });
    out.add_row(row![
        "state",
        res.map_or_else(|e| format!("{e}"), |_| "ok".to_string())
    ]);

    out.printstd();
}
