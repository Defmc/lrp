pub mod clr;
pub mod lalr;

bencher::benchmark_group!(
    table_gens,
    // clr::dragon_book,
    lalr::dragon_book,
    // clr::serokell,
    lalr::serokell,
    // clr::ucalgary_uni_oth_lr1,
    lalr::ucalgary_uni_oth_lr1,
    // clr::wikipedia,
    lalr::wikipedia,
    // clr::puncs,
    lalr::puncs
);
