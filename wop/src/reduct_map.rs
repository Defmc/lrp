use crate::{Ast, Meta, Sym};
use lrp::{ReductMap, Token};

pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
    // pub type ReductFn<T, M> = fn(&[Token<T, M>]) -> T;
    // pub type ReductMap<T, M> = Map<M, Vec<ReductFn<T, M>>>;
    /* ReductMap::from([(
        Sym,
        vec![
            fn2_with_ref_to_tokens,
            fn3_with_ref_to_tokens,
        ],
    )]); */

    let mut map = ReductMap::new();
    map.insert(Sym::EntryPoint, vec![entry_point]);
    map
}

fn entry_point(program: &[Token<Meta<Ast>, Sym>]) -> Meta<Ast> {
    let program = program[0].item.clone();
    let (start, end) = (program.start, program.end);
    Meta::new(program.item, (start, end))
}
