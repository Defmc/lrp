use builder::SrcRef;
use logos::Logos;
use lrp::{Meta, Span};

pub mod builder;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Token(Sym),
    EntryPoint(Box<Gramem>),
    Program(Vec<Gramem /* Ast::RuleDecl | Ast::Import | Ast::Alias */>),
    RuleDecl(RuleDecl),
    Rule(Vec<RulePipe>),
    RulePipe(Vec<Gramem>),
    RuleItem(
        /* item */ Box<Gramem>,
        /* optional */ bool,
        /* alias */ Option<SrcRef>,
    ),
    Import(SrcRef),
    Alias(SrcRef, SrcRef),
    IdentPath(SrcRef),
}

impl Ast {
    #[must_use]
    pub const fn get_src_ref(&self) -> Option<SrcRef> {
        match self {
            Self::IdentPath(o) => Some(*o),
            _ => None,
        }
    }
}

pub type RuleDecl = (SrcRef, SrcRef, Vec<RulePipe>);
pub type RulePipe = (Vec<Gramem>, SrcRef);
pub type Gramem = Token<Meta<Ast>, Sym>;

#[derive(Logos, Debug, PartialEq, PartialOrd, Clone, Copy, Ord, Eq)]
pub enum Sym {
    #[token("alias")]
    AliasWord,

    #[token("use")]
    UseWord,

    #[token("=")]
    Assign,

    #[token("|")]
    Pipe,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token(";")]
    Sc,

    #[token("::")]
    PathAccess,

    #[token(":")]
    TwoDots,

    #[token("*")]
    Glob,

    #[token("?")]
    Optional,

    #[regex(r#"[a-zA-Z_]\w*"#)]
    Ident,

    /// String literal
    #[regex(r#""([^"\\]|\\.)*""#)]
    StrLit,

    /// Block comment
    #[regex(r#"/\*([^\*]|\*[^//])*\*/"#, logos::skip)]
    BlockComment,

    /// Line comment
    #[regex(r#"//[^\n]*\r?"#, logos::skip)]
    LineComment,

    /// Matches a entire function tail definition: arrow ("->"), return type and a expression
    #[regex(r#"(->)([^\}]|\}[^%])*\}%"#)]
    CodeBlock,

    #[error]
    Error,

    /// Whitespace
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Ws,

    /// End of file
    Eof,

    // Nonterminals
    Token,
    EntryPoint,
    Program,
    Rule,
    Import,
    Alias,
    RulePipe,
    RuleDecl,
    IdentPath,
    RuleItem,
}

use lrp::{Dfa, Parser, Slr, Token};
pub mod out;

pub fn lexer<'source>(
    source: &'source <Sym as Logos>::Source,
) -> impl Iterator<Item = Gramem> + 'source {
    Sym::lexer(source)
        .spanned()
        .map(|(t, s)| Token::new(Meta::new(Ast::Token(t), Span::new(s.start, s.end)), t))
}

#[must_use]
pub fn build_parser<I: Iterator<Item = Gramem>>(buffer: I) -> Dfa<Meta<Ast>, Sym, I> {
    let parser = Slr::new(out::grammar());
    parser.dfa(buffer, out::reduct_map())
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::Sym;
    #[test]
    fn strings() {
        let lex = Sym::lexer(
            r#"
            ""
            "em"
            "\"aaaada"
            "é o crime perfeito\""
            "\"ce ta sempre linda mulher\""
            "\\\"\\\""
            "// a fake comment"
            "/* another fake comment */"
            // Just a comment...
            /* thats amazing */
            /* a fake* final block comment */
            NowAnIdent
            "#,
        );
        const SYMBOLS: &[Sym] = &[
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::Ident,
        ];
        SYMBOLS.iter().zip(lex).for_each(|(&l, s)| assert_eq!(l, s));
    }

    #[test]
    fn bootstrap() {
        Sym::lexer(include_str!("wop.grammar")).for_each(|tk| assert_ne!(tk, Sym::Error));
    }
}
