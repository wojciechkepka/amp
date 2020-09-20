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
        let mut stmts = Vec::new();
        loop {
            let next = self.lexer.next_token();
            dbg!(&next);
            let statement = match next {
                Token::EOF => break,
                Token::Keyword(EKeyword::Let) => self.parse_let_statement(),
                Token::Keyword(EKeyword::Return) => self.parse_return_statement(),
                Token::RightCurlyBrace => break,
                _ => Statement::Expression(Box::new(self.parse_expr(Precedence::Lowest).unwrap())),
            };

            dbg!(&statement);

            stmts.push(statement);
        }

        stmts
    }

    fn check_semi_colon(&mut self) {
        let next = self.lexer.next_token();
        if next != Token::SemiColon {
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
        match self.lexer.next_token() {
            Token::LeftParenthesis => {
                let condition = self.parse_expr(Precedence::Lowest);
                let consequence = self.parse_curly_block();
                let alternative = if self.lexer.next_token() == Token::Keyword(EKeyword::Else) {
                    self.parse_curly_block()
                } else {
                    Vec::new()
                };

                Some(Expr::If { condition: Box::new(condition?), consequence, alternative })
            }
            _ => None,
        }
    }

    fn parse_curly_block(&mut self) -> Vec<Statement> {
        match self.lexer.next_token() {
            Token::LeftCurlyBrace => self.parse(),
            t => {
                self.add_error(format!("invalid token, expected '{{' found {:?}", t));
                Vec::new()
            }
        }
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<Expr> {
        let expr = match self.lexer.next_token() {
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
}
