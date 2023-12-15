#  An easy-to-use, compilation-driven interface for the `lrp` parser library
Writing direct grammars sucks: handling the lexer's input, creating tiny different rule productions, creating the reductor table and some other stuffs that we don't care about. This library is focused on simplifying this process, the only external thing you need is a lexer like [logos](https://crates.io/crates/logos)

## A grammar example
```cpp
use my_crate::Ast;
use my_crate::Sym::*; // Each module should be imported separetely (there's no { } support yet)

// you can define alias for idents and string literals
alias Number = Number;
alias "+" = Add;
alias "(" = OpenParen;
alias ")" = CloseParen;

Add: Ast = Number:&n1 "+" Number:&n2 -> {
    Ast::Add(n1, n2)
}%;

Expr: Ast = Sym::Add:&a -> {
/* Meta::new( */
    Ast::Expr(a)/*,
    Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end)
)*/
}%
    | "(" Sym::Expr:&e ")" -> {
    Ast::Expr(e)
}%;

EntryPoint: Ast = Sym::Expr:&e -> {
    Ast::EntryPoint(e)
}%
```

## Building it
Just use `parse` or `from_str` for `Builder`:
```rs
let src = include_str!("your.grammar");
let builder1 = src.parse::<wop::Builder>();
let builder2 = wop::Builder::from_str(src);
assert_eq!(builder1, builder2);
```

## Using it
To use it, just build it from `Builder` and call `builder.dump_grammar()` to generate the code for the grammar:
```rs
format!("let grammar = Grammar::new(Sym::EntryPoint, {}, Sym::Eof)", builder.dump_grammar(src)) // `src` is the source code for the grammar we used above
```

And for DFA building, remember to dump the reduct_map:
```rs
format!("parser.dfa(buf, {})", builder.dump_reduct_map(src))
```

Fun fact: This project uses itself.

Ideas:
- [x] `Builder::dump_grammar` // should return a `RuleMap`
- [x] For composite rules (with `( x | y)`), the codeblock will be applied to both
- [x] Impl codeblocks
- [x] Impl item alias
- [x] Impl optional item
- [x] Allow to set custom slice entry
- [] Impl item alias cloning
- [x] Allow custom token entries
