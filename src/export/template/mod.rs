pub mod eval;
pub mod filters;
pub mod parser;

use std::fs;
use std::path::Path;

use crate::export::ExportError;
use crate::export::context::ExportContext;
use parser::parse_template;

#[derive(Debug, Clone)]
pub struct TemplateExporter {
    template: String,
}

impl TemplateExporter {
    pub fn from_path(path: &Path) -> Result<Self, ExportError> {
        let template = fs::read_to_string(path).map_err(|err| {
            ExportError::Io(format!("failed to read template {}: {err}", path.display()))
        })?;
        Ok(Self { template })
    }

    pub fn export_with_context(&self, context: &ExportContext) -> Result<String, ExportError> {
        render_template(&self.template, context)
    }
}

fn render_template(template: &str, context: &ExportContext) -> Result<String, ExportError> {
    let document = parse_template(template).map_err(|error| {
        ExportError::InvalidTemplate(match error {
            parser::TemplateParseError::MalformedPlaceholder(raw) => {
                format!("malformed template placeholder {raw}")
            }
            parser::TemplateParseError::UnclosedPlaceholder { start } => {
                format!("unclosed template placeholder at byte {start}")
            }
        })
    })?;

    eval::render_document(&document, context)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::color::Color;
    use crate::domain::palette::generate_palette;
    use crate::domain::params::ThemeParams;
    use crate::domain::rules::RuleSet;
    use crate::evaluator::resolve_theme;
    use crate::export::context::{ExportContext, ExportValue};
    use crate::export::template::TemplateExporter;
    use tempfile::NamedTempFile;

    fn build_context() -> ExportContext {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = crate::export::ExportProfile::template_default();

        let mut context =
            ExportContext::builder("Demo Project", &profile, &theme, &params)
                .build()
                .unwrap();
        context.token.insert(
            "comment".to_string(),
            ExportValue::Color(Color::from_rgba_u8(0x12, 0x34, 0x56, 0x80)),
        );
        context
    }

    fn write_template(template: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(template.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn template_exporter_renders_mixed_text_and_placeholder_output() {
        let file = write_template("prefix {{meta.project_name}} suffix");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let output = exporter.export_with_context(&context).unwrap();

        assert_eq!(output, "prefix Demo Project suffix");
    }

    #[test]
    fn template_exporter_renders_filtered_placeholder_output() {
        let file = write_template("comment={{token.comment | opaque_hex}}");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let output = exporter.export_with_context(&context).unwrap();

        assert_eq!(output, "comment=#123456");
    }

    #[test]
    fn template_exporter_propagates_parser_errors_precisely() {
        let file = write_template("project={{token}}\n");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let error = exporter.export_with_context(&context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message == "malformed template placeholder token"
        ));
    }
}
