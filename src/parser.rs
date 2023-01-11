use crate::{dfa::Result, Dfa};
use crate::{Grammar, Item, StackEl, State, Tabler, Term};

pub trait Parser {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[must_use]
    fn new(grammar: Grammar) -> Self
    where
        Self: Sized,
    {
        Self::with_table(Tabler::new(grammar))
    }

    #[must_use]
    fn with_table(table: Tabler) -> Self;

    #[must_use]
    fn uninit(table: Tabler) -> Self;

    #[must_use]
    fn dfa<I: Iterator<Item = Term>>(&self, buffer: I) -> Dfa<I> {
        Dfa::new(buffer, self.tables().actions.clone())
    }

    /// # Errors
    /// The same of `dfa::travel`
    fn parse<I: IntoIterator<Item = Term>>(&self, buffer: I) -> Result<Item> {
        let mut dfa = self.dfa(buffer.into_iter());
        dfa.start()?;
        let secnd = dfa.stack.swap_remove(1);
        if let StackEl::Item(item) = secnd {
            Ok(item)
        } else {
            Err(crate::dfa::Error::MissingPreviousState)
        }
    }

    /// Runs `Parser::parse` and checks by errors
    #[must_use]
    fn validate<I: IntoIterator<Item = Term>>(&self, buffer: I) -> bool {
        self.parse(buffer).is_ok()
    }

    #[must_use]
    fn state_from_kernel(&self, kernel: &State) -> Option<usize> {
        let final_kernel = self.final_kernel(kernel)?;
        self.tables().kernels.get(final_kernel).copied()
    }

    #[must_use]
    fn final_kernel<'a>(&'a self, kernel: &'a State) -> Option<&'a State> {
        Some(kernel)
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

    #[must_use]
    fn tables(&self) -> &Tabler;

    #[must_use]
    fn tables_mut(&mut self) -> &mut Tabler;

    fn reduce_equals(&mut self) {
        self.tables_mut().reduce_equals();
    }
}
