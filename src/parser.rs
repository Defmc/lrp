use crate::{BaseResult, Error, Grammar, State, Tabler, Token};
use crate::{Dfa, ReductMap};
use std::fmt::{Debug, Display};

pub trait Parser<T>
where
    T: PartialEq + Ord + Clone + Display + Debug,
{
    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[must_use]
    fn new(grammar: Grammar<T>) -> Self
    where
        Self: Sized,
    {
        Self::with_table(Tabler::new(grammar))
    }

    #[must_use]
    fn with_table(table: Tabler<T>) -> Self;

    #[must_use]
    fn uninit(table: Tabler<T>) -> Self;

    #[must_use]
    fn dfa<M, I: Iterator<Item = Token<M, T>>>(
        &self,
        buffer: I,
        maps: ReductMap<M, T>,
    ) -> Dfa<M, T, I>
    where
        M: Clone,
    {
        Dfa::new(
            buffer,
            self.tables().actions.clone(),
            maps,
            self.tables()
                .grammar
                .basis()
                .look
                .iter()
                .take(1)
                .collect::<Vec<_>>()[0]
                .clone(),
        )
    }

    #[must_use]
    fn simple_dfa<I: IntoIterator<Item = Token<(), T>>>(
        &self,
        buffer: I,
    ) -> Dfa<(), T, I::IntoIter> {
        self.dfa(buffer.into_iter(), self.empty::<I::IntoIter>())
    }

    #[must_use]
    fn cloned<I: Iterator<Item = Token<T, T>>>(&self) -> ReductMap<T, T> {
        fn clone<A: Clone>(toks: &[Token<A, A>]) -> A {
            toks[0].ty.clone()
        }
        Dfa::<T, T, I>::transparent(self.tables(), clone::<T>)
    }

    #[must_use]
    fn empty<I: Iterator<Item = Token<(), T>>>(&self) -> ReductMap<(), T> {
        const fn empty<A>(_: &[Token<(), A>]) {}
        Dfa::<(), T, I>::transparent(self.tables(), empty::<T>)
    }

    /// # Errors
    /// The same of `dfa::travel`
    fn parse<M, I: IntoIterator<Item = Token<M, T>>>(
        &self,
        buffer: I,
        maps: ReductMap<M, T>,
    ) -> BaseResult<M, Error<T>>
    where
        M: Clone,
    {
        let mut dfa = self.dfa(buffer.into_iter(), maps);
        dfa.start()?;
        let item = dfa
            .items
            .pop()
            .ok_or(crate::dfa::Error::MissingPreviousState)?;
        Ok(item.item)
    }

    /// Runs `Parser::parse` and checks by errors
    #[must_use]
    fn validate<I: IntoIterator<Item = Token<(), T>>>(&self, buffer: I) -> bool {
        self.parse(buffer, self.empty::<I::IntoIter>()).is_ok()
    }

    #[must_use]
    fn state_from_kernel(&self, kernel: &State<T>) -> Option<usize> {
        let final_kernel = self.final_kernel(kernel)?;
        self.tables().kernels.get(final_kernel).copied()
    }

    #[must_use]
    fn final_kernel<'a>(&'a self, kernel: &'a State<T>) -> Option<&'a State<T>> {
        Some(kernel)
    }

    #[must_use]
    fn merged(states: State<T>) -> State<T> {
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
    fn tables(&self) -> &Tabler<T>;

    #[must_use]
    fn tables_mut(&mut self) -> &mut Tabler<T>;

    fn reduce_equals(&mut self) {
        self.tables_mut().reduce_equals();
    }
}
