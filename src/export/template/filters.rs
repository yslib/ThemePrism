use crate::export::ExportError;
use crate::export::context::ExportValue;

pub(crate) fn apply_filter(value: ExportValue, filter: &str) -> Result<ExportValue, ExportError> {
    match filter {
        "hex" => match value {
            ExportValue::Color(color) => Ok(ExportValue::Text(color.to_hex())),
            other => Err(invalid_filter_type(filter, &other)),
        },
        "opaque_hex" => match value {
            ExportValue::Color(color) => Ok(ExportValue::Text(color.to_opaque_hex())),
            other => Err(invalid_filter_type(filter, &other)),
        },
        "rgb" => match value {
            ExportValue::Color(color) => {
                let (r, g, b) = color.to_rgb_u8();
                Ok(ExportValue::Text(format!("rgb({r}, {g}, {b})")))
            }
            other => Err(invalid_filter_type(filter, &other)),
        },
        "rgba" => match value {
            ExportValue::Color(color) => {
                let (r, g, b) = color.to_rgb_u8();
                Ok(ExportValue::Text(format!(
                    "rgba({r}, {g}, {b}, {})",
                    format_number(color.a)
                )))
            }
            other => Err(invalid_filter_type(filter, &other)),
        },
        "alpha" => match value {
            ExportValue::Color(color) => Ok(ExportValue::Number(color.a)),
            other => Err(invalid_filter_type(filter, &other)),
        },
        "float" => match value {
            ExportValue::Number(number) => Ok(ExportValue::Text(format_number(number))),
            other => Err(invalid_filter_type(filter, &other)),
        },
        "percent" => match value {
            ExportValue::Number(number) => Ok(ExportValue::Text(format!(
                "{}%",
                format_number(number * 100.0)
            ))),
            other => Err(invalid_filter_type(filter, &other)),
        },
        "lower" => match value {
            ExportValue::Text(text) => Ok(ExportValue::Text(text.to_lowercase())),
            other => Err(invalid_filter_type(filter, &other)),
        },
        "upper" => match value {
            ExportValue::Text(text) => Ok(ExportValue::Text(text.to_uppercase())),
            other => Err(invalid_filter_type(filter, &other)),
        },
        _ => Err(ExportError::InvalidTemplate(format!(
            "unknown template filter {filter}"
        ))),
    }
}

fn invalid_filter_type(filter: &str, value: &ExportValue) -> ExportError {
    ExportError::InvalidTemplate(format!(
        "filter {filter} does not support {}",
        value_kind(value)
    ))
}

fn value_kind(value: &ExportValue) -> &'static str {
    match value {
        ExportValue::Color(_) => "color values",
        ExportValue::Number(_) => "number values",
        ExportValue::Text(_) => "text values",
    }
}

pub(crate) fn format_number(value: f32) -> String {
    let mut text = format!("{value:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" { "0".to_string() } else { text }
}

#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::export::context::ExportValue;

    #[test]
    fn applies_supported_color_filters() {
        let value = ExportValue::Color(Color::new_rgba(
            0x12 as f32 / 255.0,
            0x34 as f32 / 255.0,
            0x56 as f32 / 255.0,
            0.5,
        ));

        assert_eq!(
            super::apply_filter(value.clone(), "hex")
                .unwrap()
                .render_text(),
            "#12345680"
        );
        assert_eq!(
            super::apply_filter(value.clone(), "opaque_hex")
                .unwrap()
                .render_text(),
            "#123456"
        );
        assert_eq!(
            super::apply_filter(value.clone(), "rgb")
                .unwrap()
                .render_text(),
            "rgb(18, 52, 86)"
        );
        assert_eq!(
            super::apply_filter(value.clone(), "rgba")
                .unwrap()
                .render_text(),
            "rgba(18, 52, 86, 0.5)"
        );
        assert!(matches!(
            super::apply_filter(value, "alpha").unwrap(),
            ExportValue::Number(number) if (number - 0.5).abs() < 0.001
        ));
    }

    #[test]
    fn alpha_filter_returns_number_for_chaining() {
        let value = ExportValue::Color(Color::from_rgba_u8(0x12, 0x34, 0x56, 0x80));

        assert!(matches!(
            super::apply_filter(value, "alpha").unwrap(),
            ExportValue::Number(number) if (number - (128.0 / 255.0)).abs() < 0.000_01
        ));
    }

    #[test]
    fn applies_supported_number_filters() {
        let value = ExportValue::Number(0.85);

        assert_eq!(
            super::apply_filter(value.clone(), "float")
                .unwrap()
                .render_text(),
            "0.85"
        );
        assert_eq!(
            super::apply_filter(value, "percent").unwrap().render_text(),
            "85%"
        );
    }

    #[test]
    fn applies_supported_text_filters() {
        let value = ExportValue::Text("Template".to_string());

        assert_eq!(
            super::apply_filter(value.clone(), "lower")
                .unwrap()
                .render_text(),
            "template"
        );
        assert_eq!(
            super::apply_filter(value, "upper").unwrap().render_text(),
            "TEMPLATE"
        );
    }

    #[test]
    fn returns_error_for_unknown_filter() {
        let error =
            super::apply_filter(ExportValue::Text("Template".to_string()), "mystery").unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("unknown template filter mystery")
        ));
    }

    #[test]
    fn returns_error_for_invalid_filter_type() {
        let error =
            super::apply_filter(ExportValue::Text("Template".to_string()), "percent").unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("filter percent does not support text values")
        ));
    }
}
