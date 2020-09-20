use super::{
    ast::{EKeyword, Expr, Precedence, Statement, Token},
    lexer::Lexer,
};

pub(crate) fn parse_program(src: &str) -> Vec<Statement> {
    let mut p = Parser::new(src);
    let statements = p.parse();
    for error in p.errors {
        eprintln!("error: {}", error);
    }

    statements
}

struct Parser<'s> {
    lexer: Lexer<'s>,
    errors: Vec<String>,
}
impl<'s> Parser<'s> {
    fn new(source: &'s str) -> Parser<'s> {
        Parser { lexer: Lexer::new(source), errors: Vec::new() }
    }

    fn parse(&mut self) -> Vec<Statement> {
        dbg!("parse");
        let mut stmts = Vec::new();
        loop {
            let next = self.lexer.next_token();
            dbg!(&next);
            let statement = match next {
                Token::Keyword(EKeyword::Let) => self.parse_let_statement(),
                Token::Keyword(EKeyword::Return) => self.parse_return_statement(),
                Token::Keyword(EKeyword::Else) | Token::RightCurlyBrace | Token::EOF => break,
                _ => {
                    self.lexer.rewind(next.literal().len());
                    if let Some(expr) = self.parse_expr(Precedence::Lowest) {
                        return Statement::Expression(Box::new(expr));
                    } else {
                        break;
                    }
                }
            };

            dbg!(&statement);

            stmts.push(statement);

            if self.lexer.is_last() {
                break;
            }
        }

        stmts
    }

    fn check_semi_colon(&mut self) {
        let semi_colon = self.lexer.next_token();
        dbg!(&semi_colon);
        if semi_colon != Token::SemiColon {
            self.add_error(format!("missing semicolon after expression"));
        }
    }

    fn add_error(&mut self, err: String) {
        self.errors.push(err);
    }

    fn parse_let_statement(&mut self) -> Statement {
        let ident = self.lexer.next_token();
        let assign = self.lexer.next_token();
        dbg!(&ident);
        dbg!(&assign);
        match ident {
            Token::Identifier(name) => match assign {
                Token::Assign => {
                    if let Some(expr) = self.parse_expr(Precedence::Lowest) {
                        self.check_semi_colon();
                        return Statement::Let { ident: Token::Identifier(name), value: Box::new(expr) };
                    }
                }
                t => self.add_error(format!("invalid token {:?} as assignment operator in let statement", t)),
            },
            t => self.add_error(format!("invalid token {:?} as identifier name in let statement", t)),
        }
        Statement::Empty
    }

    fn parse_return_statement(&mut self) -> Statement {
        if let Some(expr) = self.parse_expr(Precedence::Lowest) {
            self.check_semi_colon();
            return Statement::Return { value: Box::new(expr) };
        }

        self.add_error("missing expression after return keyword".to_string());
        Statement::Empty
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
        dbg!("gothere");
        match self.lexer.next_token() {
            Token::LeftParenthesis => {
                let condition = self.parse_expr(Precedence::Lowest);
                match self.lexer.next_token() {
                    Token::RightParenthesis => {
                        let consequence = self.parse_curly_block();
                        dbg!("here?");
                        match self.lexer.next_token() {
                            Token::RightCurlyBrace => {
                                dbg!(self.lexer.peek_token());
                                let alternative = if self.lexer.next_token() == Token::Keyword(EKeyword::Else) {
                                    self.parse_curly_block()
                                } else {
                                    dbg!(self.lexer.peek_token());
                                    Vec::new()
                                };

                                Some(Expr::If { condition: Box::new(condition?), consequence, alternative })
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn parse_curly_block(&mut self) -> Vec<Statement> {
        let block = self.lexer.next_token();
        dbg!(&block);
        match block {
            Token::LeftCurlyBrace => {
                let out = self.parse();
                dbg!(self.lexer.peek_token());
                if self.lexer.next_token() != Token::RightCurlyBrace {
                    dbg!("Error!");
                }
                out
            }
            t => {
                self.add_error(format!("invalid token, expected '{{' found {:?}", t));
                Vec::new()
            }
        }
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<Expr> {
        let xt = self.lexer.next_token();
        dbg!(&xt);
        let expr = match xt {
            Token::Integer(n) => Some(Expr::Const(n)),
            Token::Keyword(EKeyword::True) => Some(Expr::Boolean(true)),
            Token::Keyword(EKeyword::False) => Some(Expr::Boolean(false)),
            Token::Identifier(s) => Some(Expr::Ident(s)),
            Token::Bang => {
                Some(Expr::Prefix { prefix: Token::Bang, value: Box::new(self.parse_expr(Precedence::Prefix)?) })
            }
            Token::Minus => {
                Some(Expr::Prefix { prefix: Token::Minus, value: Box::new(self.parse_expr(Precedence::Prefix)?) })
            }
            Token::LeftCurlyBrace => {
                self.lexer.next_token();
                self.parse_expr(Precedence::Lowest)
            }
            Token::Keyword(EKeyword::If) => self.parse_if_expr(),
            _ => Some(Expr::Unknown),
        };
        dbg!(&expr);

        expr
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
            Statement::Let { ident: Token::Identifier("var".to_string()), value: Box::new(Expr::Const(5)) },
            Statement::Let { ident: Token::Identifier("is_true".to_string()), value: Box::new(Expr::Boolean(true)) },
            Statement::Let { ident: Token::Identifier("is_false".to_string()), value: Box::new(Expr::Boolean(false)) },
        ];
        let mut parser = Parser::new(code);

        assert_eq!(parser.parse(), expected);
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
            consequence: vec![Statement::Return { value: Box::new(Expr::Const(15)) }],
            alternative: vec![Statement::Return { value: Box::new(Expr::Const(30)) }],
        }))];
        let mut parser = Parser::new(code);

        assert_eq!(parser.parse(), expected);
    }
}
