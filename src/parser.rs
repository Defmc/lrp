use crate::{ActTable, Dfa, Grammar, Item, StackEl, Term};

pub trait Parser {
    fn new(grammar: Grammar) -> Self;

    fn proc_actions(&mut self);

    fn dfa<I: Iterator<Item = Term>>(&self, buffer: I) -> Dfa<I> {
        Dfa::new(buffer, self.actions().clone())
    }

    fn parse<I: IntoIterator<Item = Term>>(&self, buffer: I) -> Item {
        let mut dfa = self.dfa(buffer.into_iter());
        dfa.start();
        let secnd = dfa.stack.swap_remove(1);
        if let StackEl::Item(item) = secnd {
            item
        } else {
            panic!("unexpected state")
        }
    }

    fn actions(&self) -> &ActTable;
    fn gotos(&self) -> &ActTable;
}
