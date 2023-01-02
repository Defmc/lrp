use std::{fmt, iter::Peekable, rc::Rc};

use crate::{ActTable, Production, Rule, RuleName, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Shift(usize),
    Goto(usize),
    Reduce(RuleName, Rc<Production>),
    Acc,
    Conflict(Box<Action>, Box<Action>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    Simple(Rule),
    Compound(Rule, Vec<Item>),
}

impl Item {
    #[must_use]
    pub const fn name(&self) -> Rule {
        match self {
            Self::Simple(n) | Self::Compound(n, ..) => n,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())?;
        if let Self::Compound(_, elms) = self {
            f.write_fmt(format_args!(
                " -> ({})",
                elms.iter()
                    .map(|f| format!("{f}"))
                    .collect::<Vec<String>>()
                    .join(" ")
            ))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackEl {
    Item(Item),
    State(usize),
}

impl fmt::Display for StackEl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Item(it) => f.write_fmt(format_args!("{it}")),
            Self::State(n) => f.write_fmt(format_args!("{n}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Found a unexpected token. Contains a vector with correct tokens after it
    UnexpectedToken(Term, Vec<Term>),
    /// Unexpected buffer end. When receive `lrp::EOF` before finish it
    UnexepectedEof,
    /// Unsolved conflict. When the current state contains a conflicting action
    Conflict(Action, Action),
    /// Missing state. When reduce actions don't contains a previous state
    MissingPreviousState,
    /// State not specified in actions table. Must not occur in a run without an external interference
    StateNotSpecified,
    /// Incomplete execution. Finished the parsing without consume entire buffer
    IncompleteExec,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(found, expected) => f.write_fmt(format_args!(
                "unexpected token {found}. expected {expected:?}"
            )),
            Self::UnexepectedEof => f.write_str("unexpected eof"),
            Self::Conflict(a, b) => f.write_fmt(format_args!("conflicting action {a:?} and {b:?}")),
            Self::MissingPreviousState => {
                f.write_str("missing previous state. impossible to continue dfa execution")
            }
            Self::StateNotSpecified => f.write_str("state not specified in actions table"),
            Self::IncompleteExec => {
                f.write_str("finished the parsing without consume entire buffer")
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Dfa<I: Iterator<Item = Term>> {
    pub buffer: Peekable<I>,
    pub stack: Vec<StackEl>,
    pub table: ActTable,
    pub top: usize,
    pub finished: bool,
}

impl<I: Iterator<Item = Term>> Dfa<I> {
    #[must_use]
    pub fn new(buffer: I, table: ActTable) -> Self {
        Self {
            stack: vec![StackEl::State(0)],
            buffer: buffer.peekable(),
            top: 0,
            table,
            finished: false,
        }
    }

    pub fn shift(&mut self, to: usize) -> Result<()> {
        let item = Item::Simple(self.buffer.next().ok_or(Error::UnexepectedEof)?);
        self.stack.push(StackEl::Item(item));
        self.top = to;
        self.stack.push(StackEl::State(self.top));
        Ok(())
    }

    pub fn accept(&mut self) -> Result<()> {
        self.finished = true;
        if self.buffer.peek().is_none() {
            Ok(())
        } else {
            Err(Error::IncompleteExec)
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.trace(|_| {})
    }

    pub fn trace(&mut self, mut f: impl FnMut(&mut Self)) -> Result<()> {
        while !self.finished {
            f(self);
            let symbol = *self.buffer.peek().unwrap_or(&crate::EOF);
            self.travel(symbol)?;
        }
        Ok(())
    }

    pub fn goto(&mut self, to: usize) {
        self.stack.push(StackEl::State(to));
        self.top = to;
    }

    pub fn travel(&mut self, symbol: Term) -> Result<()> {
        let state = self.table.get(self.top).ok_or(Error::StateNotSpecified)?;
        let action = state.get(symbol).ok_or_else(|| {
            let expecteds = state.keys().copied().collect();
            Error::UnexpectedToken(symbol, expecteds)
        })?;
        match action {
            Action::Shift(to) => self.shift(*to)?,
            Action::Goto(to) => self.goto(*to),
            Action::Reduce(name, prod) => self.reduce(name, &prod.clone())?,
            Action::Acc => self.accept()?,
            Action::Conflict(a, b) => Err(Error::Conflict(*a.clone(), *b.clone()))?,
        }
        Ok(())
    }

    pub fn reduce(&mut self, name: RuleName, prod: &Rc<Production>) -> Result<()> {
        let mut item = Vec::with_capacity(prod.len());
        while item.len() != prod.len() {
            let poped = self.stack.pop().ok_or(Error::UnexepectedEof)?;
            if let StackEl::Item(i) = poped {
                item.push(i);
            }
        }
        let item = Item::Compound(name, item);
        let state = self
            .stack
            .iter()
            .rev()
            .find_map(|i| {
                if let StackEl::State(n) = i {
                    Some(*n)
                } else {
                    None
                }
            })
            .ok_or(Error::MissingPreviousState)?;
        self.stack.push(StackEl::Item(item));
        self.top = state;
        self.travel(name)
    }

    pub fn reset(&mut self) {
        self.finished = false;
        self.stack.clear();
        self.stack.push(StackEl::State(0));
        self.top = 0;
    }

    #[must_use]
    pub fn parse(&mut self, input: I) -> Result<StackEl> {
        self.reset();
        self.buffer = input.peekable();
        self.start()?;
        let res = self.stack[1].clone();
        self.reset();
        Ok(res)
    }

    #[must_use]
    pub fn stack_fmt(&self) -> String {
        self.stack
            .iter()
            .map(|f| format!("{f}"))
            .collect::<Vec<String>>()
            .join(" ")
    }
}
