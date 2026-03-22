use crate::domain::color::Color;

use super::{SpanStyle, StyledLine, StyledSpan};

pub(crate) fn line_pair(
    prefix: &str,
    value: &str,
    prefix_color: Color,
    value_color: Color,
    swatch: Option<Color>,
    bold: bool,
) -> StyledLine {
    let mut spans = vec![colored_span(prefix, prefix_color, false, false)];
    if let Some(color) = swatch {
        spans.push(swatch_span(color, 4));
        spans.push(plain_span(" "));
    }
    spans.push(colored_span(value.to_string(), value_color, bold, false));
    StyledLine { spans }
}

pub(crate) fn colored_span(
    text: impl Into<String>,
    fg: Color,
    bold: bool,
    italic: bool,
) -> StyledSpan {
    StyledSpan {
        text: text.into(),
        style: SpanStyle {
            fg: Some(fg),
            bg: None,
            bold,
            italic,
        },
    }
}

pub(crate) fn plain_span(text: &str) -> StyledSpan {
    StyledSpan {
        text: text.to_string(),
        style: SpanStyle::default(),
    }
}

pub(crate) fn swatch_span(color: Color, width: usize) -> StyledSpan {
    StyledSpan {
        text: " ".repeat(width),
        style: SpanStyle {
            fg: None,
            bg: Some(color),
            bold: false,
            italic: false,
        },
    }
}
