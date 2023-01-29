use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Sym {
    /// An identifier:
    /// ab A AB a__ a0219 _a1 _
    #[regex(r"[a-zA-Z_]\w*")]
    Ident,

    /// Includes a Rust type to the current context. e.g:
    /// ``
    ///     use wop::Token;
    /// ``
    #[token(use)]
    Use,

    /// Instruction separator.
    #[token(";")]
    Sep,

    /// Path access. e.g:
    /// wop::Token::Grammar
    #[token("::")]
    PathAccess,

    /// A normal rule. e.g:
    /// ``
    ///     GrammarType = "grammar" ("lr" | "lalr" | "slr");
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

    /// Auto-box attribute. Handles a grammem variable as a `Box<lrp::Token>`.
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
    CodeExpr,

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

    /// Terms
    AssignOp,
    AttrPrefix,
    AttrSuffix,
    VarPipe,
    TypeDecl,
    IdentPath,
    Elm,
    Prod,
    RulePipe,
    TokenDecl,
    UseDecl,
    RuleDecl,
    Declaration,
    Program,
}

use lrp::{Grammar, Map, Sym};
pub fn grammar() {
    let rules = lrp::Map::from([()]);
    Grammar::new()
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
