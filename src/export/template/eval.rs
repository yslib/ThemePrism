use crate::export::ExportError;
use crate::export::context::{ExportContext, ExportValue};
use crate::export::template::filters::{apply_filter, format_number};
use crate::export::template::parser::{TemplateDocument, TemplateSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberRenderStyle {
    Default,
    Canonical,
}

#[derive(Debug, Clone)]
struct EvaluatedValue {
    value: ExportValue,
    number_render_style: NumberRenderStyle,
}

impl EvaluatedValue {
    fn new(value: ExportValue) -> Self {
        Self {
            value,
            number_render_style: NumberRenderStyle::Default,
        }
    }

    fn apply_filter(self, filter: &str) -> Result<Self, ExportError> {
        let value = apply_filter(self.value, filter)?;
        let number_render_style = match (filter, &value) {
            ("alpha", ExportValue::Number(_)) => NumberRenderStyle::Canonical,
            _ => NumberRenderStyle::Default,
        };

        Ok(Self {
            value,
            number_render_style,
        })
    }

    fn render(&self) -> String {
        match (&self.value, self.number_render_style) {
            (ExportValue::Number(number), NumberRenderStyle::Canonical) => format_number(*number),
            _ => self.value.render_text(),
        }
    }
}

pub(crate) fn render_document(
    document: &TemplateDocument,
    context: &ExportContext,
) -> Result<String, ExportError> {
    let mut rendered = String::new();

    for segment in &document.segments {
        match segment {
            TemplateSegment::Text(text) => rendered.push_str(text),
            TemplateSegment::Placeholder(placeholder) => {
                let mut value =
                    resolve_path(&placeholder.path.namespace, &placeholder.path.key, context)?;
                for filter in &placeholder.filters {
                    value = value.apply_filter(filter)?;
                }
                rendered.push_str(&value.render());
            }
        }
    }

    Ok(rendered)
}

fn resolve_path(
    namespace: &str,
    key: &str,
    context: &ExportContext,
) -> Result<EvaluatedValue, ExportError> {
    match namespace {
        "meta" => resolve_meta(key, context),
        "token" => context.token.get(key).cloned(),
        "palette" => context.palette.get(key).cloned(),
        "param" => context.param.get(key).cloned(),
        _ => None,
    }
    .ok_or_else(|| {
        ExportError::InvalidTemplate(format!("unknown template placeholder {namespace}.{key}"))
    })
    .map(EvaluatedValue::new)
}

fn resolve_meta(key: &str, context: &ExportContext) -> Option<ExportValue> {
    match key {
        "project_name" => Some(ExportValue::Text(context.meta.project_name.clone())),
        "profile_name" => Some(ExportValue::Text(context.meta.profile_name.clone())),
        "profile_format" => Some(ExportValue::Text(context.meta.profile_format.clone())),
        "output_path" => Some(ExportValue::Text(context.meta.output_path.clone())),
        "exporter" => Some(ExportValue::Text(context.meta.exporter.clone())),
        "exporter_key" => Some(ExportValue::Text(context.meta.exporter_key.clone())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::domain::palette::generate_palette;
    use crate::domain::params::ThemeParams;
    use crate::domain::rules::RuleSet;
    use crate::evaluator::resolve_theme;
    use crate::export::context::{ExportContext, ExportValue};
    use crate::export::template::parser::parse_template;

    fn build_context() -> ExportContext {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = crate::export::ExportProfile::template_default();
        let mut context = ExportContext::builder("Demo Project", &profile, &theme, &params)
            .build()
            .unwrap();
        context.token.insert(
            "comment".to_string(),
            ExportValue::Color(Color::new_rgba(
                0x12 as f32 / 255.0,
                0x34 as f32 / 255.0,
                0x56 as f32 / 255.0,
                0.5,
            )),
        );
        context.token.insert(
            "warning".to_string(),
            ExportValue::Color(Color::from_rgba_u8(0x12, 0x34, 0x56, 0x80)),
        );
        context
            .param
            .insert("contrast".to_string(), ExportValue::Number(0.123_456));
        context
    }

    fn render(
        template: &str,
        context: &ExportContext,
    ) -> Result<String, crate::export::ExportError> {
        let document = parse_template(template).unwrap();
        super::render_document(&document, context)
    }

    #[test]
    fn renders_token_color_as_default_hex_text() {
        let context = build_context();

        let rendered = render("{{token.comment}}", &context).unwrap();

        assert_eq!(rendered, "#12345680");
    }

    #[test]
    fn renders_token_color_with_opaque_hex_filter() {
        let context = build_context();

        let rendered = render("{{token.comment | opaque_hex}}", &context).unwrap();

        assert_eq!(rendered, "#123456");
    }

    #[test]
    fn renders_token_color_with_rgba_filter() {
        let context = build_context();

        let rendered = render("{{token.comment | rgba}}", &context).unwrap();

        assert_eq!(rendered, "rgba(18, 52, 86, 0.5)");
    }

    #[test]
    fn renders_alpha_filter_with_canonical_formatting() {
        let context = build_context();

        let rendered = render("{{token.warning | alpha}}", &context).unwrap();

        assert_eq!(rendered, "0.502");
    }

    #[test]
    fn renders_chained_alpha_and_percent_filters_with_canonical_formatting() {
        let context = build_context();

        let rendered = render("{{token.warning | alpha | percent}}", &context).unwrap();

        assert_eq!(rendered, "50.196%");
    }

    #[test]
    fn renders_param_number_with_percent_filter() {
        let context = build_context();

        let rendered = render("{{param.contrast | percent}}", &context).unwrap();

        assert_eq!(rendered, "12.346%");
    }

    #[test]
    fn preserves_default_numeric_rendering_for_raw_param_placeholders() {
        let context = build_context();

        let rendered = render("{{param.contrast}}", &context).unwrap();

        assert_eq!(rendered, "0.123456");
    }

    #[test]
    fn renders_meta_text_with_upper_filter() {
        let context = build_context();

        let rendered = render("{{meta.profile_name | upper}}", &context).unwrap();

        assert_eq!(rendered, "TEMPLATE");
    }

    #[test]
    fn returns_error_for_unknown_namespace_or_key() {
        let context = build_context();

        let error = render("{{unknown.comment}}", &context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("unknown template placeholder unknown.comment")
        ));
    }

    #[test]
    fn returns_error_for_unknown_filter() {
        let context = build_context();

        let error = render("{{token.comment | mystery}}", &context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("unknown template filter mystery")
        ));
    }

    #[test]
    fn returns_error_for_invalid_filter_type() {
        let context = build_context();

        let error = render("{{meta.profile_name | percent}}", &context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("filter percent does not support text values")
        ));
    }
}
