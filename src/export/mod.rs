pub mod alacritty;
pub mod template;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evaluator::ResolvedTheme;
use crate::export::alacritty::AlacrittyExporter;
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
    theme: &ResolvedTheme,
) -> Result<String, ExportError> {
    match &profile.format {
        ExportFormat::Alacritty => AlacrittyExporter.export(theme),
        ExportFormat::Template { template_path } => {
            TemplateExporter::from_path(&profile.name, template_path)?.export(theme)
        }
    }
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ExportError {
    #[error("missing token {0}")]
    MissingToken(String),
    #[error("{0}")]
    SerializeError(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    InvalidTemplate(String),
}
