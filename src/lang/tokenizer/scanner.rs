use super::error::ScanError;
use super::token::{Token, TokenType};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::CharIndices;

pub const LOX_KEYWORDS: &[(&str, TokenType)] = &[
    ("and", TokenType::And),
    ("class", TokenType::Class),
    ("else", TokenType::Else),
    ("false", TokenType::False),
    ("for", TokenType::For),
    ("fun", TokenType::Fun),
    ("if", TokenType::If),
    ("nil", TokenType::Nil),
    ("or", TokenType::Or),
    ("print", TokenType::Print),
    ("return", TokenType::Return),
    ("super", TokenType::Super),
    ("this", TokenType::This),
    ("true", TokenType::True),
    ("var", TokenType::Var),
    ("while", TokenType::While),
    ("break", TokenType::Break),
    ("continue", TokenType::Continue),
    ("static", TokenType::Static),
];

pub struct Scanner<'src> {
    src: &'src str,
    ci: Peekable<CharIndices<'src>>,
    marker: usize,  // marker at token start
    current: usize, // current location
    keywords: HashMap<&'static str, TokenType>,
    iter_done: bool,
}

impl<'src> Scanner<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            ci: src.char_indices().peekable(),
            marker: 0,
            current: 0,
            keywords: make_keyword_map(),
            iter_done: false,
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'src>, ScanError> {
        self.skip_ws_and_comments();

        if self.is_eof() {
            return Ok(self.make_token(TokenType::Eof, "", self.position_now()));
        }

        self.set_marker();
        let ch = self.next_char().unwrap(); // we already confirmed we're not at eof yet.

        let (kind, lexeme) = match ch {
            '(' => (TokenType::LeftParen, self.take_slice()),
            ')' => (TokenType::RightParen, self.take_slice()),
            '{' => (TokenType::LeftBrace, self.take_slice()),
            '}' => (TokenType::RightBrace, self.take_slice()),
            ',' => (TokenType::Comma, self.take_slice()),
            ';' => (TokenType::Semicolon, self.take_slice()),
            '+' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::PlusEqual, self.take_slice())
                } else {
                    (TokenType::Plus, self.take_slice())
                }
            }
            '-' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::MinusEqual, self.take_slice())
                } else {
                    (TokenType::Minus, self.take_slice())
                }
            }
            '/' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::SlashEqual, self.take_slice())
                } else {
                    (TokenType::Slash, self.take_slice())
                }
            }
            '*' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::StarEqual, self.take_slice())
                } else {
                    (TokenType::Star, self.take_slice())
                }
            }
            '!' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::BangEqual, self.take_slice())
                } else {
                    (TokenType::Bang, self.take_slice())
                }
            }
            '=' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::EqualEqual, self.take_slice())
                } else {
                    (TokenType::Equal, self.take_slice())
                }
            }
            '>' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::GreaterEqual, self.take_slice())
                } else {
                    (TokenType::Greater, self.take_slice())
                }
            }
            '<' => {
                if self.next_char_if(|c| *c == '=').is_some() {
                    (TokenType::LessEqual, self.take_slice())
                } else {
                    (TokenType::Less, self.take_slice())
                }
            }
            '0'..='9' => {
                let num_literal = self.scan_number(ch)?;
                (TokenType::Number, num_literal)
            }
            '.' if self.peek_is_digit() => {
                let num_literal = self.scan_number(ch)?;
                (TokenType::Number, num_literal)
            }
            '.' => (TokenType::Dot, self.take_slice()),
            '"' => {
                let lexeme = self.scan_string()?;
                (TokenType::String, lexeme)
            }
            _ if is_ident_char(ch) => {
                let lexeme = self.scan_identifier();
                let kind = *self.keywords.get(lexeme).unwrap_or(&TokenType::Identifier);
                (kind, lexeme)
            }
            _ => return Err(ScanError::InvalidToken(ch.to_string(), self.position_now())),
        };

        Ok(self.make_token(kind, lexeme, self.position_start()))
    }

    // ---------- scanners ----------
    fn scan_number(&mut self, first: char) -> Result<&'src str, ScanError> {
        // we've already checked that a dot is followed by a num.
        // so unlike below, we don't need to verify that here in the beginning.
        let mut dot_cnt = if first == '.' { 1 } else { 0 };

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                let _ = self.next_char(); // cannot fail
                continue;
            } else if *c == '.' && dot_cnt == 0 {
                let _ = self.next_char(); // cannot fail

                if !self.peek_is_digit() {
                    return Err(ScanError::InvalidNumber(
                        self.take_slice().to_string(),
                        self.position_start(),
                    ));
                }

                dot_cnt += 1;
                continue;
            } else {
                break;
            }
        }

        Ok(self.take_slice())
    }

    fn scan_string(&mut self) -> Result<&'src str, ScanError> {
        let mut in_escape = false;

        while let Some(c) = self.next_char() {
            if in_escape {
                in_escape = false;
                continue;
            }
            if c == '\\' {
                in_escape = true;
                continue;
            }
            if c == '"' {
                return Ok(self.take_slice());
            }
        }

        Err(ScanError::StrMissingTerminator(
            self.take_slice().to_string(),
            self.position_now(),
        ))
    }

    fn scan_identifier(&mut self) -> &'src str {
        while let Some(_) = self.next_char_if(|c| is_ident_char(*c)) {}
        self.take_slice()
    }

    // ---------- skipping / helpers ----------
    fn skip_ws_and_comments(&mut self) {
        loop {
            // whitespace
            while let Some(_) = self.next_char_if(|c| c.is_whitespace()) {}
            // line comment
            if self.in_comment() {
                // consume until newline
                // once we hit a newline, the whitespace loop at the top will cut it off.
                while let Some(_) = self.next_char_if(|c| *c != '\n') {}
            } else {
                break;
            }
        }
    }

    #[inline]
    fn in_comment(&self) -> bool {
        self.src.as_bytes().get(self.current..self.current + 2) == Some(b"//")
    }

    #[inline]
    fn is_eof(&mut self) -> bool {
        self.ci.peek().is_none()
    }

    #[inline]
    fn peek(&mut self) -> Option<&char> {
        self.ci.peek().map(|(_, c)| c)
    }

    #[inline]
    fn peek_is_digit(&mut self) -> bool {
        self.ci.peek().map_or(false, |(_, c)| c.is_ascii_digit())
    }

    fn next_char(&mut self) -> Option<char> {
        self.ci.next().map(|ch| {
            self.update_pos(ch);
            return ch.1;
        })
    }

    fn next_char_if<F>(&mut self, f: F) -> Option<char>
    where
        F: FnOnce(&char) -> bool,
    {
        if let Some(c) = self.ci.next_if(|(_, c)| f(c)) {
            self.update_pos(c);
            Some(c.1)
        } else {
            None
        }
    }

    fn update_pos(&mut self, (idx, c): (usize, char)) {
        self.current = idx + c.len_utf8()
    }

    fn set_marker(&mut self) {
        self.marker = self.current;
    }

    fn take_slice(&mut self) -> &'src str {
        debug_assert!(self.marker <= self.current, "marker crossed index");
        &self.src[self.marker..self.current]
    }

    #[inline]
    fn make_token(&mut self, kind: TokenType, lex: &'src str, position: usize) -> Token<'src> {
        Token::new(kind, lex, position)
    }

    #[inline]
    fn position_now(&self) -> usize {
        self.current
    }

    #[inline]
    fn position_start(&self) -> usize {
        self.marker
    }
}

