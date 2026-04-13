pub mod alacritty;
pub mod context;
pub mod managed_block;
pub mod template;

use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::params::ThemeParams;
use crate::evaluator::ResolvedTheme;
use crate::export::alacritty::{
    BUNDLED_TEMPLATE_PATH, bundled_template_path, default_alacritty_output_path,
    generic_template_path, resolve_bundled_template_path,
};
use crate::export::context::ExportContext;
use crate::export::template::TemplateExporter;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfile {
    pub name: String,
    pub enabled: bool,
    pub output_path: PathBuf,
    #[serde(default, skip_serializing_if = "ExportWriteMode::is_replace_file")]
    pub write_mode: ExportWriteMode,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExportWriteMode {
    #[default]
    ReplaceFile,
    ManagedBlock,
}

impl ExportWriteMode {
    pub const fn is_replace_file(&self) -> bool {
        matches!(self, Self::ReplaceFile)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExportFormat {
    Alacritty,
    Template { template_path: PathBuf },
}

impl ExportFormat {
    pub fn normalize(self) -> Self {
        match self {
            Self::Alacritty => Self::Template {
                template_path: bundled_template_path(),
            },
            template => template,
        }
    }

    pub const fn key(&self) -> &'static str {
        match self {
            Self::Alacritty => "alacritty",
            Self::Template { .. } => "template",
        }
    }

    pub fn configured_template_path(&self) -> &Path {
        match self {
            Self::Alacritty => Path::new(BUNDLED_TEMPLATE_PATH),
            Self::Template { template_path } => template_path.as_path(),
        }
    }

    pub fn resolved_template_path(&self) -> PathBuf {
        match self {
            Self::Alacritty => resolve_bundled_template_path(Path::new(BUNDLED_TEMPLATE_PATH))
                .expect("missing bundled Alacritty template path mapping"),
            Self::Template { template_path } => resolve_bundled_template_path(template_path)
                .unwrap_or_else(|| template_path.clone()),
        }
    }

    pub fn set_template_path(&mut self, template_path: PathBuf) {
        *self = Self::Template { template_path };
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
            output_path: default_alacritty_output_path(),
            write_mode: ExportWriteMode::ManagedBlock,
            format: ExportFormat::Template {
                template_path: bundled_template_path(),
            },
        }
    }

    pub fn template_default() -> Self {
        Self {
            name: "Template".to_string(),
            enabled: false,
            output_path: PathBuf::from("exports/theme-template.txt"),
            write_mode: ExportWriteMode::ReplaceFile,
            format: ExportFormat::Template {
                template_path: generic_template_path(),
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

    pub fn normalize(&self) -> Self {
        Self {
            name: self.name.clone(),
            enabled: self.enabled,
            output_path: self.output_path.clone(),
            write_mode: self.write_mode,
            format: self.format.clone().normalize(),
        }
    }

    pub fn configured_template_path(&self) -> &Path {
        self.format.configured_template_path()
    }

    pub fn resolved_template_path(&self) -> PathBuf {
        self.format.resolved_template_path()
    }

    pub fn set_template_path(&mut self, template_path: PathBuf) {
        self.format.set_template_path(template_path);
    }

    pub fn summary_label(&self) -> String {
        let marker = if self.enabled { "[x]" } else { "[ ]" };
        format!("{marker} {} ({})", self.name, self.format_label())
    }
}

pub fn resolve_output_path(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return BaseDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .unwrap_or_else(|| path.to_path_buf());
    }

    if let Some(relative) = raw.strip_prefix("~/") {
        return BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(relative))
            .unwrap_or_else(|| path.to_path_buf());
    }

    path.to_path_buf()
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
    let profile = profile.normalize();
    let context = ExportContext::builder(project_name, &profile, theme, params).build()?;
    TemplateExporter::from_path(&profile.resolved_template_path())?.export_with_context(&context)
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
    use std::fs;
    use std::io::Write;
    use std::sync::Mutex;

    use crate::domain::palette::generate_palette;
    use crate::domain::params::ThemeParams;
    use crate::domain::rules::RuleSet;
    use crate::evaluator::resolve_theme;

    use super::{ExportFormat, ExportProfile, ExportWriteMode, export_with_profile};
    use tempfile::{NamedTempFile, tempdir};

    static CURRENT_DIR_LOCK: Mutex<()> = Mutex::new(());

    struct CurrentDirRestore(std::path::PathBuf);

    impl Drop for CurrentDirRestore {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.0).unwrap();
        }
    }

    fn current_dir_test_guard() -> std::sync::MutexGuard<'static, ()> {
        CURRENT_DIR_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

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
            write_mode: ExportWriteMode::ReplaceFile,
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
    fn export_with_profile_still_exports_legacy_alacritty_profiles() {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = ExportProfile {
            name: "Alacritty".to_string(),
            enabled: true,
            output_path: std::path::PathBuf::from("exports/alacritty-theme.toml"),
            write_mode: ExportWriteMode::ReplaceFile,
            format: ExportFormat::Alacritty,
        };

        let output = export_with_profile(&profile, "Ignored Project", &theme, &params).unwrap();

        assert!(output.contains("[colors.primary]"));
        assert!(output.contains("background = \""));
    }

    #[test]
    fn export_with_profile_resolves_bundled_alacritty_templates_without_using_current_dir() {
        let _guard = current_dir_test_guard();
        let _restore = CurrentDirRestore(std::env::current_dir().unwrap());
        let temp_dir = tempdir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let output = export_with_profile(
            &ExportProfile::alacritty_default(),
            "Ignored Project",
            &theme,
            &params,
        );

        let output = output.unwrap();
        assert!(output.contains("[colors.primary]"));
        assert!(output.contains("background = \""));
    }

    #[test]
    fn export_with_profile_resolves_explicit_bundled_alacritty_templates_without_using_current_dir()
    {
        let _guard = current_dir_test_guard();
        let _restore = CurrentDirRestore(std::env::current_dir().unwrap());
        let temp_dir = tempdir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let output = export_with_profile(
            &ExportProfile {
                name: "Alacritty".to_string(),
                enabled: true,
                output_path: std::path::PathBuf::from("exports/alacritty-theme.toml"),
                write_mode: ExportWriteMode::ReplaceFile,
                format: ExportFormat::Template {
                    template_path: crate::export::alacritty::bundled_template_path(),
                },
            },
            "Ignored Project",
            &theme,
            &params,
        );

        let output = output.unwrap();
        assert!(output.contains("[colors.primary]"));
        assert!(output.contains("background = \""));
    }

    #[test]
    fn export_with_profile_resolves_explicit_bundled_generic_templates_without_using_current_dir() {
        let _guard = current_dir_test_guard();
        let _restore = CurrentDirRestore(std::env::current_dir().unwrap());
        let temp_dir = tempdir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let output = export_with_profile(
            &ExportProfile {
                name: "Template".to_string(),
                enabled: true,
                output_path: std::path::PathBuf::from("exports/theme-template.txt"),
                write_mode: ExportWriteMode::ReplaceFile,
                format: ExportFormat::Template {
                    template_path: crate::export::alacritty::generic_template_path(),
                },
            },
            "Ignored Project",
            &theme,
            &params,
        );

        let output = output.unwrap();
        assert!(output.contains("profile=Template"));
        assert!(output.contains("background=#"));
    }

    #[test]
    fn export_with_profile_resolves_default_generic_template_without_using_current_dir() {
        let _guard = current_dir_test_guard();
        let _restore = CurrentDirRestore(std::env::current_dir().unwrap());
        let temp_dir = tempdir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let output = export_with_profile(
            &ExportProfile::template_default(),
            "Ignored Project",
            &theme,
            &params,
        )
        .unwrap();

        assert!(output.contains("profile=Template"));
        assert!(output.contains("background=#"));
    }

    #[test]
    fn export_with_profile_does_not_rewrite_relative_paths_matching_bundled_templates() {
        let _guard = current_dir_test_guard();
        let _restore = CurrentDirRestore(std::env::current_dir().unwrap());
        let temp_dir = tempdir().unwrap();
        let template_dir = temp_dir.path().join("templates");

        fs::create_dir_all(&template_dir).unwrap();
        fs::write(
            template_dir.join("generic-theme.txt"),
            "project-local={{meta.project_name}}\n",
        )
        .unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let output = export_with_profile(
            &ExportProfile {
                name: "Template".to_string(),
                enabled: true,
                output_path: std::path::PathBuf::from("exports/theme-template.txt"),
                write_mode: ExportWriteMode::ReplaceFile,
                format: ExportFormat::Template {
                    template_path: std::path::PathBuf::from("templates/generic-theme.txt"),
                },
            },
            "Project Local",
            &theme,
            &params,
        )
        .unwrap();

        assert_eq!(output, "project-local=Project Local\n");
    }

    #[test]
    fn default_alacritty_profile_targets_managed_block_in_user_config() {
        let profile = ExportProfile::alacritty_default();

        assert_eq!(
            profile.output_path,
            std::path::PathBuf::from("~/.config/alacritty/alacritty.toml")
        );
        assert_eq!(profile.write_mode, ExportWriteMode::ManagedBlock);
    }
}
