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

    use crate::domain::palette::generate_palette;
    use crate::domain::params::ThemeParams;
    use crate::domain::rules::RuleSet;
    use crate::evaluator::resolve_theme;
    use crate::export::context::ExportContext;
    use crate::export::template::TemplateExporter;
    use tempfile::NamedTempFile;

    fn build_context() -> ExportContext {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = crate::export::ExportProfile::template_default();

        ExportContext::builder("Demo Project", &profile, &theme, &params)
            .build()
            .unwrap()
    }

    fn write_template(template: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(template.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn template_exporter_renders_profile_metadata() {
        let file = write_template(
            "project={{meta.project_name}}\nprofile={{meta.profile_name}}\nformat={{meta.profile_format}}\noutput={{meta.output_path}}\nexporter={{meta.exporter}}\nexporter_key={{meta.exporter_key}}\n",
        );
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let output = exporter.export_with_context(&context).unwrap();

        assert_eq!(
            output,
            "project=Demo Project\nprofile=Template\nformat=template\noutput=exports/theme-template.txt\nexporter=Template\nexporter_key=template\n"
        );
    }

    #[test]
    fn template_exporter_renders_token_placeholder_output() {
        let file = write_template("token={{token.background}}\n");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let output = exporter.export_with_context(&context).unwrap();

        let expected = match context.token.get("background").unwrap() {
            crate::export::context::ExportValue::Color(color) => color.to_hex(),
            other => other.render_text(),
        };

        assert_eq!(output, format!("token={expected}\n"));
    }

    #[test]
    fn template_exporter_renders_palette_and_param_placeholder_output() {
        let file = write_template("palette={{palette.bg_0}}\ncontrast={{param.contrast}}\n");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let output = exporter.export_with_context(&context).unwrap();

        let expected_palette = context
            .palette
            .get("bg_0")
            .expect("expected bg_0 palette slot");
        let expected_contrast = context
            .param
            .get("contrast")
            .expect("expected contrast param");

        assert_eq!(
            output,
            format!(
                "palette={}\ncontrast={}\n",
                expected_palette.render_text(),
                expected_contrast.render_text()
            )
        );
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
        let file = write_template("background={{token.background | opaque_hex}}");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let output = exporter.export_with_context(&context).unwrap();

        let expected = match context.token.get("background").unwrap() {
            crate::export::context::ExportValue::Color(color) => color.to_opaque_hex(),
            other => other.render_text(),
        };

        assert_eq!(output, format!("background={expected}"));
    }

    #[test]
    fn template_exporter_preserves_literal_double_braces_text() {
        let file = write_template("literal={{literal braces}}\nproject={{meta.project_name}}\n");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let output = exporter.export_with_context(&context).unwrap();

        assert_eq!(output, "literal={{literal braces}}\nproject=Demo Project\n");
    }

    #[test]
    fn template_exporter_still_rejects_malformed_known_namespace_placeholders() {
        let file = write_template("broken={{token}}\n");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let error = exporter.export_with_context(&context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("malformed template placeholder token")
        ));
    }

    #[test]
    fn template_exporter_propagates_evaluator_errors_from_the_file_backed_boundary() {
        let file = write_template("project={{token.background | mystery}}\n");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let error = exporter.export_with_context(&context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("unknown template filter mystery")
        ));
    }

    #[test]
    fn template_exporter_propagates_parse_errors_from_the_file_backed_boundary() {
        let file = write_template("project={{token.background\n");
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = build_context();

        let error = exporter.export_with_context(&context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(message)
                if message.contains("unclosed template placeholder")
        ));
    }
}
