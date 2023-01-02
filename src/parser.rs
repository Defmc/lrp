use crate::{
    transitive, Action, Dfa, Grammar, Item, Map, Position, Rule, StackEl, State, Tabler, Term,
};

pub trait Parser {
    #[must_use]
    fn new(grammar: Grammar) -> Self;

    fn proc_actions(&mut self);

    #[must_use]
    fn dfa<I: Iterator<Item = Term>>(&self, buffer: I) -> Dfa<I> {
        Dfa::new(buffer, self.tables().actions.clone())
    }

    #[must_use]
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

    #[must_use]
    fn validate<I: IntoIterator<Item = Term>>(&self, buffer: I) -> bool {
        let a = self.parse(buffer);
        // TODO: Error handling
        true
    }

    #[must_use]
    fn closure(&self, state: State) -> State;

    #[must_use]
    fn prop_closure(&self, state: State) -> State {
        Self::merged(transitive(state, |s| self.closure(s)))
    }

    #[must_use]
    fn merged(states: State) -> State {
        let mut new = State::new();
        'outter: for state in states {
            let keys: Vec<_> = new.iter().cloned().collect();
            for key in keys {
                if new.get(&key).unwrap().body_eq(&state) {
                    let mut state = state;
                    state.look.extend(new.get(&key).unwrap().look.clone());
                    new.remove(&key);
                    new.insert(state);
                    continue 'outter;
                }
            }
            new.insert(state);
        }
        new
    }

    fn proc_closures(&mut self);

    #[must_use]
    fn goto(&self, kernels: State, sym: &Term) -> Option<(State, State)>;

    #[must_use]
    fn tables(&self) -> &Tabler;

    #[must_use]
    fn tables_mut(&mut self) -> &mut Tabler;

    #[must_use]
    fn decision(&self, start: Rule, pos: &Position, row: &State) -> Map<Term, Action>;
}