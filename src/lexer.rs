use super::{
    ast::{EKeyword, Token},
    reader::Reader,
};

pub(crate) struct Lexer<'r> {
    reader: Reader<'r>,
}
impl<'r> Lexer<'r> {
    pub(crate) fn new(source: &'r str) -> Lexer<'r> {
        Lexer { reader: Reader::new(source) }
    }

    pub(crate) fn peek_token(&mut self) -> Token {
        self.reader.save_hook();
        let token = self.next_token();
        self.reader.rewind_last_hook();
        token
    }

    pub(crate) fn next_token(&mut self) -> Token {
        let token = match self.reader.current() {
            ch @ '{'
            | ch @ '}'
            | ch @ '['
            | ch @ ']'
            | ch @ '('
            | ch @ ')'
            | ch @ ','
            | ch @ '+'
            | ch @ ';'
            | ch @ '*'
            | ch @ '/' => {
                self.reader.skip(1);
                Token::from_char(ch).unwrap()
            }
            '!' => self.parse_double_or_single('=', Token::NotEqual, Token::Bang),
            '<' => self.parse_double_or_single('=', Token::LessThanOrEqual, Token::LessThan),
            '>' => self.parse_double_or_single('=', Token::GreaterThanOrEqual, Token::GreaterThan),
            '=' => self.parse_double_or_single('=', Token::Equal, Token::Assign),
            ch if ch.is_ascii_whitespace() => {
                self.reader.skip_whitespace();
                if self.reader.is_last() {
                    return Token::EOF;
                }
                self.next_token()
            }
            ch if ch.is_ascii_digit() => self.parse_number(),
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.parse_ident_or_keyword(),
            ch => Token::Invalid(ch.to_string()),
        };
        token
    }

    fn parse_double_or_single(&mut self, second_char: char, double: Token, single: Token) -> Token {
        if let Some(ch) = self.reader.peek() {
            if ch == second_char {
                self.reader.skip(2);
                return double;
            }
        }
        self.reader.skip(1);
        single
    }

    fn parse_number(&mut self) -> Token {
        let mut num = format!("{}", self.reader.current());
        while let Some(ch) = self.reader.next() {
            if !ch.is_ascii_digit() {
                break;
            }
            num.push(ch);
        }

        Token::Integer(num.parse().unwrap())
    }

    fn parse_ident_or_keyword(&mut self) -> Token {
        let mut ident = format!("{}", self.reader.current());
        while let Some(ch) = self.reader.next() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                ident.push(ch);
            } else {
                break;
            }
        }

        match ident.as_ref() {
            "let" => Token::Keyword(EKeyword::Let),
            "fn" => Token::Keyword(EKeyword::Function),
            "if" => Token::Keyword(EKeyword::If),
            "else" => Token::Keyword(EKeyword::Else),
            "true" => Token::Keyword(EKeyword::True),
            "false" => Token::Keyword(EKeyword::False),
            "return" => Token::Keyword(EKeyword::Return),
            _ => Token::Identifier(ident),
        }
    }

    pub(crate) fn is_last(&self) -> bool {
        self.reader.is_last()
    }

    pub(crate) fn rewind(&mut self, n: usize) {
        self.reader.rewind(n as isize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_tokens() {
        let input = "{}[]()+=,";
        let expected = vec![
            Token::LeftCurlyBrace,
            Token::RightCurlyBrace,
            Token::LeftSquareBrace,
            Token::RightSquareBrace,
            Token::LeftParenthesis,
            Token::RightParenthesis,
            Token::Plus,
            Token::Assign,
            Token::Comma,
        ];

        let mut l = Lexer::new(input);

        for token in expected {
            assert_eq!(l.next_token(), token);
        }
    }

    #[test]
    fn parses_simple_syntax() {
        let input = "let five = 5;
let add = fn(x, y) {
    x + y;
};

let result = add(5, 10);
";
        let expected = vec![
            Token::Keyword(EKeyword::Let),
            Token::Identifier("five".to_string()),
            Token::Assign,
            Token::Integer(5),
            Token::SemiColon,
            Token::Keyword(EKeyword::Let),
            Token::Identifier("add".to_string()),
            Token::Assign,
            Token::Keyword(EKeyword::Function),
            Token::LeftParenthesis,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::RightParenthesis,
            Token::LeftCurlyBrace,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::SemiColon,
            Token::RightCurlyBrace,
            Token::SemiColon,
            Token::Keyword(EKeyword::Let),
            Token::Identifier("result".to_string()),
            Token::Assign,
            Token::Identifier("add".to_string()),
            Token::LeftParenthesis,
            Token::Integer(5),
            Token::Comma,
            Token::Integer(10),
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let mut l = Lexer::new(input);

        for token in expected {
            assert_eq!(l.next_token(), token);
        }
    }

    #[test]
    fn parses_if_else_return_true_false() {
        let input = "
let is_smaller = fn(x, y) {
    if x < y {
        return true;
    } else {
        return false;
    }
};

!is_smaller(3*4, 25/5);


";
        let expected = vec![
            Token::Keyword(EKeyword::Let),
            Token::Identifier("is_smaller".to_string()),
            Token::Assign,
            Token::Keyword(EKeyword::Function),
            Token::LeftParenthesis,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::RightParenthesis,
            Token::LeftCurlyBrace,
            Token::Keyword(EKeyword::If),
            Token::Identifier("x".to_string()),
            Token::LessThan,
            Token::Identifier("y".to_string()),
            Token::LeftCurlyBrace,
            Token::Keyword(EKeyword::Return),
            Token::Keyword(EKeyword::True),
            Token::SemiColon,
            Token::RightCurlyBrace,
            Token::Keyword(EKeyword::Else),
            Token::LeftCurlyBrace,
            Token::Keyword(EKeyword::Return),
            Token::Keyword(EKeyword::False),
            Token::SemiColon,
            Token::RightCurlyBrace,
            Token::RightCurlyBrace,
            Token::SemiColon,
            Token::Bang,
            Token::Identifier("is_smaller".to_string()),
            Token::LeftParenthesis,
            Token::Integer(3),
            Token::Asterisk,
            Token::Integer(4),
            Token::Comma,
            Token::Integer(25),
            Token::Slash,
            Token::Integer(5),
            Token::RightParenthesis,
            Token::SemiColon,
        ];

        let mut l = Lexer::new(input);

        for token in expected {
            dbg!(token.clone());
            assert_eq!(l.next_token(), token);
        }
    }

    #[test]
    fn parses_double_char_operators() {
        let input = "
            x == 5;
            x != 6;
            x <= 3;
            x >= 4;
        ";

        let expected = vec![
            Token::Identifier("x".to_string()),
            Token::Equal,
            Token::Integer(5),
            Token::SemiColon,
            Token::Identifier("x".to_string()),
            Token::NotEqual,
            Token::Integer(6),
            Token::SemiColon,
            Token::Identifier("x".to_string()),
            Token::LessThanOrEqual,
            Token::Integer(3),
            Token::SemiColon,
            Token::Identifier("x".to_string()),
            Token::GreaterThanOrEqual,
            Token::Integer(4),
            Token::SemiColon,
        ];

        let mut l = Lexer::new(input);

        for token in expected {
            dbg!(token.clone());
            assert_eq!(l.next_token(), token);
        }
    }
}
