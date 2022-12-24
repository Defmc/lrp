use crate::ActTable;

#[derive(Debug, Clone)]
pub enum State {
    Shift(usize),
    Reduce(usize, &'static str, &'static str),
    Acc,
    Error,
}

#[derive(Debug, Clone)]
pub enum Item {
    Simple(&'static str),
    Compound(&'static str, Vec<Item>),
    Empty,
}

impl Item {
    pub fn name(&self) -> Option<&'static str> {
        match self {
            Self::Simple(n) => Some(n),
            Self::Compound(n, ..) => Some(n),
            Self::Empty => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dfa {
    pub stack: Vec<usize>,
    pub buffer: Vec<&'static str>,
    pub forest: Vec<Item>,
    pub table: ActTable,
    pub top: usize,
    pub finished: Option<bool>,
}

impl Dfa {
    pub fn new(buffer: Vec<&'static str>, table: ActTable) -> Self {
        Self {
            stack: vec![0],
            buffer,
            top: 0,
            forest: Vec::new(),
            table,
            finished: None,
        }
    }

    pub fn shift(&mut self, to: usize) {
        println!("shift({to})");
        self.forest.push(Item::Simple(self.buffer.remove(0)));
        self.stack.push(self.top);
        self.top = to;
        println!("shifted {:?}", self.forest.last());
    }

    pub fn accept(&mut self) {
        println!("accept()");
        self.top = self.stack.pop().unwrap();
        self.finished = Some(self.stack.is_empty() && self.buffer.is_empty());
        println!("accepted: {:?}", self.forest.pop().unwrap());
    }

    pub fn start(&mut self, symbol: &'static str) {
        while self.finished.is_none() {
            self.travel(symbol)
        }
    }

    pub fn travel(&mut self, symbol: &'static str) {
        println!("item: {}:{}", self.top, symbol);
        println!("forest: {:?}", self.forest);
        match self.table[self.top][symbol] {
            State::Shift(to) => self.shift(to),
            State::Reduce(to, sym, name) => self.reduce(to, sym, name),
            State::Acc => self.accept(),
            State::Error => panic!("incompatible input"),
        }
    }

    pub fn reduce(&mut self, qty: usize, next_sym: &'static str, name: &'static str) {
        println!("reduce({qty}, {next_sym}, {name})");
        let mut prod = Vec::with_capacity(qty);
        for _ in 0..qty {
            self.top = self.stack.pop().unwrap();
            prod.push(self.forest.pop().unwrap());
        }
        let item = Item::Compound(name, prod);
        println!("reduced {item:?}");
        self.forest.push(item);
        self.travel(next_sym);
    }
}
