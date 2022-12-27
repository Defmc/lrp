use crate::{ActTable, Position, Rule, Term};

#[derive(Debug, Clone)]
pub enum Action {
    Shift(usize),
    Goto(usize),
    Reduce(Position),
    Acc,
    Conflict(Box<Action>, Box<Action>),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum StackEl {
    Item(Item),
    State(usize),
}

#[derive(Debug, Clone)]
pub struct Dfa {
    pub buffer: Vec<Term>,
    pub stack: Vec<StackEl>,
    pub table: ActTable,
    pub top: usize,
    pub finished: Option<bool>,
}

impl Dfa {
    pub fn new(buffer: Vec<Term>, table: ActTable) -> Self {
        Self {
            stack: vec![StackEl::State(0)],
            buffer,
            top: 0,
            table,
            finished: None,
        }
    }

    pub fn shift(&mut self, to: usize) {
        println!("shift({to})");
        let item = Item::Simple(self.buffer.remove(0));
        println!("shifted {} and {}", item.name(), self.top);
        self.stack.push(StackEl::Item(item));
        self.top = to;
        self.stack.push(StackEl::State(self.top));
    }

    pub fn accept(&mut self) {
        println!("accept()");
        self.finished = Some(&self.buffer == &["$"]);
        println!("accepted: {:?}", self.stack);
    }

    pub fn start(&mut self) {
        while self.finished.is_none() {
            let symbol = self.buffer[0];
            print!("\nstack: ");
            for item in &self.stack {
                print!(
                    "{} ",
                    match item {
                        StackEl::State(n) => format!("{n}"),
                        StackEl::Item(i) => i.name().into(),
                    }
                );
            }
            println!("\ntravelling to {}:{symbol}", self.top);
            self.travel(symbol);
        }
    }

    pub fn goto(&mut self, to: usize) {
        println!("goto({to})");
        self.stack.push(StackEl::State(to));
        self.top = to;
    }

    pub fn travel(&mut self, symbol: Term) {
        print!("\nstack: ");
        for item in &self.stack {
            print!(
                "{} ",
                match item {
                    StackEl::State(n) => format!("{n}"),
                    StackEl::Item(i) => i.name().into(),
                }
            );
        }
        println!("\ntravelling to {}:{symbol}", self.top);
        match &self.table[self.top][symbol] {
            Action::Shift(to) => self.shift(*to),
            Action::Goto(to) => self.goto(*to),
            Action::Reduce(r) => self.reduce(r.clone()),
            Action::Acc => self.accept(),
            Action::Conflict(a, b) => panic!("conflict between {a:?} and {b:?}"),
        }
    }

    pub fn reduce(&mut self, rule: Position) {
        println!("reduce({rule})");
        let mut prod = Vec::with_capacity(rule.seq.len());
        while prod.len() != rule.seq.len() {
            let poped = self.stack.pop().unwrap();
            if let StackEl::Item(i) = poped {
                prod.push(i);
            }
        }
        let item = Item::Compound(rule.rule, prod);
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
        println!("reduced {item:?}");
        self.stack.push(StackEl::Item(item));
        self.top = state;
        self.travel(rule.rule);
    }
}
