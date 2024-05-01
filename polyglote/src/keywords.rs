use strum_macros::{AsRefStr, Display, EnumCount, EnumIter, EnumString};

#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum JavaKeyword {
    // Java::LBRACE => "{",
    #[strum(serialize = "{")]
    LBrace,
    // Java::RBRACE => "}",
    #[strum(serialize = "}")]
    RBrace,
    // Java::LPAREN => "(",
    #[strum(serialize = "(")]
    LParen,
    // Java::RPAREN => ")",
    #[strum(serialize = ")")]
    RParen,
    // Java::LBRACK => "[",
    #[strum(serialize = "[")]
    LBracket,
    // Java::RBRACK => "]",
    #[strum(serialize = "]")]
    RBracket,
    // Java::SEMI => ";",
    #[strum(serialize = ";")]
    SemiColon,
    // Java::COMMA => ",",
    #[strum(serialize = ",")]
    Comma,
    // Java::DOT => ".",
    #[strum(serialize = ".")]
    Dot,
    // Java::PLUS => "+",
    #[strum(serialize = "+")]
    Plus,
    // Java::DASH => "-",
    #[strum(serialize = "-")]
    Dash,
    // Java::STAR => "*",
    #[strum(serialize = "*")]
    Star,
    // Java::SLASH => "/",
    #[strum(serialize = "/")]
    Slash,
    // Java::PERCENT => "%",
    #[strum(serialize = "%")]
    Percent,
    // Java::BANG => "!",
    #[strum(serialize = "!")]
    Bang,
    // Java::GT => ">",
    #[strum(serialize = ">")]
    GT,
    // Java::LT => "<",
    #[strum(serialize = "<")]
    LT,
    // Java::GTEQ => ">=",
    #[strum(serialize = ">=")]
    GTEq,
    // Java::LTEQ => "<=",
    #[strum(serialize = "<=")]
    LTEq,
    // Java::EQEQ => "==",
    #[strum(serialize = "==")]
    EqEq,
    // Java::BANGEQ => "!=",
    #[strum(serialize = "!=")]
    BangEq,
    // Java::AMPAMP => "&&",
    #[strum(serialize = "&&")]
    AmpAmp,
    // Java::PIPEPIPE => "||",
    #[strum(serialize = "||")]
    PipePipe,
    // Java::QMARK => "?",
    #[strum(serialize = "?")]
    QMark,
    // Java::COLON => ":",
    #[strum(serialize = ":")]
    Colon,
    // Java::EQ => "=",
    #[strum(serialize = "=")]
    Eq,
    // Java::PLUSEQ => "+=",
    #[strum(serialize = "+=")]
    PlusEq,
    // Java::DASHEQ => "-=",
    #[strum(serialize = "-=")]
    DashEq,
    // Java::STAREQ => "*=",
    #[strum(serialize = "*=")]
    StarEq,
    // Java::SLASHEQ => "/=",
    #[strum(serialize = "/=")]
    SlashEq,
    // Java::AMPEQ => "&=",
    #[strum(serialize = "&=")]
    AmpEq,
    // Java::PIPEEQ => "|=",
    #[strum(serialize = "|=")]
    PipeEq,
    // Java::CARETEQ => "^=",
    #[strum(serialize = "^=")]
    CaretEq,
    // Java::PERCENTEQ => "%=",
    #[strum(serialize = "%=")]
    PercentEq,
    // Java::LTLTEQ => "<<=",
    #[strum(serialize = "<<=")]
    LtLtEq,
    // Java::GTGTEQ => ">>=",
    #[strum(serialize = ">>=")]
    GtGtEq,
    // Java::GTGTGTEQ => ">>>=",
    #[strum(serialize = ">>>=")]
    GtGtGtEq,
    // Java::AMP => "&",
    #[strum(serialize = "&")]
    Amp,
    // Java::PIPE => "|",
    #[strum(serialize = "|")]
    Pipe,
    // Java::CARET => "^",
    #[strum(serialize = "^")]
    Caret,
    // Java::LTLT => "<<",
    #[strum(serialize = "<<")]
    LtLt,
    // Java::GTGT => ">>",
    #[strum(serialize = ">>")]
    GtGt,
    // Java::GTGTGT => ">>>",
    #[strum(serialize = ">>>")]
    GtGtGt,
    // Java::DASHGT => "->",
    #[strum(serialize = "->")]
    DashGt,
    // Java::TILDE => "~",
    #[strum(serialize = "~")]
    Tilde,
    // Java::PLUSPLUS => "++",
    #[strum(serialize = "++")]
    PlusPlus,
    // Java::DASHDASH => "--",
    #[strum(serialize = "--")]
    DashDash,
    // Java::AT => "@",
    #[strum(serialize = "@")]
    At,
    // Java::COLONCOLON => "::",
    #[strum(serialize = "::")]
    ColonColon,
    True,
    False,
    New,
    Instanceof,
    Final,
    Class,
    Extends,
    Switch,
    Case,
    Default,
    Assert,
    Do,
    While,
    Break,
    Continue,
    Return,
    Yield,
    Synchronized,
    Throw,
    Try,
    Catch,
    Finally,
    If,
    Else,
    For,
    End,
    // # Concrete
    // Java::Identifier => "identifier",
    // Java::DecimalIntegerLiteral => "decimal_integer_literal",
    // Java::HexIntegerLiteral => "hex_integer_literal",
    // Java::OctalIntegerLiteral => "octal_integer_literal",
    // Java::BinaryIntegerLiteral => "binary_integer_literal",
    // Java::DecimalFloatingPointLiteral => "decimal_floating_point_literal",
    // Java::HexFloatingPointLiteral => "hex_floating_point_literal",
    // Java::CharacterLiteral => "character_literal",
    // Java::StringLiteral => "string_literal",
    // Java::TextBlock => "text_block",
    // Java::NullLiteral => "null_literal",
}
#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum CppKeyword {
    #[strum(serialize = "\n")]
    NewLine,
    // Cpp::LBRACE => "{",
    #[strum(serialize = "{")]
    LBrace,
    // Cpp::RBRACE => "}",
    #[strum(serialize = "}")]
    RBrace,
    // Cpp::LPAREN => "(",
    #[strum(serialize = "(")]
    LParen,
    // Cpp::RPAREN => ")",
    #[strum(serialize = ")")]
    RParen,
    // Cpp::LPAREN2 => "(", // ?
    // Cpp::LBRACK => "[",
    #[strum(serialize = "[")]
    LBracket,
    // Cpp::RBRACK => "]",
    #[strum(serialize = "]")]
    RBracket,
    // Cpp::SEMI => ";",
    #[strum(serialize = ";")]
    SemiColon,
    // Cpp::COMMA => ",",
    #[strum(serialize = ",")]
    Comma,
    // Cpp::DOT => ".",
    #[strum(serialize = ".")]
    Dot,

    // Cpp::PLUS => "+",
    #[strum(serialize = "+")]
    Plus,
    // Cpp::DASH => "-",
    #[strum(serialize = "-")]
    Dash,
    // Cpp::STAR => "*",
    #[strum(serialize = "*")]
    Star,
    // Cpp::SLASH => "/",
    #[strum(serialize = "/")]
    Slash,
    // Cpp::PERCENT => "%",
    #[strum(serialize = "%")]
    Percent,
    // Cpp::BANG => "!",
    #[strum(serialize = "!")]
    Bang,
    // Cpp::GT => ">",
    #[strum(serialize = ">")]
    GT,
    // Cpp::LT => "<",
    #[strum(serialize = "<")]
    LT,
    // Cpp::GTEQ => ">=",
    #[strum(serialize = ">=")]
    GTEq,
    // Cpp::LTEQ => "<=",
    #[strum(serialize = "<=")]
    LTEq,
    // Cpp::EQEQ => "==",
    #[strum(serialize = "==")]
    EqEq,
    // Cpp::BANGEQ => "!=",
    #[strum(serialize = "!=")]
    BangEq,
    // Cpp::AMPAMP => "&&",
    #[strum(serialize = "&&")]
    AmpAmp,
    // Cpp::PIPEPIPE => "||",
    #[strum(serialize = "||")]
    PipePipe,

    // Cpp::TILDE => "~",
    #[strum(serialize = "~")]
    Tilde,
    // Cpp::AMP => "&",
    #[strum(serialize = "&")]
    Amp,
    // Cpp::PIPE => "|",
    #[strum(serialize = "|")]
    Pipe,
    // Cpp::CARET => "^",
    #[strum(serialize = "^")]
    Caret,
    // Cpp::LTLT => "<<",
    #[strum(serialize = "<<")]
    LtLt,
    // Cpp::GTGT => ">>",
    #[strum(serialize = ">>")]
    GtGt,
    // Cpp::QMARK => "?",
    #[strum(serialize = "?")]
    QMark,
    // Cpp::COLON => ":",
    #[strum(serialize = ":")]
    Colon,
    // Cpp::EQ => "=",
    #[strum(serialize = "=")]
    Eq,
    // Cpp::PLUSEQ => "+=",
    #[strum(serialize = "+=")]
    PlusEq,
    // Cpp::DASHEQ => "-=",
    #[strum(serialize = "-=")]
    DashEq,
    // Cpp::STAREQ => "*=",
    #[strum(serialize = "*=")]
    StarEq,
    // Cpp::SLASHEQ => "/=",
    #[strum(serialize = "/=")]
    SlashEq,
    // Cpp::AMPEQ => "&=",
    #[strum(serialize = "&=")]
    AmpEq,
    // Cpp::PIPEEQ => "|=",
    #[strum(serialize = "|=")]
    PipeEq,
    // Cpp::CARETEQ => "^=",
    #[strum(serialize = "^=")]
    CaretEq,
    // Cpp::PERCENTEQ => "%=",
    #[strum(serialize = "%=")]
    PercentEq,
    // Cpp::LTLTEQ => "<<=",
    #[strum(serialize = "<<=")]
    LtLtEq,
    // Cpp::GTGTEQ => ">>=",
    #[strum(serialize = ">>=")]
    GtGtEq,
    // Java::DASHGT => "->",
    #[strum(serialize = "->")]
    DashGt,
    // Cpp::PLUSPLUS => "++",
    #[strum(serialize = "++")]
    PlusPlus,
    // Cpp::DASHDASH => "--",
    #[strum(serialize = "--")]
    DashDash,
    // Java::AT => "@",
    #[strum(serialize = "@")]
    At,
    // Cpp::COLONCOLON => "::",
    #[strum(serialize = "::")]
    ColonColon,

    // Cpp::DOTDOTDOT => "...",
    #[strum(serialize = "...")]
    DotDotDot,
    // Cpp::DASHGTSTAR => "->*",
    #[strum(serialize = "->*")]
    DashGtStar,

    // Cpp::HASHif => "#if",
    #[strum(serialize = "#if")]
    HashIf,
    // Cpp::HASHinclude => "#include",
    #[strum(serialize = "#include")]
    HashInclude,
    // Cpp::HASHdefine => "#define",
    #[strum(serialize = "#define")]
    HashDefine,
    // Cpp::HASHendif => "#endif",
    #[strum(serialize = "#endif")]
    HashEndif,
    // Cpp::HASHifdef => "#ifdef",
    #[strum(serialize = "#ifdef")]
    HashIfdef,
    // Cpp::HASHifndef => "#ifndef",
    #[strum(serialize = "#ifndef")]
    HashIfndef,
    // Cpp::HASHelse => "#else",
    #[strum(serialize = "#else")]
    HashElse,
    // Cpp::HASHelif => "#elif",
    #[strum(serialize = "#elif")]
    HashElif,
    #[strum(serialize = "#elifdef")]
    HashElifdef,
    #[strum(serialize = "#elifndef")]
    HashElifndef,
    // Cpp::LSQUOTE => "L'",
    // Cpp::USQUOTE => "u'",
    // Cpp::USQUOTE2 => "U'",
    // Cpp::U8SQUOTE => "u8'",
    // Cpp::SQUOTE => "'",
    // Cpp::CharLiteralToken1 => "char_literal_token1",
    // Cpp::LDQUOTE => "L\"",
    // Cpp::UDQUOTE => "u\"",
    // Cpp::UDQUOTE2 => "U\"",
    // Cpp::U8DQUOTE => "u8\"",
    // Cpp::LBRACKLBRACK => "[[",
    // Cpp::RBRACKRBRACK => "]]",
    // Cpp::LPARENRPAREN => "()",
    // Cpp::LBRACKRBRACK => "[]",
    // Cpp::DQUOTE => "\"",
    // Cpp::DQUOTEDQUOTE => "\"\"",
    // Cpp::LF => "\n",

    // Cpp::Declspec => "__declspec",
    // Cpp::Based => "__based",
    // Cpp::Cdecl => "__cdecl",
    // Cpp::Clrcall => "__clrcall",
    // Cpp::Stdcall => "__stdcall",
    // Cpp::Fastcall => "__fastcall",
    // Cpp::Thiscall => "__thiscall",
    // Cpp::Vectorcall => "__vectorcall",
    // Cpp::Attribute2 => "__attribute__",
    // Cpp::Unaligned => "_unaligned",
    // Cpp::Unaligned2 => "__unaligned",

    // Cpp::MsRestrictModifier => "ms_restrict_modifier",
    // Cpp::MsUnsignedPtrModifier => "ms_unsigned_ptr_modifier",
    // Cpp::MsSignedPtrModifier => "ms_signed_ptr_modifier",
}
#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum AdditionalKeyword {
    #[strum(serialize = "\"")]
    DQuote,
    #[strum(serialize = "'")]
    SQuote,
    #[strum(serialize = "_")]
    Inderscore,
    #[strum(serialize = "#")]
    Sharp,
    #[strum(serialize = "=>")]
    BigArrow,
    #[strum(serialize = "**")]
    StarStar,
    #[strum(serialize = "===")]
    EqEqEq,
    #[strum(serialize = "!==")]
    BangEqEq,
    #[strum(serialize = "??")]
    QMarkQMark,
    #[strum(serialize = "`")]
    BQuote,
    #[strum(serialize = "**=")]
    StarStarEq,
    #[strum(serialize = "&&=")]
    AmpAmpEq,
    #[strum(serialize = "||=")]
    PipePipeEq,
    #[strum(serialize = "??=")]
    QMarkQMarkEq,
    #[strum(serialize = "?.")]
    QMarkDot,
    #[strum(serialize = "<template>")]
    TemplateOpen,
    #[strum(serialize = "</template>")]
    TemplateClose,
    #[strum(serialize = "")]
    QMark,
    #[strum(serialize = "-?:")]
    MinusQMarkColon,
    #[strum(serialize = "+?:")]
    PlusQMarkColon,
    #[strum(serialize = "?:")]
    QMarkColon,
    #[strum(serialize = "${")]
    DollarLBrace,
    #[strum(serialize = "{|")]
    LBracePipe,
    #[strum(serialize = "|}")]
    PipeRBrace,
}
