#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Eof,
    Def,                // def
    Extern,             // extern
    Identifier(String), // \p{Aphabetic}\w*
    Number(f64),        // \d+\.?\d*
    Char(char),         //
}

pub struct Lexer<I>
where
    I: Iterator<Item = char>,
{
    input: I,
    last_char: Option<char>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(mut input: I) -> Lexer<I> {
        let last_char = input.next();
        Lexer { input, last_char }
    }

    fn step(&mut self) -> Option<char> {
        self.last_char = self.input.next();
        self.last_char
    }

    // lex and return next token
    pub fn next_token(&mut self) -> Token {
        // skip white space
        while matches!(self.last_char, Some(c) if c.is_ascii_whitespace()) {
            self.step();
        }

        // unpack last char or return EOF
        let last_char = if let Some(c) = self.last_char {
            c
        } else {
            return Token::Eof;
        };

        // Identifier: [a-zA-Z][a-zA-Z0-9]*
        if last_char.is_ascii_alphabetic() {
            let mut identifier = String::new();
            identifier.push(last_char);

            while let Some(c) = self.step() {
                if c.is_ascii_alphanumeric() {
                    identifier.push(c)
                } else {
                    break;
                }
            }

            match identifier.as_ref() {
                "def" => return Token::Def,
                "extern" => return Token::Extern,
                _ => {}
            }

            return Token::Identifier(identifier);
        }

        // Number: [0-9.]+
        if last_char.is_ascii_digit() || last_char == '.' {
            let mut num = String::new();
            num.push(last_char);

            while let Some(c) = self.step() {
                if c.is_ascii_digit() || c == '.' {
                    num.push(c)
                } else {
                    break;
                }
            }

            let num: f64 = num.parse().unwrap_or_default();
            return Token::Number(num);
        }

        // skip comment
        if last_char == '#' {
            loop {
                match self.step() {
                    Some(c) if c == '\r' || c == '\n' => return self.next_token(),
                    None => return Token::Eof,
                    _ => {}
                }
            }
        }

        // advance last char
        self.step();
        Token::Char(last_char)
    }
}

#[cfg(test)]
mod test {
    use super::{Lexer, Token};

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("a b c".chars());
        assert_eq!(Token::Identifier("a".into()), lexer.next_token());
        assert_eq!(Token::Identifier("b".into()), lexer.next_token());
        assert_eq!(Token::Identifier("c".into()), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());
    }

    #[test]
    fn test_keyword() {
        let mut lexer = Lexer::new("def extern".chars());
        assert_eq!(Token::Def, lexer.next_token());
        assert_eq!(Token::Extern, lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());
    }

    #[test]
    fn test_number() {
        let mut lexer = Lexer::new("12.34".chars());
        assert_eq!(Token::Number(12.34f64), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());

        let mut lexer = Lexer::new(" 1.0 2.0 3.1".chars());
        assert_eq!(Token::Number(1.0f64), lexer.next_token());
        assert_eq!(Token::Number(2.0f64), lexer.next_token());
        assert_eq!(Token::Number(3.1f64), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());

        let mut lexer = Lexer::new("12.34.1".chars());
        assert_eq!(Token::Number(0f64), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("# seom comment".chars());
        assert_eq!(Token::Eof, lexer.next_token());

        let mut lexer = Lexer::new("abc # comment \n xyz".chars());
        assert_eq!(Token::Identifier("abc".into()), lexer.next_token());
        assert_eq!(Token::Identifier("xyz".into()), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());
    }

    #[test]
    fn test_chars() {
        let mut lexer = Lexer::new("a+b-c".chars());
        assert_eq!(Token::Identifier("a".into()), lexer.next_token());
        assert_eq!(Token::Char('+'), lexer.next_token());
        assert_eq!(Token::Identifier("b".into()), lexer.next_token());
        assert_eq!(Token::Char('-'), lexer.next_token());
        assert_eq!(Token::Identifier("c".into()), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());
    }

    #[test]
    fn test_whitespaces() {
        let mut lexer = Lexer::new("    +a  b     c!    ".chars());
        assert_eq!(Token::Char('+'), lexer.next_token());
        assert_eq!(Token::Identifier("a".into()), lexer.next_token());
        assert_eq!(Token::Identifier("b".into()), lexer.next_token());
        assert_eq!(Token::Identifier("c".into()), lexer.next_token());
        assert_eq!(Token::Char('!'), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());

        let mut lexer = Lexer::new("\n    a \n\r  b \r \n   c \r\r  \n  ".chars());
        assert_eq!(Token::Identifier("a".into()), lexer.next_token());
        assert_eq!(Token::Identifier("b".into()), lexer.next_token());
        assert_eq!(Token::Identifier("c".into()), lexer.next_token());
        assert_eq!(Token::Eof, lexer.next_token());
    }
}
