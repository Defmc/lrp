use logos::Logos;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum Ast {
    Token(Sym),
    EntryPoint(Box<Ast /* Ast::Program */>),
    Program(Vec<Ast /* Ast::Declaration */>),
    Declaration(Box<Ast /* Ast::TokenDecl | Ast::UseDecl | Ast::Ruledecl */>),
    TokenDecl(Sym, Box<Ast /* Ast::IdentPath */>),
    IdentPath(Vec<Sym /* Sym::Ident */>),
    UseDecl(Box<Ast /* Ast::IdentPath */>),
    AssignOp(Sym /* "*=" | "+=" | "?=" | "=" */),
    AttrPrefix(Vec<Sym /* "@" | "~" */>),
    AttrSuffix(Sym /* "?" | "*" | "+" */),
    VarPipe(Sym /* Sym::Ident */),
    TypeDecl(Box<Ast /* Ast::IdentPath */>),
    Elm(
        Option<Box<Ast /* Ast::AttrPrefix */>>,
        Box<Ast /* Ast::ElmBase */>,
        Option<Box<Ast /* Ast::AttrSuffix */>>,
    ),
    Prod(Vec<(Ast /* Ast::Elm */, Option<Sym /* "|" */>)>),
    RulePipeRepeater(Vec<Ast /* Ast::Prod */>),
    RulePipe(Vec<Ast /* Ast::Prod */>),
    RuleDecl(
        Sym, /* Sym::Ident */
        Option<Box<Ast /* Ast::TypeDecl */>>,
        Box<Ast /* Ast::AssignOp */>,
        Box<Ast /* Ast::RulePipe */>,
    ),
    ElmBase(
        Sym, /* Sym::Ident */
        Option<Box<Ast /* Ast::VarPipe */>>,
        Box<Ast /* Ast::RulePipe */>,
        Option<Box<Ast /* Ast::VarPipe */>>,
    ),
}

#[derive(Logos, Debug, PartialEq, PartialOrd, Clone, Ord, Eq)]
pub enum Sym {
    /// An identifier:
    /// ab A AB a__ a0219 _a1 _
    #[regex(r"[a-zA-Z_]\w*")]
    Ident,

    /// An token link. A literal string that acts like a substitution macro.
    #[token("token")]
    TokenWord,

    /// Includes a Rust type to the current context. e.g:
    /// ``
    ///     use wop::Token;
    /// ``
    #[token("use")]
    UseWord,

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

    /// An optional member. Something that can be parsed as empty.
    /// ``
    /// Optional = Digit | ();
    /// ``
    /// Is the same of:
    /// ``
    /// Optional = Digit?;
    /// ``
    #[token("?")]
    Opt,

    /// A repeated member. Something that can occur one or more times.
    /// ``
    /// Variadic = Digit Digit*;
    /// ``
    /// Is the same of:
    /// ``
    /// Variadic = Digit+;
    /// ``
    #[token("+")]
    Rep,

    /// A variadic member. Something that can occur one time, more than one time, or never.
    /// ``
    /// Variadic = () | Digit+;
    /// ``
    /// Is the same of:
    /// ``
    /// Variadic = Digit*;
    /// ``
    #[token("*")]
    Var,

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

    Eof,

    /// Non terminals
    AssignOp,
    AttrPrefix,
    AttrSuffix,
    VarPipe,
    TypeDecl,
    IdentPath,
    Elm,
    Prod,
    RulePipe,
    RulePipeRepeater,
    TokenDecl,
    UseDecl,
    RuleDecl,
    Declaration,
    Program,
    EntryPoint,
    ElmBase,
}

use lrp::{Dfa, Grammar, Parser, Slr, Token};

#[must_use]
pub fn grammar() -> Grammar<Sym> {
    #[allow(clippy::enum_glob_use)]
    use Sym::*;

    let rules = lrp::grammar_map! {
    EntryPoint -> Program,

    Program -> Program Declaration Sep
        | Declaration Sep,

    Declaration -> TokenDecl | UseDecl | RuleDecl,

    TokenDecl -> TokenWord StrLit IdentPath | TokenWord Ident IdentPath,

    IdentPath -> IdentPath PathAccess Ident | Ident,

    UseDecl -> UseWord IdentPath,

    AssignOp -> VarSpec | RepSpec | OptSpec | NormalSpec,

    AttrPrefix -> MetaAttr | BoxAttr | MetaAttr AttrPrefix | BoxAttr AttrPrefix,

    AttrSuffix -> Opt | Var | Rep,

    VarPipe -> Type Ident,

    TypeDecl -> Type IdentPath,

    ElmBase -> Ident VarPipe
        | Ident
        | StrLit VarPipe
        | StrLit
        | OpenParen RulePipe CloseParen
        | OpenParen RulePipe CloseParen VarPipe,

    Elm -> AttrPrefix ElmBase AttrSuffix
        | ElmBase AttrSuffix
        | AttrPrefix ElmBase
        | ElmBase,

    Prod -> Prod Elm
        | Prod Elm CodeExpr
        | Elm CodeExpr
        | Elm,

    RulePipeRepeater -> RulePipeRepeater Prod Or
        | Prod Or,

    RulePipe -> RulePipeRepeater Prod
        | Prod,

    RuleDecl -> Ident TypeDecl AssignOp RulePipe
        | Ident AssignOp RulePipe
    };

    Grammar::new(EntryPoint, rules, Eof)
}

pub mod reduct_map;

#[must_use]
pub fn lexer<'source>(
    source: &'source <Sym as Logos>::Source,
) -> impl Iterator<Item = Token<Ast, Sym>> + 'source {
    Sym::lexer(source.as_ref()).map(|s| Token::new(Ast::Token(s.clone()), s))
}

#[must_use]
pub fn build_parser<I: Iterator<Item = Token<(), Sym>>>(buffer: I) -> Dfa<(), Sym, I> {
    let parser = Slr::new(grammar());
    parser.dfa(buffer, reduct_map::reduct_map())
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
