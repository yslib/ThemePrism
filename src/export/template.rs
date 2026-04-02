use std::fs;
use std::path::Path;

use crate::export::context::ExportContext;
use crate::export::ExportError;

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
    let mut rendered = String::with_capacity(template.len());
    let mut cursor = 0;

    while let Some(start) = template[cursor..].find("{{") {
        let start = cursor + start;
        rendered.push_str(&template[cursor..start]);

        let placeholder_start = start + 2;
        let Some(end) = template[placeholder_start..].find("}}") else {
            return Err(ExportError::InvalidTemplate(format!(
                "unknown template placeholder {}",
                &template[start..]
            )));
        };
        let end = placeholder_start + end;
        let raw = template[placeholder_start..end].trim();
        let replacement = match resolve_placeholder(raw, context) {
            Some(value) => value,
            None => {
                return Err(ExportError::InvalidTemplate(format!(
                    "unknown template placeholder {{{{{raw}}}}}"
                )));
            }
        };
        rendered.push_str(&replacement);
        cursor = end + 2;
    }

    rendered.push_str(&template[cursor..]);
    Ok(rendered)
}

fn resolve_placeholder(raw: &str, context: &ExportContext) -> Option<String> {
    match raw.split_once('.') {
        Some(("meta", key)) => match key {
            "project_name" => Some(context.meta.project_name.clone()),
            "profile_name" => Some(context.meta.profile_name.clone()),
            "profile_format" => Some(context.meta.profile_format.clone()),
            "output_path" => Some(context.meta.output_path.clone()),
            "exporter" => Some(context.meta.exporter.clone()),
            _ => None,
        },
        Some(("token", key)) => context.token.get(key).map(|value| value.render_text()),
        Some(("palette", key)) => context.palette.get(key).map(|value| value.render_text()),
        Some(("param", key)) => context.param.get(key).map(|value| value.render_text()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::domain::params::ThemeParams;
    use crate::domain::palette::generate_palette;
    use crate::domain::rules::RuleSet;
    use crate::evaluator::resolve_theme;
    use crate::export::context::ExportContext;
    use crate::export::template::TemplateExporter;
    use tempfile::NamedTempFile;

    #[test]
    fn template_exporter_uses_export_context() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(
            b"project={{meta.project_name}}\nprofile={{meta.profile_name}}\nformat={{meta.profile_format}}\noutput={{meta.output_path}}\nbackground={{token.background}}\npalette={{palette.bg_0}}\ncontrast={{param.contrast}}\n",
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
        assert!(output.contains("output=exports/theme-template.txt"));
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

        assert!(matches!(error, crate::export::ExportError::InvalidTemplate(_)));
    }

    #[test]
    fn template_exporter_preserves_braces_inside_metadata_values() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"project={{meta.project_name}}\n").unwrap();
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
        let output = exporter.export_with_context(&context).unwrap();

        assert!(output.contains("project=Project with {{braces}}"));
    }
}
