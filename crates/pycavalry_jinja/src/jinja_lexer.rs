// This file is part of pycavalry.
//
// pycavalry is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{iter::Peekable, ops::Range, str::CharIndices};

use pycavalry_diagnostics::Diag;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TokenType {
    Add,
    Assign,
    Colon,
    Comma,
    Div,
    Dot,
    Eq,
    Floordiv,
    Gt,
    Gteq,
    Lbrace,
    Lbracket,
    Lparen,
    Lt,
    Lteq,
    Mod,
    Mul,
    Ne,
    Pipe,
    Pow,
    Rbrace,
    Rbracket,
    Rparen,
    Semicolon,
    Sub,
    Tilde,
    Float,
    Integer,
    Name,
    String,
    VariableBegin,
    VariableEnd,
    RawBegin,
    RawEnd,
    Comment,
    StatementBegin,
    StatementEnd,
    Eof,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for TextRange {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub typ: TokenType,
    pub range: TextRange,
}

impl Token {
    fn new(typ: TokenType, range: TextRange) -> Self {
        Self { typ, range }
    }
}

impl From<(TokenType, TextRange)> for Token {
    fn from(value: (TokenType, TextRange)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<(TokenType, Range<usize>)> for Token {
    fn from(value: (TokenType, Range<usize>)) -> Self {
        Self::new(value.0, value.1.into())
    }
}

fn add_token(
    tokens: &mut Vec<Token>,
    chars: &mut Peekable<CharIndices<'_>>,
    i: usize,
    len: usize,
    typ: TokenType,
) {
    tokens.push((typ, (i..i + len)).into());
    for _ in 1..len {
        // TODO: This does not work properly when popping multi byte characters
        chars.next();
    }
}

fn tokenize_string(
    tokens: &mut Vec<Token>,
    chars: &mut Peekable<CharIndices<'_>>,
    opening_quote: char,
    start: usize,
) {
}

fn tokenize_number(tokens: &mut Vec<Token>, chars: &mut Peekable<CharIndices<'_>>, start: usize) {
    let mut is_int = true;
    let mut last_was_dot = false;
    loop {
        let Some((next_i, next_char)) = chars.peek() else {
            panic!("Unexpected EOF")
        };
        match *next_char {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                last_was_dot = false;
                chars.next();
            }
            '.' => {
                last_was_dot = true;
                if is_int == false {
                    let range = start..next_i + 1;
                    panic!("Failed to parse float, multiple dots found. {:?}", range);
                }
                is_int = false;
                chars.next();
            }
            _ => {
                if last_was_dot {
                    let range = start..next_i + 1;
                    panic!("Failed to parse number. {:?}", range);
                }
                let typ = if is_int {
                    TokenType::Integer
                } else {
                    TokenType::Float
                };
                tokens.push((typ, start..*next_i).into());
                return;
            }
        }
    }
}

fn tokenize_identifier(
    tokens: &mut Vec<Token>,
    chars: &mut Peekable<CharIndices<'_>>,
    opening_quote: char,
    start: usize,
) {
}

fn tokenize_jinja(tokens: &mut Vec<Token>, chars: &mut Peekable<CharIndices<'_>>) {
    loop {
        let Some((curr_i, curr_char)) = chars.next() else {
            panic!("Unexpected EOF")
        };
        match curr_char {
            ' ' | '\n' | '\t' => continue,
            '+' => add_token(tokens, chars, curr_i, 1, TokenType::Add),
            '-' => add_token(tokens, chars, curr_i, 1, TokenType::Sub),
            '/' => match chars.peek() {
                Some((_, '/')) => add_token(tokens, chars, curr_i, 2, TokenType::Floordiv),
                _ => add_token(tokens, chars, curr_i, 1, TokenType::Div),
            },
            '*' => match chars.peek() {
                Some((_, '*')) => add_token(tokens, chars, curr_i, 2, TokenType::Pow),
                _ => add_token(tokens, chars, curr_i, 1, TokenType::Mul),
            },
            '%' => match chars.peek() {
                Some((_, '}')) => {
                    add_token(tokens, chars, curr_i, 2, TokenType::StatementEnd);
                    return;
                }
                _ => add_token(tokens, chars, curr_i, 1, TokenType::Mod),
            },
            '~' => add_token(tokens, chars, curr_i, 1, TokenType::Tilde),
            '[' => add_token(tokens, chars, curr_i, 1, TokenType::Lbracket),
            ']' => add_token(tokens, chars, curr_i, 1, TokenType::Rbracket),
            '(' => add_token(tokens, chars, curr_i, 1, TokenType::Lparen),
            ')' => add_token(tokens, chars, curr_i, 1, TokenType::Rparen),
            '{' => add_token(tokens, chars, curr_i, 1, TokenType::Lbrace),
            '}' => match chars.peek() {
                Some((_, '}')) => {
                    add_token(tokens, chars, curr_i, 2, TokenType::VariableEnd);
                    return;
                }
                _ => add_token(tokens, chars, curr_i, 1, TokenType::Rbrace),
            },
            '>' => match chars.peek() {
                Some((_, '=')) => add_token(tokens, chars, curr_i, 2, TokenType::Gteq),
                _ => add_token(tokens, chars, curr_i, 1, TokenType::Gt),
            },
            '<' => match chars.peek() {
                Some((_, '=')) => add_token(tokens, chars, curr_i, 2, TokenType::Lteq),
                _ => add_token(tokens, chars, curr_i, 1, TokenType::Lt),
            },
            '=' => match chars.peek() {
                Some((_, '=')) => add_token(tokens, chars, curr_i, 2, TokenType::Eq),
                _ => add_token(tokens, chars, curr_i, 1, TokenType::Assign),
            },
            '.' => add_token(tokens, chars, curr_i, 1, TokenType::Dot),
            ':' => add_token(tokens, chars, curr_i, 1, TokenType::Colon),
            '|' => add_token(tokens, chars, curr_i, 1, TokenType::Pipe),
            ',' => add_token(tokens, chars, curr_i, 1, TokenType::Comma),
            ';' => add_token(tokens, chars, curr_i, 1, TokenType::Semicolon),
            '!' => match chars.peek() {
                Some((_, '=')) => add_token(tokens, chars, curr_i, 2, TokenType::Ne),
                _ => (),
            },
            '"' | '\'' => tokenize_string(tokens, chars, curr_char, curr_i),
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                tokenize_number(tokens, chars, curr_i)
            }
            c => tokenize_identifier(tokens, chars, c, curr_i),
        };
    }
}

fn tokenize_comment(
    tokens: &mut Vec<Token>,
    chars: &mut Peekable<CharIndices<'_>>,
    content: &str,
    start: usize,
) {
    loop {
        match chars.next() {
            Some((_, curr_char)) => {
                if curr_char != '#' {
                    continue;
                }
                let Some((next_i, next_char)) = chars.peek() else {
                    continue;
                };
                if *next_char == '}' {
                    tokens.push((TokenType::Comment, start..next_i + 1).into());
                    return;
                }
            }
            None => {
                tokens.push((TokenType::Comment, start..content.len()).into());
                return;
            }
        }
    }
}

pub fn tokenize(content: &str) -> Result<Vec<Token>, Vec<Box<dyn Diag>>> {
    let mut tokens = vec![];

    let mut chars = content.char_indices().peekable();

    loop {
        let maybe_curr = chars.next();
        let Some((curr_i, curr_char)) = maybe_curr else {
            tokens.push((TokenType::Eof, (content.len()..content.len())).into());
            break;
        };
        if curr_char == '{' {
            let Some(next) = chars.peek() else { continue };
            match next.1 {
                '{' => {
                    add_token(&mut tokens, &mut chars, curr_i, 2, TokenType::VariableBegin);
                    tokenize_jinja(&mut tokens, &mut chars);
                }
                '%' => {
                    add_token(
                        &mut tokens,
                        &mut chars,
                        curr_i,
                        2,
                        TokenType::StatementBegin,
                    );
                    tokenize_jinja(&mut tokens, &mut chars);
                }
                '#' => {
                    chars.next();
                    tokenize_comment(&mut tokens, &mut chars, content, curr_i);
                }
                _ => (),
            };
        }
    }

    Ok(tokens)
}
