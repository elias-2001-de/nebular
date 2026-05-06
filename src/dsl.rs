use crate::{ContentItem, TuiConfig};
use std::collections::VecDeque;

// ── Tokens ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum Token {
    Ident(String),
    Str(String),
    Dot,
    Eq,
    Num(u16),
    LBrace,
    RBrace,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '/' => {
                chars.next();
                match chars.peek() {
                    Some('/') => {
                        while chars.peek().map(|&c| c != '\n').unwrap_or(false) {
                            chars.next();
                        }
                    }
                    _ => return Err("unexpected '/' — did you mean '//'?".to_string()),
                }
            }
            '#' => {
                while chars.peek().map(|&c| c != '\n').unwrap_or(false) {
                    chars.next();
                }
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            '.' => {
                chars.next();
                tokens.push(Token::Dot);
            }
            '=' => {
                chars.next();
                tokens.push(Token::Eq);
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some('"') => s.push('"'),
                            Some('\\') => s.push('\\'),
                            Some('n') => s.push('\n'),
                            Some(c) => {
                                s.push('\\');
                                s.push(c);
                            }
                            None => return Err("unterminated string escape".to_string()),
                        },
                        Some(c) => s.push(c),
                        None => return Err("unterminated string literal".to_string()),
                    }
                }
                tokens.push(Token::Str(s));
            }
            c if c.is_ascii_digit() => {
                let mut n = String::new();
                while chars.peek().map(|&c| c.is_ascii_digit()).unwrap_or(false) {
                    n.push(chars.next().unwrap());
                }
                let num: u16 = n.parse().map_err(|_| format!("number too large: {n}"))?;
                tokens.push(Token::Num(num));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                while chars
                    .peek()
                    .map(|&c| c.is_alphanumeric() || c == '_' || c == '-')
                    .unwrap_or(false)
                {
                    ident.push(chars.next().unwrap());
                }
                tokens.push(Token::Ident(ident));
            }
            c => return Err(format!("unexpected character: {c:?}")),
        }
    }

    Ok(tokens)
}

// ── Parser ────────────────────────────────────────────────────────────────────

struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens: tokens.into() }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn expect_ident(&mut self, name: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Ident(s)) if s == name => Ok(()),
            Some(tok) => Err(format!("expected '{name}', found {tok:?}")),
            None => Err(format!("expected '{name}', found end of input")),
        }
    }

    /// page title="…" border margin=N { … }
    fn parse_page(&mut self) -> Result<TuiConfig, String> {
        self.expect_ident("page")?;

        let mut title = String::from("nebular");
        let mut border = false;
        let mut margin = 0u16;

        loop {
            match self.peek() {
                Some(Token::LBrace) => {
                    self.next();
                    break;
                }
                Some(Token::Ident(_)) => match self.next() {
                    Some(Token::Ident(key)) => match key.as_str() {
                        "title" => {
                            self.expect_eq()?;
                            match self.next() {
                                Some(Token::Str(s)) => title = s,
                                other => {
                                    return Err(format!(
                                        "expected string after 'title=', found {other:?}"
                                    ))
                                }
                            }
                        }
                        "border" => border = true,
                        "margin" => {
                            self.expect_eq()?;
                            match self.next() {
                                Some(Token::Num(n)) => margin = n,
                                other => {
                                    return Err(format!(
                                        "expected number after 'margin=', found {other:?}"
                                    ))
                                }
                            }
                        }
                        other => return Err(format!("unknown page attribute: '{other}'")),
                    },
                    _ => unreachable!(),
                },
                other => {
                    return Err(format!(
                        "unexpected token in page attributes: {other:?}"
                    ))
                }
            }
        }

        let content = self.parse_body()?;
        Ok(TuiConfig { title, border, margin, content })
    }

    /// Everything inside the { … } of a page block.
    fn parse_body(&mut self) -> Result<Vec<ContentItem>, String> {
        let mut items = Vec::new();

        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.next();
                    break;
                }
                // "" — blank/empty line
                Some(Token::Str(_)) => {
                    if let Some(Token::Str(s)) = self.next() {
                        items.push(ContentItem { text: s, color: None, style: None });
                    }
                }
                // .color.bold.italic "text"
                Some(Token::Dot) => {
                    items.push(self.parse_styled_item()?);
                }
                None => return Err("unexpected end of input — missing '}'".to_string()),
                other => {
                    return Err(format!("unexpected token in page body: {other:?}"))
                }
            }
        }

        Ok(items)
    }

    /// .color?.modifier* "text"
    fn parse_styled_item(&mut self) -> Result<ContentItem, String> {
        let mut color: Option<String> = None;
        let mut styles: Vec<String> = Vec::new();

        while self.peek() == Some(&Token::Dot) {
            self.next(); // consume '.'
            match self.next() {
                Some(Token::Ident(cls)) => {
                    if matches!(cls.as_str(), "bold" | "italic" | "underlined") {
                        styles.push(cls);
                    } else {
                        color = Some(cls);
                    }
                }
                other => {
                    return Err(format!("expected class name after '.', found {other:?}"))
                }
            }
        }

        match self.next() {
            Some(Token::Str(s)) => Ok(ContentItem {
                text: s,
                color,
                style: if styles.is_empty() { None } else { Some(styles) },
            }),
            other => Err(format!("expected string after modifiers, found {other:?}")),
        }
    }

    fn expect_eq(&mut self) -> Result<(), String> {
        match self.next() {
            Some(Token::Eq) => Ok(()),
            Some(tok) => Err(format!("expected '=', found {tok:?}")),
            None => Err("expected '='".to_string()),
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn parse(input: &str) -> Result<TuiConfig, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    let config = parser.parse_page()?;
    if parser.peek().is_some() {
        return Err("unexpected content after closing '}'".to_string());
    }
    Ok(config)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_page_attributes() {
        let cfg = parse(r#"page title="Hello" border margin=2 {}"#).unwrap();
        assert_eq!(cfg.title, "Hello");
        assert!(cfg.border);
        assert_eq!(cfg.margin, 2);
    }

    #[test]
    fn parses_styled_item() {
        let cfg = parse(r#"page { .blue.bold "Hi there" }"#).unwrap();
        assert_eq!(cfg.content[0].text, "Hi there");
        assert_eq!(cfg.content[0].color.as_deref(), Some("blue"));
        assert_eq!(
            cfg.content[0].style.as_deref(),
            Some(["bold".to_string()].as_slice())
        );
    }

    #[test]
    fn parses_plain_and_blank() {
        let cfg = parse(r#"page { "plain text" "" }"#).unwrap();
        assert_eq!(cfg.content[0].text, "plain text");
        assert!(cfg.content[0].color.is_none());
        assert_eq!(cfg.content[1].text, "");
    }

    #[test]
    fn rejects_dot_without_ident() {
        assert!(parse(r#"page { . "oops" }"#).is_err());
    }

    #[test]
    fn ignores_comments() {
        let input = "// top\npage { // inline\n.red \"hi\" }";
        let cfg = parse(input).unwrap();
        assert_eq!(cfg.content.len(), 1);
    }

    #[test]
    fn rejects_unknown_page_attr() {
        assert!(parse(r#"page foo=1 {}"#).is_err());
    }

    #[test]
    fn multiple_modifiers() {
        let cfg = parse(r#"page { .cyan.bold.italic "text" }"#).unwrap();
        assert_eq!(cfg.content[0].color.as_deref(), Some("cyan"));
        let styles = cfg.content[0].style.as_deref().unwrap();
        assert!(styles.contains(&"bold".to_string()));
        assert!(styles.contains(&"italic".to_string()));
    }
}
