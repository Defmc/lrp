use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    /// Specifies the grammar type. e.g:
    /// ``
    ///     grammar lr | lalr | slr;
    /// ``
    #[token("grammar")]
    Grammar,

    /// LR(1) (Canonical LR parser) grammar type.
    /// It's the more powerful, but uses a lot of memory more than LALR(1) or SLR.
    /// As LR(k) can process all CFGs and every LR(k) can be converted to a LR(1) grammar, this
    /// type can parse every CFG grammar.
    #[token("lr")]
    LrParser,

    /// LALR(1) grammar type.
    /// It's the ideal type for almost every grammar. Has a balance between power and memory usage.
    /// Can take more compile-time than LR(1) grammars due to table optimizations and is guaranteed
    /// to have the same SLR parser table size.
    #[token("lalr")]
    LalrParser,

    /// SLR grammar type.
    /// It's the simplest type and has the fatest compile-time.
    #[token("slr")]
    SlrParser,

    /// An identifier:
    /// ab A AB a__ a0219 _a1 _
    #[regex(r"[a-zA-Z_]\w*")]
    Ident,

    /// Specifies which type will be used as token. e.g:
    /// ``
    ///     link wop::Token;
    /// ``
    #[token("link")]
    Link,

    /// Instruction separator.
    #[token(";")]
    Sep,

    /// Path access. e.g:
    /// wop::Token::Grammar
    #[token("::")]
    PathAccess,

    /// Declares a new rule. e.g:
    /// ``
    ///     TwoIdents = Ident Ident;
    /// ``
    #[token("rule")]
    RuleDecl,

    /// A normal rule. e.g:
    /// ``
    ///     GrammarType = "grammar" ("lr" | "lalr" | "slr");
    /// ``
    /// It's the same of:
    /// ``
    ///     TermDecls = ("term" str "=" Ident ";")*;
    /// ``
    #[token("=")]
    NormalSpec,

    /// A variadic specific rule. e.g:
    /// ``
    ///     TermDecls *= "term" str "=" Ident ";";
    /// ``
    /// It's the same of:
    /// ``
    ///     TermDecls = ("term" str "=" Ident ";")*;
    /// ``
    #[token("*=")]
    VarSpec,

    /// A repeated specific rule. e.g:
    /// ``
    ///     TermDecls += "term" str "=" Ident ";";
    /// ``
    /// It's the same of:
    /// ``
    ///     TermDecls = ("term" str "=" Ident ";")+;
    /// ``
    #[token("+=")]
    RepSpec,

    /// An optional specific rule. e.g:
    /// ``
    ///     RuleType ?= "?" | "+" | "*";
    /// ``
    /// It's the same of:
    /// ``
    ///     RuleType = ("?" | "+" | "*")?;
    /// ``
    #[token("?=")]
    OptSpec,

    /// Type or ident specifier. Used to define the return type of a or a gramem variable name
    /// for parsing expression.
    #[token(":")]
    Type,

    /// String literal
    #[regex(r#""([^"\\]|\\.)*""#)]
    StrLit,

    /// Metadata included attribute. Handles a gramem variable as a `lrp::Token`
    /// NOTE: Using it with a `~` attribute creates some order differences:
    /// `~@` means a boxed token;
    /// `@~` means a token with a boxed content.
    #[token("@")]
    MetaAttr,

    /// Auto-box attribute. Assigns to gramem variable an boxed version of the token.
    /// NOTE: Using it with a `@` attribute creates some order differences:
    /// `~@` means a boxed token;
    /// `@~` means a token with a boxed content.
    #[token("~")]
    BoxAttr,

    /// Or production clause. Allows to expand others productions to the same rule. e.g:
    /// ``
    ///     GrammarType = "grammar" ("lr" | "lalr" | "slr");
    /// ``
    #[token("|")]
    Or,

    /// Context guards. Allows to re-use a production fraction. e.g:
    /// ``
    ///     GrammarType = "grammar" ("lr" | "lalr" | "slr");
    /// ``
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    /// Parsing expressions. A piece of Rust code to handle a reduction process. e.g:
    /// ``
    ///     Sum: i32 = Int:lhs "+" Int:rhs => lhs + rhs;;
    /// ``
    #[regex(r#"=>.*;;"#)]
    Pexpr,

    #[error]
    Error,

    /// Whitespace
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Ws,

    /// Block comment
    #[regex(r#"/\*([^\*]|\*[^//])*\*/"#, logos::skip)]
    BlockComment,

    /// Line comment
    #[regex(r#"//[^\n]*\r?"#, logos::skip)]
    LineComment,
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::Token;
    #[test]
    fn strings() {
        let mut toks = Token::lexer(
            r#"
            ""
            "em"
            "\"aaaada"
            "é o crime perfeito\""
            "\"ce ta sempre linda mulher\""
            "\\\"\\\""
            // Just a comment...
            /* thats amazing */
            /* a fake* final block comment */
            NowAnIdent
            "#,
        );
        let mut tokens = vec![
            Token::Ident,
            Token::StrLit,
            Token::StrLit,
            Token::StrLit,
            Token::StrLit,
            Token::StrLit,
            Token::StrLit,
        ];
        while let Some(tok) = toks.next() {
            println!("{tok:?}: {} | {:?}", toks.slice(), toks.span());
            assert_eq!(Some(tok), tokens.pop());
        }
    }
}
