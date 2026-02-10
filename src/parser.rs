/// Regex parser: converts a pattern string into an AST.

use crate::ast::*;

pub struct Parser {
    chars: Vec<char>,
    pos: usize,
    group_count: usize,
}

impl Parser {
    pub fn new(pattern: &str) -> Self {
        Parser {
            chars: pattern.chars().collect(),
            pos: 0,
            group_count: 0,
        }
    }

    /// Parse the full pattern and return an AST node.
    pub fn parse(&mut self) -> Result<AstNode, String> {
        let node = self.parse_alternation()?;
        if self.pos < self.chars.len() {
            return Err(format!(
                "Unexpected character '{}' at position {}",
                self.chars[self.pos], self.pos
            ));
        }
        Ok(node)
    }

    /// Returns total number of capturing groups found.
    pub fn group_count(&self) -> usize {
        self.group_count
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', got '{}'", expected, c)),
            None => Err(format!("Expected '{}', got end of pattern", expected)),
        }
    }

    /// Parse alternation: `a|b|c`
    fn parse_alternation(&mut self) -> Result<AstNode, String> {
        let mut branches = vec![self.parse_concat()?];
        while self.peek() == Some('|') {
            self.advance(); // consume '|'
            branches.push(self.parse_concat()?);
        }
        if branches.len() == 1 {
            Ok(branches.pop().unwrap())
        } else {
            Ok(AstNode::Alternation(branches))
        }
    }

    /// Parse concatenation: `abc`
    fn parse_concat(&mut self) -> Result<AstNode, String> {
        let mut nodes = Vec::new();
        while let Some(ch) = self.peek() {
            if ch == ')' || ch == '|' {
                break;
            }
            nodes.push(self.parse_quantified()?);
        }
        if nodes.len() == 1 {
            Ok(nodes.pop().unwrap())
        } else {
            Ok(AstNode::Concat(nodes))
        }
    }

    /// Parse an atom possibly followed by a quantifier.
    fn parse_quantified(&mut self) -> Result<AstNode, String> {
        let node = self.parse_atom()?;
        if let Some(ch) = self.peek() {
            match ch {
                '*' | '+' | '?' => {
                    self.advance();
                    let kind = match ch {
                        '*' => QuantifierKind::Star,
                        '+' => QuantifierKind::Plus,
                        '?' => QuantifierKind::Question,
                        _ => unreachable!(),
                    };
                    let greedy = if self.peek() == Some('?') {
                        self.advance();
                        false
                    } else {
                        true
                    };
                    Ok(AstNode::Quantifier {
                        node: Box::new(node),
                        kind,
                        greedy,
                    })
                }
                '{' => self.parse_brace_quantifier(node),
                _ => Ok(node),
            }
        } else {
            Ok(node)
        }
    }

    /// Parse `{n}`, `{n,}`, `{n,m}` quantifier.
    fn parse_brace_quantifier(&mut self, node: AstNode) -> Result<AstNode, String> {
        let save_pos = self.pos;
        self.advance(); // consume '{'

        // Try to parse as a quantifier, fall back to literal if it doesn't parse
        match self.try_parse_brace_contents() {
            Ok((kind, greedy)) => Ok(AstNode::Quantifier {
                node: Box::new(node),
                kind,
                greedy,
            }),
            Err(_) => {
                // Not a valid quantifier, revert position — the '{' was a literal
                self.pos = save_pos;
                Ok(node)
            }
        }
    }

    fn try_parse_brace_contents(&mut self) -> Result<(QuantifierKind, bool), String> {
        let n = self.parse_number()?;
        let kind = if self.peek() == Some(',') {
            self.advance(); // consume ','
            if self.peek() == Some('}') {
                QuantifierKind::AtLeast(n)
            } else {
                let m = self.parse_number()?;
                QuantifierKind::Range(n, m)
            }
        } else {
            QuantifierKind::Exact(n)
        };
        self.expect('}')?;
        let greedy = if self.peek() == Some('?') {
            self.advance();
            false
        } else {
            true
        };
        Ok((kind, greedy))
    }

    fn parse_number(&mut self) -> Result<usize, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err("Expected number".to_string());
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<usize>().map_err(|e| e.to_string())
    }

    /// Parse a single atom (literal, class, group, anchor, etc.)
    fn parse_atom(&mut self) -> Result<AstNode, String> {
        match self.peek() {
            None => Err("Unexpected end of pattern".to_string()),
            Some('(') => self.parse_group(),
            Some('[') => self.parse_char_class(),
            Some('.') => {
                self.advance();
                Ok(AstNode::Dot)
            }
            Some('^') => {
                self.advance();
                Ok(AstNode::Anchor(AnchorKind::Start))
            }
            Some('$') => {
                self.advance();
                Ok(AstNode::Anchor(AnchorKind::End))
            }
            Some('\\') => self.parse_escape(),
            Some(ch) => {
                self.advance();
                Ok(AstNode::Literal(ch))
            }
        }
    }

    /// Parse an escape sequence.
    fn parse_escape(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume '\\'
        match self.advance() {
            None => Err("Unexpected end of pattern after '\\'".to_string()),
            Some('d') => Ok(AstNode::ShorthandClass(ShorthandKind::Digit)),
            Some('D') => Ok(AstNode::ShorthandClass(ShorthandKind::NonDigit)),
            Some('w') => Ok(AstNode::ShorthandClass(ShorthandKind::Word)),
            Some('W') => Ok(AstNode::ShorthandClass(ShorthandKind::NonWord)),
            Some('s') => Ok(AstNode::ShorthandClass(ShorthandKind::Space)),
            Some('S') => Ok(AstNode::ShorthandClass(ShorthandKind::NonSpace)),
            Some('b') => Ok(AstNode::Anchor(AnchorKind::WordBoundary)),
            Some('B') => Ok(AstNode::Anchor(AnchorKind::NonWordBoundary)),
            Some(ch) if ch.is_ascii_digit() && ch != '0' => {
                // Backreference \1 through \9
                let idx = (ch as u8 - b'0') as usize;
                Ok(AstNode::Backreference(idx))
            }
            Some('n') => Ok(AstNode::Literal('\n')),
            Some('r') => Ok(AstNode::Literal('\r')),
            Some('t') => Ok(AstNode::Literal('\t')),
            Some(ch) => {
                // Escaped literal: \., \*, \\, etc.
                Ok(AstNode::Literal(ch))
            }
        }
    }

    /// Parse a group: `(...)`, `(?:...)`, `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)`.
    fn parse_group(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume '('

        if self.peek() == Some('?') {
            self.advance(); // consume '?'
            match self.peek() {
                Some(':') => {
                    self.advance();
                    let node = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(AstNode::NonCapturingGroup {
                        node: Box::new(node),
                    })
                }
                Some('=') => {
                    self.advance();
                    let node = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(AstNode::Lookahead {
                        node: Box::new(node),
                        positive: true,
                    })
                }
                Some('!') => {
                    self.advance();
                    let node = self.parse_alternation()?;
                    self.expect(')')?;
                    Ok(AstNode::Lookahead {
                        node: Box::new(node),
                        positive: false,
                    })
                }
                Some('<') => {
                    self.advance(); // consume '<'
                    match self.peek() {
                        Some('=') => {
                            self.advance();
                            let node = self.parse_alternation()?;
                            self.expect(')')?;
                            Ok(AstNode::Lookbehind {
                                node: Box::new(node),
                                positive: true,
                            })
                        }
                        Some('!') => {
                            self.advance();
                            let node = self.parse_alternation()?;
                            self.expect(')')?;
                            Ok(AstNode::Lookbehind {
                                node: Box::new(node),
                                positive: false,
                            })
                        }
                        _ => Err("Invalid lookbehind syntax".to_string()),
                    }
                }
                _ => Err("Invalid group syntax after '(?'".to_string()),
            }
        } else {
            // Capturing group
            self.group_count += 1;
            let index = self.group_count;
            let node = self.parse_alternation()?;
            self.expect(')')?;
            Ok(AstNode::Group {
                index,
                node: Box::new(node),
            })
        }
    }

    /// Parse a character class: `[abc]`, `[a-z]`, `[^abc]`.
    fn parse_char_class(&mut self) -> Result<AstNode, String> {
        self.advance(); // consume '['
        let negated = if self.peek() == Some('^') {
            self.advance();
            true
        } else {
            false
        };

        let mut items = Vec::new();
        // Allow ']' as first character in class
        if self.peek() == Some(']') {
            self.advance();
            items.push(ClassItem::Literal(']'));
        }

        while self.peek() != Some(']') {
            match self.peek() {
                None => return Err("Unterminated character class".to_string()),
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        None => return Err("Unexpected end in character class escape".to_string()),
                        Some('d') => items.push(ClassItem::Shorthand(ShorthandKind::Digit)),
                        Some('D') => items.push(ClassItem::Shorthand(ShorthandKind::NonDigit)),
                        Some('w') => items.push(ClassItem::Shorthand(ShorthandKind::Word)),
                        Some('W') => items.push(ClassItem::Shorthand(ShorthandKind::NonWord)),
                        Some('s') => items.push(ClassItem::Shorthand(ShorthandKind::Space)),
                        Some('S') => items.push(ClassItem::Shorthand(ShorthandKind::NonSpace)),
                        Some('n') => items.push(ClassItem::Literal('\n')),
                        Some('r') => items.push(ClassItem::Literal('\r')),
                        Some('t') => items.push(ClassItem::Literal('\t')),
                        Some(ch) => items.push(ClassItem::Literal(ch)),
                    }
                }
                Some(ch) => {
                    self.advance();
                    // Check for range like a-z
                    if self.peek() == Some('-')
                        && self.pos + 1 < self.chars.len()
                        && self.chars[self.pos + 1] != ']'
                    {
                        self.advance(); // consume '-'
                        let end_ch = match self.peek() {
                            Some('\\') => {
                                self.advance();
                                self.advance().ok_or("Unexpected end in range")?
                            }
                            Some(c) => {
                                self.advance();
                                c
                            }
                            None => return Err("Unexpected end in character class range".to_string()),
                        };
                        items.push(ClassItem::Range(ch, end_ch));
                    } else {
                        items.push(ClassItem::Literal(ch));
                    }
                }
            }
        }
        self.advance(); // consume ']'
        Ok(AstNode::CharClass {
            ranges: items,
            negated,
        })
    }
}
