use std::{iter::Peekable, rc::Rc};

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
    pub fn name(&self) -> Rule {
        match self {
            Self::Simple(n) => n,
            Self::Compound(n, ..) => n,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackEl {
    Item(Item),
    State(usize),
}

#[derive(Debug, Clone)]
pub struct Dfa<I: Iterator<Item = Term>> {
    pub buffer: Peekable<I>,
    pub stack: Vec<StackEl>,
    pub table: ActTable,
    pub top: usize,
    pub finished: Option<bool>,
}

impl<I: Iterator<Item = Term>> Dfa<I> {
    pub fn new(buffer: I, table: ActTable) -> Self {
        Self {
            stack: vec![StackEl::State(0)],
            buffer: buffer.peekable(),
            top: 0,
            table,
            finished: None,
        }
    }

    pub fn shift(&mut self, to: usize) {
        let item = Item::Simple(self.buffer.next().unwrap());
        self.stack.push(StackEl::Item(item));
        self.top = to;
        self.stack.push(StackEl::State(self.top));
    }

    pub fn accept(&mut self) {
        self.finished = Some(self.buffer.peek() == Some(&"$"));
    }

    pub fn start(&mut self) {
        while self.finished.is_none() {
            let symbol = *self.buffer.peek().unwrap_or(&crate::EOF);
            self.travel(symbol);
        }
    }

    pub fn goto(&mut self, to: usize) {
        self.stack.push(StackEl::State(to));
        self.top = to;
    }

    pub fn travel(&mut self, symbol: Term) {
        match &self.table[self.top][symbol] {
            Action::Shift(to) => self.shift(*to),
            Action::Goto(to) => self.goto(*to),
            Action::Reduce(name, prod) => self.reduce(name, prod.clone()),
            Action::Acc => self.accept(),
            Action::Conflict(a, b) => panic!("conflict between {a:?} and {b:?}"),
        }
    }

    pub fn reduce(&mut self, name: RuleName, prod: Rc<Production>) {
        let mut item = Vec::with_capacity(prod.len());
        while item.len() != prod.len() {
            let poped = self.stack.pop().unwrap();
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
            .unwrap();
        self.stack.push(StackEl::Item(item));
        self.top = state;
        self.travel(name);
    }

    pub fn reset(&mut self) {
        self.finished = None;
        self.stack.clear();
        self.stack.push(StackEl::State(0));
        self.top = 0;
    }

    pub fn parse(&mut self, input: I) -> StackEl {
        self.reset();
        self.buffer = input.peekable();
        self.start();
        let res = self.stack[1].clone();
        self.reset();
        res
    }
}
