use crate::{ContentItem, TuiConfig};

/// Parses a .neb file into a TuiConfig.
///
/// Format:
///   title <text>       – window title
///   border             – enable border
///   margin <n>         – margin size
///   ---                – separator between config and content
///   [color? style*] text  – styled text line  (bold | italic | underlined)
///   <blank line>       – empty line in output
///   # comment          – ignored everywhere
///   plain text         – unstyled line
pub fn parse(input: &str) -> Result<TuiConfig, String> {
    let (header, body) = match input.split_once("---") {
        Some((h, b)) => (h, b.trim_start_matches('\n')),
        None => ("", input),
    };

    let mut title = String::from("nebular");
    let mut border = false;
    let mut margin = 0u16;

    for line in header.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("title ") {
            title = rest.trim().to_string();
        } else if line == "border" {
            border = true;
        } else if let Some(rest) = line.strip_prefix("margin ") {
            margin = rest
                .trim()
                .parse()
                .map_err(|_| format!("invalid margin value: '{rest}'"))?;
        } else {
            return Err(format!("unknown directive: '{line}'"));
        }
    }

    let mut content = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if line.is_empty() {
            content.push(ContentItem {
                text: String::new(),
                color: None,
                style: None,
            });
        } else if let Some(inner) = line.strip_prefix('[') {
            let close = inner
                .find(']')
                .ok_or_else(|| format!("missing ']' in line: {line}"))?;
            let text = inner[close + 1..].trim().to_string();
            let mut color = None;
            let mut styles: Vec<String> = Vec::new();
            for attr in inner[..close].split_whitespace() {
                match attr {
                    "bold" | "italic" | "underlined" => styles.push(attr.to_string()),
                    _ => color = Some(attr.to_string()),
                }
            }
            content.push(ContentItem {
                text,
                color,
                style: if styles.is_empty() { None } else { Some(styles) },
            });
        } else {
            content.push(ContentItem {
                text: line.to_string(),
                color: None,
                style: None,
            });
        }
    }

    Ok(TuiConfig { title, border, margin, content })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_directives() {
        let input = "title Hello\nborder\nmargin 2\n---\n";
        let cfg = parse(input).unwrap();
        assert_eq!(cfg.title, "Hello");
        assert!(cfg.border);
        assert_eq!(cfg.margin, 2);
    }

    #[test]
    fn parses_styled_line() {
        let input = "---\n[blue bold] Hi there\n";
        let cfg = parse(input).unwrap();
        assert_eq!(cfg.content[0].text, "Hi there");
        assert_eq!(cfg.content[0].color.as_deref(), Some("blue"));
        assert_eq!(cfg.content[0].style.as_deref(), Some(["bold".to_string()].as_slice()));
    }

    #[test]
    fn parses_plain_and_blank_lines() {
        let input = "---\nplain text\n\n";
        let cfg = parse(input).unwrap();
        assert_eq!(cfg.content[0].text, "plain text");
        assert!(cfg.content[0].color.is_none());
        assert_eq!(cfg.content[1].text, "");
    }

    #[test]
    fn rejects_missing_bracket() {
        let input = "---\n[blue oops\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn ignores_comments() {
        let input = "# top comment\ntitle X\n---\n# body comment\n[red] hi\n";
        let cfg = parse(input).unwrap();
        assert_eq!(cfg.title, "X");
        assert_eq!(cfg.content.len(), 1);
    }
}
