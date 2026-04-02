pub mod alacritty;
pub mod context;
pub mod template;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::params::ThemeParams;
use crate::evaluator::ResolvedTheme;
use crate::export::alacritty::AlacrittyExporter;
use crate::export::context::ExportContext;
use crate::export::template::TemplateExporter;

#[allow(dead_code)]
pub trait Exporter {
    fn name(&self) -> &'static str;
    fn export(&self, theme: &ResolvedTheme) -> Result<String, ExportError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfile {
    pub name: String,
    pub enabled: bool,
    pub output_path: PathBuf,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExportFormat {
    Alacritty,
    Template { template_path: PathBuf },
}

impl ExportFormat {
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Alacritty => "alacritty",
            Self::Template { .. } => "template",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportArtifact {
    pub profile_name: String,
    pub output_path: PathBuf,
}

impl Default for ExportProfile {
    fn default() -> Self {
        Self::alacritty_default()
    }
}

impl ExportProfile {
    pub fn alacritty_default() -> Self {
        Self {
            name: "Alacritty".to_string(),
            enabled: true,
            output_path: PathBuf::from("exports/alacritty-theme.toml"),
            format: ExportFormat::Alacritty,
        }
    }

    pub fn template_default() -> Self {
        Self {
            name: "Template".to_string(),
            enabled: false,
            output_path: PathBuf::from("exports/theme-template.txt"),
            format: ExportFormat::Template {
                template_path: PathBuf::from("templates/generic-theme.txt"),
            },
        }
    }

    pub fn format_label(&self) -> &'static str {
        match self.format {
            ExportFormat::Alacritty => "Alacritty",
            ExportFormat::Template { .. } => "Template",
        }
    }

    pub const fn format_key(&self) -> &'static str {
        self.format.key()
    }

    pub fn summary_label(&self) -> String {
        let marker = if self.enabled { "[x]" } else { "[ ]" };
        format!("{marker} {} ({})", self.name, self.format_label())
    }
}

pub fn default_export_profiles() -> Vec<ExportProfile> {
    vec![
        ExportProfile::alacritty_default(),
        ExportProfile::template_default(),
    ]
}

pub fn export_with_profile(
    profile: &ExportProfile,
    project_name: &str,
    theme: &ResolvedTheme,
    params: &ThemeParams,
) -> Result<String, ExportError> {
    match &profile.format {
        ExportFormat::Alacritty => AlacrittyExporter.export(theme),
        ExportFormat::Template { template_path } => {
            let context = ExportContext::builder(project_name, profile, theme, params).build()?;
            TemplateExporter::from_path(template_path)?.export_with_context(&context)
        }
    }
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ExportError {
    #[error("missing export context value {0}")]
    MissingExportContextValue(String),
    #[error("missing token {0}")]
    MissingToken(String),
    #[error("{0}")]
    SerializeError(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    InvalidTemplate(String),
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::domain::palette::generate_palette;
    use crate::domain::params::ThemeParams;
    use crate::domain::rules::RuleSet;
    use crate::evaluator::resolve_theme;

    use super::{ExportFormat, ExportProfile, export_with_profile};
    use tempfile::NamedTempFile;

    #[test]
    fn export_with_profile_uses_export_context_for_template_profiles() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(
            b"project={{meta.project_name}}\nprofile={{meta.profile_name}}\nformat={{meta.profile_format}}\noutput={{meta.output_path}}\nbackground={{token.background}}\npalette={{palette.bg_0}}\ncontrast={{param.contrast}}\n",
        )
        .unwrap();
        file.flush().unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = ExportProfile {
            name: "Context Test".to_string(),
            enabled: true,
            output_path: std::path::PathBuf::from("exports/context-test.txt"),
            format: ExportFormat::Template {
                template_path: file.path().to_path_buf(),
            },
        };
        let output = export_with_profile(&profile, "Demo Project", &theme, &params).unwrap();

        assert!(output.contains("project=Demo Project"));
        assert!(output.contains("profile=Context Test"));
        assert!(output.contains("format=template"));
        assert!(output.contains(&format!("output={}", profile.output_path.display())));
        assert!(output.contains("background=#"));
        assert!(output.contains("palette=#"));
        assert!(output.contains("contrast=0.85"));
    }

    #[test]
    fn export_with_profile_still_exports_alacritty_profiles() {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = ExportProfile::alacritty_default();

        let output = export_with_profile(&profile, "Ignored Project", &theme, &params).unwrap();

        assert!(output.contains("[colors.primary]"));
        assert!(output.contains("background = \""));
    }
}
