use super::{
    ast::{EKeyword, Expr, Precedence, Statement, Token},
    lexer::Lexer,
    AmpError,
};

pub(crate) fn parse_program(src: &str) -> Result<Vec<Statement>, AmpError> {
    let mut p = Parser::new(src);
    let statements = p.parse();
    statements
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
            current: Token::EOF,
            peek: Token::EOF,
        }
    }
    fn expect(&mut self, first: &Token, second: &Token) -> Result<(), AmpError> {
        if first != second {
            return Err(AmpError::InvalidToken(first.clone(), second.clone()));
        }

        Ok(())
    }

    fn expect_current(&mut self, token: &Token) -> Result<(), AmpError> {
        self.expect(&self.current.clone(), &token)
    }

    fn expect_peek(&mut self, token: &Token) -> Result<(), AmpError> {
        self.expect(&self.peek.clone(), &token)
    }

    fn next(&mut self) {
        dbg!(&self.current);
        self.current = self.lexer.next_token();
        dbg!(&self.current);
        self.peek = self.lexer.peek_token();
        dbg!(&self.peek);
    }

    fn parse(&mut self) -> Result<Vec<Statement>, AmpError> {
        dbg!("parse");
        let mut stmts = Vec::new();
        dbg!(format!("{:?}", self.current));
        loop {
            self.next();
            let statement = match self.current.clone() {
                Token::Keyword(EKeyword::Let) => self.parse_let_statement()?,
                Token::Keyword(EKeyword::Return) => self.parse_return_statement()?,
                Token::EOF => break,
                rewind @ Token::Keyword(EKeyword::Else) | rewind @ Token::RightCurlyBrace => {
                    self.lexer.rewind(self.current.literal().len());
                    self.current = rewind.clone();
                    self.peek = self.lexer.peek_token();
                    dbg!(&self.current);
                    break;
                }
                _ => {
                    self.lexer.rewind(self.current.literal().len());
                    Statement::Expression(Box::new(self.parse_expr(Precedence::Lowest)?))
                }
            };

            dbg!(&statement);

            stmts.push(statement);

            if self.lexer.is_last() {
                break;
            }
        }

        Ok(stmts)
    }

    fn parse_let_statement(&mut self) -> Result<Statement, AmpError> {
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

        let expr = self.parse_expr(Precedence::Lowest)?;
        self.expect_peek(&Token::SemiColon)?;
        self.next();

        Ok(Statement::Let {
            ident: Token::Identifier(ident),
            value: Box::new(expr),
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, AmpError> {
        let expr = self.parse_expr(Precedence::Lowest)?;
        self.expect_peek(&Token::SemiColon)?;
        self.next();
        Ok(Statement::Return {
            value: Box::new(expr),
        })
    }

    fn parse_if_expr(&mut self) -> Result<Expr, AmpError> {
        self.expect_peek(&Token::LeftParenthesis)?;
        self.next();
        let condition = self.parse_expr(Precedence::Lowest);
        self.expect_peek(&Token::RightParenthesis)?;
        self.next();
        let consequence = self.parse_curly_block()?;
        dbg!("here");
        let alternative = if self.current == Token::Keyword(EKeyword::Else) {
            self.next();
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
        self.expect_peek(&Token::LeftCurlyBrace)?;
        self.next();
        dbg!(format!("here {:?}", &self.current));
        let out = self.parse();
        self.expect_peek(&Token::RightCurlyBrace)?;
        self.next();
        Ok(out?)
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Result<Expr, AmpError> {
        self.next();
        let expr = match self.current.clone() {
            Token::Integer(n) => Expr::Const(n),
            Token::Keyword(EKeyword::True) => Expr::Boolean(true),
            Token::Keyword(EKeyword::False) => Expr::Boolean(false),
            Token::Identifier(s) => Expr::Ident(s),
            Token::Bang => Expr::Prefix {
                prefix: Token::Bang,
                value: Box::new(self.parse_expr(Precedence::Prefix)?),
            },
            Token::Minus => Expr::Prefix {
                prefix: Token::Minus,
                value: Box::new(self.parse_expr(Precedence::Prefix)?),
            },
            Token::LeftCurlyBrace => {
                self.next();
                self.parse_expr(Precedence::Lowest)?
            }
            Token::Keyword(EKeyword::If) => self.parse_if_expr()?,
            _ => Expr::Unknown,
        };
        dbg!(&expr);

        Ok(expr)
    }
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
}
