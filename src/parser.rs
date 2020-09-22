use super::{
    ast::{EKeyword, Expr, Precedence, Statement, Token},
    lexer::Lexer,
    AmpError,
};
use log::debug;

pub(crate) fn parse_program(src: &str) -> Result<Vec<Statement>, AmpError> {
    let mut p = Parser::new(src);
    let statements = p.parse();
    statements
}

macro_rules! function_name {
    () => {{
        // Okay, this is ugly, I get it. However, this is the best we can get on a stable rust.
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        // `3` is the length of the `::f`.
        &name[..name.len() - 3]
    }};
}

macro_rules! ldebug {
    ($msg:expr) => {
        debug!("[{}:{}] {}", function_name!(), line!(), $msg);
    };
}

struct Parser<'s> {
    lexer: Lexer<'s>,
    current: Token,
    peek: Token,
}
impl<'s> Parser<'s> {
    fn new(source: &'s str) -> Parser<'s> {
        Parser {
            lexer: Lexer::new(source),
            current: Token::Null,
            peek: Token::Null,
        }
    }
    fn expect(&mut self, first: &Token, second: &Token) -> Result<(), AmpError> {
        if first != second {
            return Err(AmpError::InvalidToken(first.clone(), second.clone()));
        }

        Ok(())
    }

    fn dbg(&self) -> String {
        format!("current - '{:?}', peek - '{:?}'", self.current, self.peek)
    }

    #[allow(dead_code)]
    fn expect_current(&mut self, token: &Token) -> Result<(), AmpError> {
        self.expect(&self.current.clone(), &token)
    }

    fn expect_peek(&mut self, token: &Token) -> Result<(), AmpError> {
        ldebug!(format!("got '{:?}', expecting peek '{:?}'", &self.peek, &token));
        self.expect(&self.peek.clone(), &token)
    }

    fn next(&mut self) {
        std::mem::swap(&mut self.peek, &mut self.current);
        self.peek = self.lexer.next_token();
        ldebug!(format!("after `{}`", self.dbg()));
    }

    fn parse(&mut self) -> Result<Vec<Statement>, AmpError> {
        let mut stmts = Vec::new();
        loop {
            ldebug!(format!("begin `{}`", self.dbg()));
            let statement = match self.current.clone() {
                Token::Keyword(EKeyword::Let) => self.parse_let_statement()?,
                Token::Keyword(EKeyword::Return) => self.parse_return_statement()?,
                Token::EOF => {
                    break;
                }
                Token::Keyword(EKeyword::Else) | Token::RightCurlyBrace => {
                    break;
                }
                Token::SemiColon | Token::Null => {
                    self.next();
                    continue;
                }
                _ => Statement::Expression(Box::new(self.parse_expr(Precedence::Lowest)?)),
            };

            stmts.push(statement);

            if self.lexer.is_last() {
                break;
            }

            self.next();
        }

        Ok(stmts)
    }

    #[allow(unused_assignments)]
    fn parse_let_statement(&mut self) -> Result<Statement, AmpError> {
        ldebug!(format!("begin `{}`", self.dbg()));
        self.next();
        let mut ident = String::new();
        if let Token::Identifier(name) = self.current.clone() {
            ident = name;
        } else {
            return Err(AmpError::InvalidToken(
                self.current.clone(),
                Token::Identifier("".to_string()),
            ));
        }
        self.expect_peek(&Token::Assign)?;
        self.next();
        self.next();

        let expr = self.parse_expr(Precedence::Lowest)?;
        self.expect_peek(&Token::SemiColon)?;
        self.next();
        ldebug!(format!("end `{}`", self.dbg()));

        Ok(Statement::Let {
            ident: Token::Identifier(ident),
            value: Box::new(expr),
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, AmpError> {
        ldebug!(format!("begin `{}`", self.dbg()));
        self.next();
        let expr = self.parse_expr(Precedence::Lowest)?;
        self.expect_peek(&Token::SemiColon)?;
        self.next();
        ldebug!(format!("end `{}`", self.dbg()));
        Ok(Statement::Return { value: Box::new(expr) })
    }

    fn parse_if_expr(&mut self) -> Result<Expr, AmpError> {
        ldebug!(format!("begin `{}`", self.dbg()));
        self.expect_peek(&Token::LeftParenthesis)?;
        self.next();
        self.next();
        ldebug!(format!("parsing condition `{}`", self.dbg()));
        let condition = self.parse_expr(Precedence::Lowest);
        self.expect_peek(&Token::RightParenthesis)?;
        self.next();
        ldebug!(format!("parsing consequence `{}`", self.dbg()));
        let consequence = self.parse_curly_block()?;
        ldebug!(format!("parsing alternative `{}`", self.dbg()));
        let alternative = if self.current == Token::Keyword(EKeyword::Else) {
            self.parse_curly_block()?
        } else {
            Vec::new()
        };

        Ok(Expr::If {
            condition: Box::new(condition?),
            consequence,
            alternative,
        })
    }

    fn parse_curly_block(&mut self) -> Result<Vec<Statement>, AmpError> {
        ldebug!(format!("begin `{}`", self.dbg()));
        self.expect_peek(&Token::LeftCurlyBrace)?;
        self.next();
        self.next();
        let out = self.parse();
        ldebug!(format!("after parse `{}`", self.dbg()));
        self.next();
        ldebug!(format!("[{}] out - '{:?}'", function_name!(), &out));
        Ok(out?)
    }

    fn parse_prefix_expr(&mut self, prefix: Token) -> Result<Expr, AmpError> {
        ldebug!(format!("begin `{}`", self.dbg()));
        self.next();
        let value = self.parse_expr(Precedence::Prefix)?;
        ldebug!(format!("[{}] out - '{:?}'", function_name!(), &value));
        Ok(Expr::Prefix {
            prefix,
            value: Box::new(value),
        })
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Result<Expr, AmpError> {
        ldebug!(format!("begin `{}`", self.dbg()));
        let expr = match self.current.clone() {
            Token::Integer(n) => Expr::Const(n),
            Token::Keyword(EKeyword::True) => Expr::Boolean(true),
            Token::Keyword(EKeyword::False) => Expr::Boolean(false),
            Token::Identifier(s) => Expr::Ident(s),
            t @ Token::Bang | t @ Token::Minus => self.parse_prefix_expr(t)?,
            Token::LeftCurlyBrace => {
                self.next();
                self.parse_expr(Precedence::Lowest)?
            }
            Token::Keyword(EKeyword::If) => self.parse_if_expr()?,
            t => panic!("Unknown token {:?}", t),
        };
        ldebug!(format!("[{}] out - '{:?}'", function_name!(), &expr));
        ldebug!(format!("end `{}`", self.dbg()));
        Ok(expr)
    }
}
pub fn parses_if_else() {
    let code = "if (x) {
    return 15;
}
    else {
    return 30;
    }";
    let expected = vec![Statement::Expression(Box::new(Expr::If {
        condition: Box::new(Expr::Ident("x".to_string())),
        consequence: vec![Statement::Return {
            value: Box::new(Expr::Const(15)),
        }],
        alternative: vec![Statement::Return {
            value: Box::new(Expr::Const(30)),
        }],
    }))];
    let mut parser = Parser::new(code);

    assert_eq!(parser.parse(), Ok(expected));
}
pub fn parses_let_statement() {
    let code = "let var = 5;
let is_true = true;
let is_false = false;";
    let expected = vec![
        Statement::Let {
            ident: Token::Identifier("var".to_string()),
            value: Box::new(Expr::Const(5)),
        },
        Statement::Let {
            ident: Token::Identifier("is_true".to_string()),
            value: Box::new(Expr::Boolean(true)),
        },
        Statement::Let {
            ident: Token::Identifier("is_false".to_string()),
            value: Box::new(Expr::Boolean(false)),
        },
    ];
    let mut parser = Parser::new(code);

    assert_eq!(parser.parse(), Ok(expected));
}
pub fn parses_prefix_expression() {
    let code = "
            -1000;
            !true;
            ";
    let expected = vec![
        Statement::Expression(Box::new(Expr::Prefix {
            prefix: Token::Minus,
            value: Box::new(Expr::Const(1000)),
        })),
        Statement::Expression(Box::new(Expr::Prefix {
            prefix: Token::Bang,
            value: Box::new(Expr::Boolean(true)),
        })),
    ];
    let mut parser = Parser::new(code);

    assert_eq!(parser.parse(), Ok(expected));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_let_statement() {
        let code = "let var = 5;
let is_true = true;
let is_false = false;";
        let expected = vec![
            Statement::Let {
                ident: Token::Identifier("var".to_string()),
                value: Box::new(Expr::Const(5)),
            },
            Statement::Let {
                ident: Token::Identifier("is_true".to_string()),
                value: Box::new(Expr::Boolean(true)),
            },
            Statement::Let {
                ident: Token::Identifier("is_false".to_string()),
                value: Box::new(Expr::Boolean(false)),
            },
        ];
        let mut parser = Parser::new(code);

        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn parses_if_else() {
        let code = "if (x) {
    return 15;
}    else {
    return 30;
}
";
        let expected = vec![Statement::Expression(Box::new(Expr::If {
            condition: Box::new(Expr::Ident("x".to_string())),
            consequence: vec![Statement::Return {
                value: Box::new(Expr::Const(15)),
            }],
            alternative: vec![Statement::Return {
                value: Box::new(Expr::Const(30)),
            }],
        }))];
        let mut parser = Parser::new(code);

        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn parses_if_else_multiple_statements() {
        let code = "
    if (!x) {
    let z = y;
    let y = x;
    let z = -1000;
    let y = !x;
    } else {
    let z = -1000;
    let y = !x;
    let z = -1000;
    let y = !x;
    }
    ";
        let expected = vec![Statement::Expression(Box::new(Expr::If {
            condition: Box::new(Expr::Prefix {
                prefix: Token::Bang,
                value: Box::new(Expr::Ident("x".to_string())),
            }),
            consequence: vec![
                Statement::Let {
                    ident: Token::Identifier("z".to_string()),
                    value: Box::new(Expr::Ident("y".to_string())),
                },
                Statement::Let {
                    ident: Token::Identifier("y".to_string()),
                    value: Box::new(Expr::Ident("x".to_string())),
                },
                Statement::Let {
                    ident: Token::Identifier("z".to_string()),
                    value: Box::new(Expr::Prefix {
                        prefix: Token::Minus,
                        value: Box::new(Expr::Const(1000)),
                    }),
                },
                Statement::Let {
                    ident: Token::Identifier("y".to_string()),
                    value: Box::new(Expr::Prefix {
                        prefix: Token::Bang,
                        value: Box::new(Expr::Ident("x".to_string())),
                    }),
                },
            ],
            alternative: vec![
                Statement::Let {
                    ident: Token::Identifier("z".to_string()),
                    value: Box::new(Expr::Prefix {
                        prefix: Token::Minus,
                        value: Box::new(Expr::Const(1000)),
                    }),
                },
                Statement::Let {
                    ident: Token::Identifier("y".to_string()),
                    value: Box::new(Expr::Prefix {
                        prefix: Token::Bang,
                        value: Box::new(Expr::Ident("x".to_string())),
                    }),
                },
                Statement::Let {
                    ident: Token::Identifier("z".to_string()),
                    value: Box::new(Expr::Prefix {
                        prefix: Token::Minus,
                        value: Box::new(Expr::Const(1000)),
                    }),
                },
                Statement::Let {
                    ident: Token::Identifier("y".to_string()),
                    value: Box::new(Expr::Prefix {
                        prefix: Token::Bang,
                        value: Box::new(Expr::Ident("x".to_string())),
                    }),
                },
            ],
        }))];
        let mut parser = Parser::new(code);

        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn parses_prefix_expression() {
        let code = "
            -1000;
            !true;
            ";
        let expected = vec![
            Statement::Expression(Box::new(Expr::Prefix {
                prefix: Token::Minus,
                value: Box::new(Expr::Const(1000)),
            })),
            Statement::Expression(Box::new(Expr::Prefix {
                prefix: Token::Bang,
                value: Box::new(Expr::Boolean(true)),
            })),
        ];
        let mut parser = Parser::new(code);

        assert_eq!(parser.parse(), Ok(expected));
    }
}
