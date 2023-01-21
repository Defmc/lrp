use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    /// Specifies the grammar type. e.g:
    /// ```
    ///     grammar lr | lalr | slr;
    /// ```
    #[token("grammar")]
    Grammar,

    /// An identifier:
    /// ab A AB a__ a0219 _a1 _
    #[regex(r"[a-zA-Z_]\w*")]
    Ident,

    /// Specifies which type will be used as token. e.g:
    /// ```
    ///     link wop::Token;
    /// ```
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
    /// ```
    ///     rule TwoIdents = Ident Ident;
    /// ```
    #[token("rule")]
    RuleDecl,

    /// A normal rule. e.g:
    /// ```
    ///     rule GrammarType = "grammar" ("lr" | "lalr" | "slr");
    /// ```
    /// It's the same of:
    /// ```
    ///     rule TermDecls = ("term" str "=" Ident ";")*;
    /// ```
    #[token("=")]
    NormalSpec,

    /// A variadic specific rule. e.g:
    /// ```
    ///     rule TermDecls *= "term" str "=" Ident ";";
    /// ```
    /// It's the same of:
    /// ```
    ///     rule TermDecls = ("term" str "=" Ident ";")*;
    /// ```
    #[token("*=")]
    VarSpec,

    /// A repeated specific rule. e.g:
    /// ```
    ///     rule TermDecls += "term" str "=" Ident ";";
    /// ```
    /// It's the same of:
    /// ```
    ///     rule TermDecls = ("term" str "=" Ident ";")+;
    /// ```
    #[token("+=")]
    RepSpec,

    /// An optional specific rule. e.g:
    /// ```
    ///     rule RuleType ?= "?" | "+" | "*";
    /// ```
    /// It's the same of:
    /// ```
    ///     rule RuleType = ("?" | "+" | "*")?;
    /// ```
    #[token("?=")]
    OptSpec,

    /// Type or ident specifier. Used to define the return type of a rule or a gramem variable name
    /// for parsing expression.
    #[token(":")]
    Type,

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
    /// ```
    ///     rule GrammarType = "grammar" ("lr" | "lalr" | "slr");
    /// ```
    #[token("|")]
    Or,

    /// Context guards. Allows to re-use a production fraction. e.g:
    /// ```
    ///     rule GrammarType = "grammar" ("lr" | "lalr" | "slr");
    /// ```
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    /// Parsing expressions. A piece of Rust code to handle a reduction process. e.g:
    /// ```
    ///     rule Sum: i32 = Int:lhs "+" Int:rhs => { lhs + rhs }
    /// ```
    #[token("=>")]
    Pexpr,

    #[error]
    Error,

    #[regex(r"[ \t\n\f]+", logos::skip)]
    Ws,
}
