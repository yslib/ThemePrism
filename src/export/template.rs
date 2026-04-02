use std::fs;
use std::path::Path;

use crate::export::context::ExportContext;
use crate::evaluator::ResolvedTheme;
use crate::export::{ExportError, Exporter};
use crate::tokens::TokenRole;

#[derive(Debug, Clone)]
pub struct TemplateExporter {
    profile_name: String,
    template: String,
}

impl TemplateExporter {
    pub fn from_path(profile_name: &str, path: &Path) -> Result<Self, ExportError> {
        let template = fs::read_to_string(path).map_err(|err| {
            ExportError::Io(format!("failed to read template {}: {err}", path.display()))
        })?;
        Ok(Self {
            profile_name: profile_name.to_string(),
            template,
        })
    }

    pub fn export_with_context(&self, context: &ExportContext) -> Result<String, ExportError> {
        let mut rendered = self.template.clone();
        rendered = rendered.replace("{{meta.project_name}}", &context.meta.project_name);
        rendered = rendered.replace("{{meta.profile_name}}", &context.meta.profile_name);
        rendered = rendered.replace("{{meta.profile_format}}", &context.meta.profile_format);
        rendered = rendered.replace("{{meta.output_path}}", &context.meta.output_path);
        rendered = rendered.replace("{{meta.exporter}}", self.name());

        for (key, value) in &context.token {
            rendered = rendered.replace(&format!("{{{{token.{key}}}}}"), &value.render_text());
        }

        for (key, value) in &context.palette {
            rendered = rendered.replace(&format!("{{{{palette.{key}}}}}"), &value.render_text());
        }

        for (key, value) in &context.param {
            rendered = rendered.replace(&format!("{{{{param.{key}}}}}"), &value.render_text());
        }

        if let Some(start) = rendered.find("{{token.") {
            let tail = &rendered[start..];
            let end = tail
                .find("}}")
                .map(|index| start + index + 2)
                .unwrap_or(rendered.len());
            let unknown = &rendered[start..end];
            return Err(ExportError::InvalidTemplate(format!(
                "unknown template placeholder {unknown}"
            )));
        }

        Ok(rendered)
    }
}

impl Exporter for TemplateExporter {
    fn name(&self) -> &'static str {
        "Template"
    }

    fn export(&self, theme: &ResolvedTheme) -> Result<String, ExportError> {
        let mut rendered = self.template.clone();
        rendered = rendered.replace("{{meta.profile_name}}", &self.profile_name);
        rendered = rendered.replace("{{meta.exporter}}", self.name());

        for role in TokenRole::ALL {
            let color = theme
                .token(role)
                .map(|value| value.to_hex())
                .ok_or_else(|| ExportError::MissingToken(role.label().to_string()))?;
            let key = format!("{{{{token.{}}}}}", role.key());
            rendered = rendered.replace(&key, &color);
        }

        if let Some(start) = rendered.find("{{token.") {
            let tail = &rendered[start..];
            let end = tail
                .find("}}")
                .map(|index| start + index + 2)
                .unwrap_or(rendered.len());
            let unknown = &rendered[start..end];
            return Err(ExportError::InvalidTemplate(format!(
                "unknown template placeholder {unknown}"
            )));
        }

        Ok(rendered)
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
        let exporter = TemplateExporter::from_path("Test Profile", file.path()).unwrap();
        let context = ExportContext::builder(
            "Demo Project",
            &crate::export::ExportProfile::template_default(),
            &theme,
            &params,
        )
        .build();
        let output = exporter.export_with_context(&context).unwrap();

        assert!(output.contains("project=Demo Project"));
        assert!(output.contains("profile=Template"));
        assert!(output.contains("format=template"));
        assert!(output.contains("output=exports/theme-template.txt"));
        assert!(output.contains("background=#"));
        assert!(output.contains("palette=#"));
        assert!(output.contains("contrast=0.85"));
    }
}
