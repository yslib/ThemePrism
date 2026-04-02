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

pub fn parse_template(input: &str) -> Result<TemplateDocument, TemplateParseError> {
    let mut segments = Vec::new();
    let mut cursor = 0;

    while let Some(open_offset) = input[cursor..].find("{{") {
        let open = cursor + open_offset;
        if open > cursor {
            segments.push(TemplateSegment::Text(input[cursor..open].to_string()));
        }

        let inner_start = open + 2;
        let Some(close_offset) = input[inner_start..].find("}}") else {
            return Err(TemplateParseError::UnclosedPlaceholder { start: open });
        };
        let close = inner_start + close_offset;
        let raw = input[inner_start..close].trim();
        let placeholder = parse_placeholder(raw)?;
        segments.push(TemplateSegment::Placeholder(placeholder));
        cursor = close + 2;
    }

    if cursor < input.len() {
        segments.push(TemplateSegment::Text(input[cursor..].to_string()));
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
