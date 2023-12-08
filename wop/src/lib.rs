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
    RuleItem(SrcRef),
    Import(SrcRef),
    Alias(SrcRef, SrcRef),
    IdentPath(SrcRef),
}

impl Ast {
    pub fn get_src_ref(&self) -> Option<SrcRef> {
        match self {
            Self::RuleItem(o) => Some(*o),
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

use lrp::{Dfa, Grammar, Parser, Slr, Token};
use reduct_map::reduct_map;

#[must_use]
pub fn grammar() -> Grammar<Sym> {
    #[allow(clippy::enum_glob_use)]
    use Sym::*;

    let rules = lrp::grammar_map! {
    EntryPoint -> Program,

    Program -> Program Alias Sc
        |  Program Import Sc
        |  Program RuleDecl Sc
        |  Alias Sc
        |  Import Sc
        |  RuleDecl Sc,

    IdentPath -> IdentPath PathAccess Ident
        | Ident,

    RuleDecl -> IdentPath TwoDots IdentPath Assign Rule,

    Rule -> Rule Pipe RulePipe CodeBlock
        | RulePipe CodeBlock,

    RulePipe -> RulePipe RuleItem | RuleItem,

    RuleItem -> IdentPath
        | StrLit,

    Import -> UseWord IdentPath PathAccess Glob
        | UseWord IdentPath,

    Alias -> AliasWord Ident IdentPath
        | AliasWord StrLit IdentPath
    };

    Grammar::new(EntryPoint, rules, Eof)
}

pub mod reduct_map;

pub fn lexer<'source>(
    source: &'source <Sym as Logos>::Source,
) -> impl Iterator<Item = Gramem> + 'source {
    Sym::lexer(source).spanned().map(|(t, s)| {
        Token::new(
            Meta::new(Ast::Token(t.clone()), Span::new(s.start, s.end)),
            t,
        )
    })
}

#[must_use]
pub fn build_parser<I: Iterator<Item = Gramem>>(buffer: I) -> Dfa<Meta<Ast>, Sym, I> {
    let parser = Slr::new(grammar());
    parser.dfa(buffer, reduct_map())
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::Sym;
    #[test]
    fn strings() {
        let mut toks = Sym::lexer(
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
            Sym::Ident,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
            Sym::StrLit,
        ];
        while let Some(tok) = toks.next() {
            println!("{tok:?}: {} | {:?}", toks.slice(), toks.span());
            assert_eq!(Some(tok), tokens.pop());
        }
    }
}
