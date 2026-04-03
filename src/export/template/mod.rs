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

    #[test]
    fn template_exporter_uses_export_context() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(
            b"project={{meta.project_name}}\nprofile={{meta.profile_name}}\nformat={{meta.profile_format}}\noutput={{meta.output_path}}\nexporter={{meta.exporter}}\nexporter_key={{meta.exporter_key}}\nbackground={{token.background}}\npalette={{palette.bg_0}}\ncontrast={{param.contrast}}\n",
        )
        .unwrap();
        file.flush().unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = ExportContext::builder(
            "Demo Project",
            &crate::export::ExportProfile::template_default(),
            &theme,
            &params,
        )
        .build()
        .unwrap();
        let output = exporter.export_with_context(&context).unwrap();

        assert!(output.contains("project=Demo Project"));
        assert!(output.contains("profile=Template"));
        assert!(output.contains("format=template"));
        assert!(output.contains(&format!(
            "output={}",
            crate::export::ExportProfile::template_default()
                .output_path
                .display()
        )));
        assert!(output.contains("exporter=Template"));
        assert!(output.contains("exporter_key=template"));
        assert!(output.contains("background=#"));
        assert!(output.contains("palette=#"));
        assert!(output.contains("contrast=0.85"));
    }

    #[test]
    fn template_exporter_rejects_unknown_placeholders() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(
            b"project={{meta.project_name}}\nprofile={{meta.profile_name}}\nformat={{meta.profile_format}}\noutput={{meta.output_path}}\nmissing={{meta.missing}}\n",
        )
        .unwrap();
        file.flush().unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = ExportContext::builder(
            "Demo Project",
            &crate::export::ExportProfile::template_default(),
            &theme,
            &params,
        )
        .build()
        .unwrap();
        let error = exporter.export_with_context(&context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(_)
        ));
    }

    #[test]
    fn template_exporter_rejects_malformed_non_namespaced_placeholder() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"note={{literal braces}}\nproject={{meta.project_name}}\n")
            .unwrap();
        file.flush().unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = ExportContext::builder(
            "Demo Project",
            &crate::export::ExportProfile::template_default(),
            &theme,
            &params,
        )
        .build()
        .unwrap();
        let error = exporter.export_with_context(&context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(_)
        ));
    }

    #[test]
    fn template_exporter_rejects_malformed_placeholders_like_parser() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"project={{token}}\n").unwrap();
        file.flush().unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let exporter = TemplateExporter::from_path(file.path()).unwrap();
        let context = ExportContext::builder(
            "Project with {{braces}}",
            &crate::export::ExportProfile::template_default(),
            &theme,
            &params,
        )
        .build()
        .unwrap();
        let error = exporter.export_with_context(&context).unwrap_err();

        assert!(matches!(
            error,
            crate::export::ExportError::InvalidTemplate(_)
        ));
    }
}
