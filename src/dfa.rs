use std::{fmt, iter::Peekable, rc::Rc};

use crate::{ActTable, Production, Token};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action<T> {
    Shift(usize),
    Goto(usize),
    Reduce(T, Rc<Production<T>>),
    Acc,
    Conflict(Box<Action<T>>, Box<Action<T>>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item<T, M>
where
    M: fmt::Debug + fmt::Display,
{
    Simple(Token<T, M>),
    Compound(Token<T, M>, Vec<Item<T, M>>),
}

impl<T, M> Item<T, M>
where
    M: fmt::Debug + fmt::Display,
{
    #[must_use]
    pub const fn name(&self) -> &M {
        &self.token().ty
    }

    #[must_use]
    pub const fn source(&self) -> &T {
        &self.token().item
    }

    #[must_use]
    pub const fn token(&self) -> &Token<T, M> {
        match self {
            Self::Simple(n) | Self::Compound(n, ..) => n,
        }
    }
}

impl<T, M> fmt::Display for Item<T, M>
where
    M: fmt::Debug + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}", self.name()))?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error<T>
where
    T: fmt::Debug + fmt::Display,
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
    T: fmt::Debug + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(found, expected) => f.write_fmt(format_args!(
                "unexpected token {found}. expected {expected:?}"
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

#[derive(Debug, Clone)]
pub struct Dfa<M, T, I: Iterator<Item = Token<M, T>>>
where
    T: fmt::Debug + fmt::Display + Clone,
    M: Clone,
{
    pub buffer: Peekable<I>,
    pub states: Vec<usize>,
    pub items: Vec<Item<M, T>>,
    pub table: ActTable<T>,
    pub top: usize,
    pub finished: bool,
    pub eof: T,
}

impl<M, T, I: Iterator<Item = Token<M, T>>> Dfa<M, T, I>
where
    T: fmt::Debug + fmt::Display + Clone + Ord,
    M: Clone,
{
    #[must_use]
    pub fn new(buffer: I, table: ActTable<T>, eof: T) -> Self {
        Self {
            states: vec![0],
            items: Vec::new(),
            buffer: buffer.peekable(),
            top: 0,
            table,
            finished: false,
            eof,
        }
    }

    /// # Errors
    /// When there is no more data in buffer, raises an `Error::UnexepectedEof`
    pub fn shift(&mut self, to: usize) -> BaseResult<(), Error<T>> {
        let item = self.buffer.next().ok_or(Error::UnexpectedEof)?;
        self.items.push(Item::Simple(item));
        self.top = to;
        self.states.push(self.top);
        Ok(())
    }

    /// # Errors
    /// When finished parse without consume entire buffer, raises an `Error::IncompleteExec`
    pub fn accept(&mut self) -> BaseResult<(), Error<T>> {
        self.finished = true;
        if self.buffer.peek().is_none() {
            Ok(())
        } else {
            Err(Error::IncompleteExec)
        }
    }

    /// # Errors
    /// The same of `dfa::travel`
    pub fn start(&mut self) -> BaseResult<(), Error<T>> {
        self.trace(|_| {})
    }

    /// # Errors
    /// The same of `dfa::travel`
    pub fn trace(&mut self, mut f: impl FnMut(&mut Self)) -> BaseResult<(), Error<T>> {
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
    pub fn goto(&mut self, to: usize) -> BaseResult<(), Error<T>> {
        self.states.push(to);
        self.top = to;
        Ok(())
    }

    /// # Errors
    /// If the current state don't exists in actions table, raises an `Error::StateNotSpecified`
    /// If there isn't an action in current state for `symbol`, raises an `Error::UnexpectedToken`
    /// Returns the action result
    pub fn travel(&mut self, symbol: &T) -> BaseResult<(), Error<T>> {
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
    pub fn reduce(&mut self, name: &T, prod: &Production<T>) -> BaseResult<(), Error<T>> {
        // let mut item = Vec::with_capacity(prod.len());
        // while item.len() != prod.len() {
        //     let poped = self.stack.pop().ok_or(Error::UnexpectedEof)?;
        //     if let StackEl::Item(i) = poped {
        //         item.push(i);
        //     }
        // }
        let len = self.items.len();
        let items = self.items.split_off(len - prod.0.len());
        // TODO: PEG
        self.items.push(Item::Compound(
            Token::new(items[0].source().clone(), name.clone()),
            items,
        ));
        // TODO: Use `set_len`, once it can't be extended:
        // len - prod.len() <= len
        let len = self.states.len();
        self.top = *self
            .states
            .get(len - prod.0.len() - 1)
            .ok_or(Error::MissingPreviousState)?;
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
    pub fn parse(&mut self, input: I) -> BaseResult<Item<M, T>, Error<T>> {
        self.reset();
        self.buffer = input.peekable();
        self.start()?;
        let res = self.items.pop().ok_or(Error::MissingPreviousState)?;
        self.reset();
        Ok(res)
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
                fmts.push(format!("{it}"));
            }
        }
        fmts.join(" ")
    }
}
