use crate::{Ast, Meta, Sym};
use lrp::{ReductMap, Span, Token};

pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
    // pub type ReductFn<T, M> = fn(&[Token<T, M>]) -> T;
    // pub type ReductMap<T, M> = Map<M, Vec<ReductFn<T, M>>>;

    let mut map = ReductMap::new();
    map
}
