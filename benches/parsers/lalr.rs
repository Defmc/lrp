use crate::gen_bench_table_construction;
use lrp::{Lalr, Parser};

gen_bench_table_construction! {
    Lalr,
    dragon_book,
    serokell,
    ucalgary_uni_oth_lr1,
    wikipedia,
    puncs
}
