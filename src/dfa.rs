use std::{fmt, iter::Peekable, rc::Rc};

use crate::{ActTable, Map, Production, Tabler, Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action<T> {
    Shift(usize),
    Goto(usize),
    Reduce(T, Rc<Production<T>>),
    Acc,
    Conflict(Box<Action<T>>, Box<Action<T>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error<T>
where
    T: fmt::Debug,
{
    /// Found a unexpected token. Contains a vector with correct tokens after it. Indicates a bad
    /// input.
    UnexpectedToken(T, Vec<T>),
    /// Specialized version of `Error::UnexpectedToken` for buffer end in a Shifting Action. Indicates a bad input.
    UnexpectedEof,
    /// Unsolved conflict. When the current state contains a conflicting action. It's the unique
    /// natural possible error.
    Conflict(Action<T>, Action<T>),
    /// Missing state. When reduce actions don't contains a previous state. Mustn't occur in a run without an external interference
    MissingPreviousState,
    /// State not specified in actions table. Mustn't occur in a run without an external interference
    StateNotSpecified,
    /// Incomplete execution. Finished the parsing without consume entire buffer. Indicates a bad
    /// input.
    IncompleteExec,
}

impl<T> fmt::Display for Error<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(found, expected) => f.write_fmt(format_args!(
                "unexpected token {found:?}. expected {expected:?}"
            )),
            Self::UnexpectedEof => f.write_str("unexpected eof"),
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

pub type Result<T> = BaseResult<T, Error<T>>;
pub type BaseResult<T, E> = std::result::Result<T, E>;

// TODO: Allow to move the value
pub type ReductFn<T, M> = fn(&[Token<T, M>]) -> T;
pub type ReductMap<T, M> = Map<M, Vec<ReductFn<T, M>>>;

#[derive(Clone)]
pub struct Dfa<T, M, I: Iterator<Item = Token<T, M>>>
where
    T: Clone,
    M: fmt::Debug + Clone,
{
    pub buffer: Peekable<I>,
    pub states: Vec<usize>,
    pub items: Vec<Token<T, M>>,
    pub table: ActTable<M>,
    pub top: usize,
    pub finished: bool,
    pub reductors: ReductMap<T, M>,
    pub eof: M,
}

#[allow(clippy::mismatching_type_param_order)]
impl<T, I: Iterator<Item = Token<T, T>>> Dfa<T, T, I> where
    T: Clone + fmt::Debug + fmt::Display + Ord
{
}

impl<T, M, I: Iterator<Item = Token<T, M>>> Dfa<T, M, I>
where
    T: Clone,
    M: fmt::Debug + Clone + Ord,
{
    #[must_use]
    pub fn new(buffer: I, table: ActTable<M>, reductors: ReductMap<T, M>, eof: M) -> Self {
        Self {
            states: vec![0],
            items: Vec::new(),
            buffer: buffer.peekable(),
            top: 0,
            table,
            reductors,
            finished: false,
            eof,
        }
    }

    #[must_use]
    pub fn transparent(table: &Tabler<M>, func: ReductFn<T, M>) -> ReductMap<T, M> {
        table
            .grammar
            .rules()
            .map(|r| {
                let prods = r.prods().map(|_| func);
                (r.name.clone(), prods.collect::<Vec<_>>())
            })
            .collect()
    }

    /// # Errors
    /// When there is no more data in buffer, raises an `Error::UnexepectedEof`
    pub fn shift(&mut self, to: usize) -> BaseResult<(), Error<M>> {
        let item = self.buffer.next().ok_or(Error::UnexpectedEof)?;
        self.items.push(item);
        self.top = to;
        self.states.push(self.top);
        Ok(())
    }

    /// # Errors
    /// When finished parse without consume entire buffer, raises an `Error::IncompleteExec`
    pub fn accept(&mut self) -> BaseResult<(), Error<M>> {
        self.finished = true;
        if self.buffer.peek().is_none() {
            Ok(())
        } else {
            Err(Error::IncompleteExec)
        }
    }

    /// # Errors
    /// The same of `dfa::travel`
    pub fn start(&mut self) -> BaseResult<(), Error<M>> {
        self.trace(|_| ())
    }

    /// # Errors
    /// The same of `dfa::travel`
    pub fn trace(&mut self, mut f: impl FnMut(&mut Self)) -> BaseResult<(), Error<M>> {
        while !self.finished {
            f(self);
            let symbol = self
                .buffer
                .peek()
                .map_or_else(|| &self.eof, |t| &t.ty)
                .clone();
            self.travel(&symbol)?;
        }
        Ok(())
    }

    /// # Errors
    /// None
    pub fn goto(&mut self, to: usize) -> BaseResult<(), Error<M>> {
        self.states.push(to);
        self.top = to;
        Ok(())
    }

    /// # Errors
    /// If the current state don't exists in actions table, raises an `Error::StateNotSpecified`
    /// If there isn't an action in current state for `symbol`, raises an `Error::UnexpectedToken`
    /// Returns the action result
    pub fn travel(&mut self, symbol: &M) -> BaseResult<(), Error<M>> {
        let state = self.table.get(self.top).ok_or(Error::StateNotSpecified)?;
        let action = state.get(symbol).ok_or_else(|| {
            let expecteds = state.keys().cloned().collect();
            Error::UnexpectedToken(symbol.clone(), expecteds)
        })?;
        match action {
            Action::Shift(to) => self.shift(*to),
            Action::Goto(to) => self.goto(*to),
            Action::Reduce(name, prod) => self.reduce(&name.clone(), &prod.clone()),
            Action::Acc => self.accept(),
            Action::Conflict(a, b) => Err(Error::Conflict(*a.clone(), *b.clone())),
        }
    }

    /// # Errors
    /// If stack doesn't contains the necessary terms amount, raises an `Error::UnexepectedEof`
    /// If there isn't a previous state, raises an `Error::MissingPreviousState`
    pub fn reduce(&mut self, name: &M, prod: &Production<M>) -> BaseResult<(), Error<M>> {
        let len = self.items.len();
        let items = &self.items[len - prod.0.len()..];
        // TODO: Create a custom Error
        debug_assert!(
            self.reductors.get(name).is_some(),
            "missing reductor table for {name:?}"
        );
        debug_assert!(
            self.reductors[name].get(prod.1).is_some(),
            "missing production {} reductor for {name:?}",
            prod.1
        );
        let new_item = Token::new(self.reductors[name][prod.1](items), name.clone());
        self.items.truncate(len - prod.0.len());
        self.items.push(new_item);

        let len = self.states.len();
        self.top = self.states[len - prod.0.len() - 1];

        self.states.truncate(len - prod.0.len());
        self.travel(name)
    }

    pub fn reset(&mut self) {
        self.finished = false;
        self.states.clear();
        self.items.clear();
        self.top = 0;
    }

    /// # Errors
    /// The same of `dfa::travel`
    pub fn parse(&mut self, input: I) -> BaseResult<T, Error<M>> {
        self.reset();
        self.buffer = input.peekable();
        self.start()?;
        let res = self.items.pop().ok_or(Error::MissingPreviousState)?;
        self.reset();
        Ok(res.item)
    }

    #[must_use]
    pub fn stack_fmt(&self) -> String {
        let mut fmts = Vec::new();
        for i in 0.. {
            if i >= self.states.len() && i >= self.items.len() {
                break;
            }
            if let Some(s) = self.states.get(i) {
                fmts.push(format!("{s}"));
            }

            if let Some(it) = self.items.get(i) {
                fmts.push(format!("{:?}", it.ty));
            }
        }
        fmts.join(" ")
    }

    #[must_use]
    pub fn stack_debug(&self) -> String
    where
        T: std::fmt::Debug,
    {
        let mut fmts = Vec::new();
        for i in 0.. {
            if i >= self.states.len() && i >= self.items.len() {
                break;
            }
            if let Some(s) = self.states.get(i) {
                fmts.push(format!("{s}"));
            }

            if let Some(it) = self.items.get(i) {
                fmts.push(format!("{it:?}"));
            }
        }
        fmts.join(" ")
    }
}
