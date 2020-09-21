#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let { ident: Token, value: Box<Expr> },
    Expression(Box<Expr>),
    Return { value: Box<Expr> },
    Empty,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Const(u64),
    String(String),
    Boolean(bool),
    Ident(String),
    Prefix {
        prefix: Token,
        value: Box<Expr>,
    },
    Infix {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        consequence: Vec<Statement>,
        alternative: Vec<Statement>,
    },
    Function {
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    Call {
        function: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EKeyword {
    Function,
    Let,
    If,
    Else,
    True,
    False,
    Return,
}

impl ToString for EKeyword {
    fn to_string(&self) -> String {
        match self {
            EKeyword::Function => "fn".to_string(),
            EKeyword::Let => "let".to_string(),
            EKeyword::If => "if".to_string(),
            EKeyword::Else => "else".to_string(),
            EKeyword::True => "true".to_string(),
            EKeyword::False => "false".to_string(),
            EKeyword::Return => "return".to_string(),
        }
    }
}

#[derive(PartialOrd, PartialEq)]
pub enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    RightCurlyBrace,
    LeftCurlyBrace,
    RightSquareBrace,
    LeftSquareBrace,
    RightParenthesis,
    LeftParenthesis,
    Comma,
    SemiColon,

    // Operators
    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Bang,
    LessThan,
    GreaterThan,
    Equal,
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,

    EOF,
    Invalid(String),

    Integer(u64),
    Identifier(String),
    Keyword(EKeyword),
}
impl ToString for Token {
    fn to_string(&self) -> String {
        match self.clone() {
            Token::RightCurlyBrace => "}".to_string(),
            Token::LeftCurlyBrace => "{".to_string(),
            Token::RightSquareBrace => "]".to_string(),
            Token::LeftSquareBrace => "[".to_string(),
            Token::RightParenthesis => ")".to_string(),
            Token::LeftParenthesis => "(".to_string(),
            Token::Comma => ",".to_string(),
            Token::SemiColon => ";".to_string(),
            Token::Assign => "=".to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Asterisk => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Bang => "!".to_string(),
            Token::LessThan => "<".to_string(),
            Token::GreaterThan => ">".to_string(),
            Token::Equal => "==".to_string(),
            Token::NotEqual => "!=".to_string(),
            Token::LessThanOrEqual => "<=".to_string(),
            Token::GreaterThanOrEqual => ">=".to_string(),
            Token::Integer(n) => n.to_string(),
            Token::Identifier(id) => id,
            Token::Keyword(kw) => kw.to_string(),
            Token::EOF => "".to_string(),
            Token::Invalid(s) => format!("<invalid=\"{}\"", s),
        }
    }
}
impl Token {
    pub fn from_char(ch: char) -> Option<Token> {
        match ch {
            '{' => Some(Token::LeftCurlyBrace),
            '}' => Some(Token::RightCurlyBrace),
            '[' => Some(Token::LeftSquareBrace),
            ']' => Some(Token::RightSquareBrace),
            '(' => Some(Token::LeftParenthesis),
            ')' => Some(Token::RightParenthesis),
            ',' => Some(Token::Comma),
            '=' => Some(Token::Assign),
            '+' => Some(Token::Plus),
            ';' => Some(Token::SemiColon),
            '*' => Some(Token::Asterisk),
            '!' => Some(Token::Bang),
            '/' => Some(Token::Slash),
            '<' => Some(Token::LessThan),
            '>' => Some(Token::GreaterThan),
            _ => None,
        }
    }

    pub fn precedence(&self) -> Precedence {
        match self {
            Token::Plus => Precedence::Sum,
            Token::Minus => Precedence::Sum,
            Token::Slash => Precedence::Product,
            Token::Equal => Precedence::Equals,
            Token::NotEqual => Precedence::Equals,
            Token::Asterisk => Precedence::Product,
            Token::LessThan => Precedence::LessGreater,
            Token::GreaterThan => Precedence::LessGreater,
            Token::LessThanOrEqual => Precedence::LessGreater,
            Token::GreaterThanOrEqual => Precedence::LessGreater,
            _ => Precedence::Lowest,
        }
    }

    pub fn literal(&self) -> String {
        self.to_string()
    }
}
