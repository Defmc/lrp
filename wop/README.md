```hs
use my_crate::Ast;
use my_crate::Sym;

alias Number = Sym::Number;
alias "+" = Sym::Add;
alias "(" = Sym::OpenParen;
alias ")" = Sym::CloseParen;

// :alias for codeblock (REQUIRED), * for clone it
Ast::Add: Ast = Number:*n1 "+" Number:*n2 {
    Ast::Add(n1, n2)
};

Ast::Expr: Ast = Sym::Add:*a {
/* Meta::new( */
    Ast:Expr(a)/*,
    Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end)
)*/
}
    | "(" Sym::Expr:*e ")" {
    Ast::Expr(e)
};

Ast::EntryPoint: Ast = Sym::Expr:*e {
    Ast::EntryPoint(e)
}
```
To use, just build it from `Builder` and call `builder.dump_grammar()` to generate the code for the grammar:
```rs
format!("let grammar = Grammar::new(Sym::EntryPoint, {}, Sym::Eof)", builder.dump_grammar())
```

And for DFA building, remember to dump the reduct_map:
```rs
format!("parser.dfa(buf, {})", builder.dump_reduct_map())
```

Ideas:
- [x] `Builder::dump_grammar` // should return a `RuleMap`
- [] For composite rules (with `( x | y)`), the codeblock will be applied to both
