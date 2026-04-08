#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateDocument {
    pub segments: Vec<TemplateSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSegment {
    Text(String),
    Placeholder(TemplatePlaceholder),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePlaceholder {
    pub path: TemplatePath,
    pub filters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePath {
    pub namespace: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateParseError {
    MalformedPlaceholder(String),
    UnclosedPlaceholder { start: usize },
}

const KNOWN_NAMESPACES: [&str; 4] = ["meta", "token", "palette", "param"];

pub fn parse_template(input: &str) -> Result<TemplateDocument, TemplateParseError> {
    let mut segments = Vec::new();
    let mut cursor = 0;

    while let Some(open_offset) = input[cursor..].find("{{") {
        let open = cursor + open_offset;
        if open > cursor {
            push_text_segment(&mut segments, &input[cursor..open]);
        }

        let inner_start = open + 2;
        let Some(close_offset) = input[inner_start..].find("}}") else {
            let raw = &input[inner_start..];
            if looks_like_template_syntax(raw) {
                return Err(TemplateParseError::UnclosedPlaceholder { start: open });
            }
            push_text_segment(&mut segments, &input[open..]);
            cursor = input.len();
            break;
        };
        let close = inner_start + close_offset;
        let raw = &input[inner_start..close];
        match parse_placeholder(raw.trim()) {
            Ok(placeholder) => segments.push(TemplateSegment::Placeholder(placeholder)),
            Err(error) if looks_like_template_syntax(raw) => return Err(error),
            Err(_) => push_text_segment(&mut segments, &input[open..close + 2]),
        }
        cursor = close + 2;
    }

    if cursor < input.len() {
        push_text_segment(&mut segments, &input[cursor..]);
    }

    if segments.is_empty() {
        segments.push(TemplateSegment::Text(String::new()));
    }

    Ok(TemplateDocument { segments })
}

fn parse_placeholder(raw: &str) -> Result<TemplatePlaceholder, TemplateParseError> {
    let mut parts = raw.split('|').map(str::trim);
    let path = parts
        .next()
        .filter(|part| !part.is_empty())
        .ok_or_else(|| TemplateParseError::MalformedPlaceholder(raw.to_string()))?;

    let path = parse_path(path)?;
    let mut filters = Vec::new();

    for filter in parts {
        if filter.is_empty() {
            return Err(TemplateParseError::MalformedPlaceholder(raw.to_string()));
        }
        filters.push(filter.to_string());
    }

    Ok(TemplatePlaceholder { path, filters })
}

fn parse_path(raw: &str) -> Result<TemplatePath, TemplateParseError> {
    let Some((namespace, key)) = raw.split_once('.') else {
        return Err(TemplateParseError::MalformedPlaceholder(raw.to_string()));
    };

    if namespace.trim().is_empty()
        || key.trim().is_empty()
        || namespace.contains('.')
        || key.contains('.')
    {
        return Err(TemplateParseError::MalformedPlaceholder(raw.to_string()));
    }

    Ok(TemplatePath {
        namespace: namespace.trim().to_string(),
        key: key.trim().to_string(),
    })
}

fn looks_like_template_syntax(raw: &str) -> bool {
    let raw = raw.trim();
    if raw.is_empty() || raw.contains('.') || raw.contains('|') {
        return true;
    }

    KNOWN_NAMESPACES.iter().any(|namespace| {
        raw == *namespace
            || raw
                .strip_prefix(namespace)
                .and_then(|rest| rest.chars().next())
                .is_some_and(char::is_whitespace)
    })
}

fn push_text_segment(segments: &mut Vec<TemplateSegment>, text: &str) {
    if text.is_empty() {
        return;
    }

    match segments.last_mut() {
        Some(TemplateSegment::Text(existing)) => existing.push_str(text),
        _ => segments.push(TemplateSegment::Text(text.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TemplateDocument, TemplateParseError, TemplatePath, TemplatePlaceholder, TemplateSegment,
        parse_template,
    };

    #[test]
    fn parses_plain_text_only() {
        let ast = parse_template("plain text only").unwrap();

        assert_eq!(
            ast,
            TemplateDocument {
                segments: vec![TemplateSegment::Text("plain text only".to_string())],
            }
        );
    }

    #[test]
    fn parses_single_placeholder() {
        let ast = parse_template("{{token.comment}}").unwrap();

        assert_eq!(
            ast,
            TemplateDocument {
                segments: vec![TemplateSegment::Placeholder(TemplatePlaceholder {
                    path: TemplatePath {
                        namespace: "token".to_string(),
                        key: "comment".to_string(),
                    },
                    filters: vec![],
                })],
            }
        );
    }

    #[test]
    fn parses_placeholder_with_whitespace() {
        let ast = parse_template("{{  token.comment   }}").unwrap();

        assert_eq!(
            ast,
            TemplateDocument {
                segments: vec![TemplateSegment::Placeholder(TemplatePlaceholder {
                    path: TemplatePath {
                        namespace: "token".to_string(),
                        key: "comment".to_string(),
                    },
                    filters: vec![],
                })],
            }
        );
    }

    #[test]
    fn parses_placeholder_with_one_filter() {
        let ast = parse_template("{{token.comment | opaque_hex}}").unwrap();

        assert_eq!(
            ast,
            TemplateDocument {
                segments: vec![TemplateSegment::Placeholder(TemplatePlaceholder {
                    path: TemplatePath {
                        namespace: "token".to_string(),
                        key: "comment".to_string(),
                    },
                    filters: vec!["opaque_hex".to_string()],
                })],
            }
        );
    }

    #[test]
    fn parses_placeholder_with_multiple_filters() {
        let ast = parse_template("{{ token.comment | hex | upper }}").unwrap();

        assert_eq!(
            ast,
            TemplateDocument {
                segments: vec![TemplateSegment::Placeholder(TemplatePlaceholder {
                    path: TemplatePath {
                        namespace: "token".to_string(),
                        key: "comment".to_string(),
                    },
                    filters: vec!["hex".to_string(), "upper".to_string()],
                })],
            }
        );
    }

    #[test]
    fn parses_mixed_content_segments() {
        let ast = parse_template("prefix {{token.comment}} suffix").unwrap();

        assert_eq!(
            ast,
            TemplateDocument {
                segments: vec![
                    TemplateSegment::Text("prefix ".to_string()),
                    TemplateSegment::Placeholder(TemplatePlaceholder {
                        path: TemplatePath {
                            namespace: "token".to_string(),
                            key: "comment".to_string(),
                        },
                        filters: vec![],
                    }),
                    TemplateSegment::Text(" suffix".to_string()),
                ],
            }
        );
    }

    #[test]
    fn treats_non_engine_double_braces_as_literal_text() {
        let ast = parse_template("prefix {{literal braces}} suffix").unwrap();

        assert_eq!(
            ast,
            TemplateDocument {
                segments: vec![TemplateSegment::Text(
                    "prefix {{literal braces}} suffix".to_string()
                )],
            }
        );
    }

    #[test]
    fn rejects_malformed_placeholder() {
        let error = parse_template("{{token}}").unwrap_err();

        assert!(matches!(error, TemplateParseError::MalformedPlaceholder(_)));
    }

    #[test]
    fn rejects_unclosed_placeholder() {
        let error = parse_template("{{token.comment").unwrap_err();

        assert!(matches!(
            error,
            TemplateParseError::UnclosedPlaceholder { .. }
        ));
    }
}