// Optional: ergonomic iteration
impl<'src> Iterator for Scanner<'src> {
    type Item = Result<Token<'src>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_done {
            return None;
        }
        match self.next_token() {
            Ok(tok) => {
                if tok.token_type == TokenType::Eof {
                    self.iter_done = true;
                }
                return Some(Ok(tok));
            }
            res => Some(res),
        }
    }
}

#[inline]
fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
}

fn make_keyword_map() -> HashMap<&'static str, TokenType> {
    let mut map = HashMap::with_capacity(LOX_KEYWORDS.len());
    for &(k, v) in LOX_KEYWORDS {
        map.insert(k, v);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_single_tokens() {
        let src = "(){},;+";
        let mut scanner = Scanner::new(src);

        let expected_tokens = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Semicolon,
            TokenType::Plus,
            TokenType::Eof,
        ];

        for expected in expected_tokens {
            let token = scanner.next_token().unwrap();
            assert_eq!(token.token_type, expected);
        }
    }

    #[test]
    fn test_scan_keywords() {
        let src =
            "and class else false for fun if nil or print return super this true var while break";
        let mut scanner = Scanner::new(src);

        for &(keyword, token_type) in LOX_KEYWORDS {
            let token = scanner.next_token().unwrap();
            assert_eq!(token.token_type, token_type);
            assert_eq!(token.lexeme, keyword);
        }

        let eof = scanner.next_token().unwrap();
        assert_eq!(eof.token_type, TokenType::Eof);
    }

    #[test]
    fn test_scan_identifiers() {
        let src = "foo bar _baz qux123";
        let mut scanner = Scanner::new(src);

        let expected_identifiers = vec!["foo", "bar", "_baz", "qux123"];

        for expected in expected_identifiers {
            let token = scanner.next_token().unwrap();
            assert_eq!(token.token_type, TokenType::Identifier);
            assert_eq!(token.lexeme, expected);
        }

        let eof = scanner.next_token().unwrap();
        assert_eq!(eof.token_type, TokenType::Eof);
    }

    #[test]
    fn test_scan_numbers() {
        let src = "123 45.67 .89";
        let mut scanner = Scanner::new(src);

        let expected_numbers = vec!["123", "45.67", ".89"];

        for expected in expected_numbers {
            let token = scanner.next_token().unwrap();
            assert_eq!(token.token_type, TokenType::Number);
            assert_eq!(token.lexeme, expected);
        }

        let eof = scanner.next_token().unwrap();
        assert_eq!(eof.token_type, TokenType::Eof);
    }

    #[test]
    fn test_scan_strings() {
        let src = "\"hello\" \"world\" \"escaped \\\"quote\\\"\"";
        let mut scanner = Scanner::new(src);

        let expected_strings = vec!["\"hello\"", "\"world\"", "\"escaped \\\"quote\\\"\""];

        for expected in expected_strings {
            let token = scanner.next_token().unwrap();
            assert_eq!(token.token_type, TokenType::String);
            assert_eq!(token.lexeme, expected);
        }

        let eof = scanner.next_token().unwrap();
        assert_eq!(eof.token_type, TokenType::Eof);
    }

    #[test]
    fn test_skip_whitespace_and_comments() {
        let src = "  // this is a comment\n 123 // another comment\n \"string\"";
        let mut scanner = Scanner::new(src);

        let token1 = scanner.next_token().unwrap();
        assert_eq!(token1.token_type, TokenType::Number);
        assert_eq!(token1.lexeme, "123");

        let token2 = scanner.next_token().unwrap();
        assert_eq!(token2.token_type, TokenType::String);
        assert_eq!(token2.lexeme, "\"string\"");

        let eof = scanner.next_token().unwrap();
        assert_eq!(eof.token_type, TokenType::Eof);
    }

    #[test]
    fn test_invalid_tokens() {
        let src = "@";
        let mut scanner = Scanner::new(src);

        let error = scanner.next_token().unwrap_err();
        match error {
            ScanError::InvalidToken(lexeme, _) => assert_eq!(lexeme, "@"),
            _ => panic!("Expected InvalidToken error"),
        }
    }
}
