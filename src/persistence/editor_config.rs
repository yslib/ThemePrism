use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_PROJECT_PATH: &str = "projects/theme-project.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EditorStartupFocus {
    #[default]
    Tokens,
    Params,
    Inspector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorConfig {
    pub project_path: PathBuf,
    #[serde(default)]
    pub auto_load_project_on_startup: bool,
    #[serde(default)]
    pub auto_save_project_on_export: bool,
    #[serde(default)]
    pub startup_focus: EditorStartupFocus,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::from(DEFAULT_PROJECT_PATH),
            auto_load_project_on_startup: false,
            auto_save_project_on_export: false,
            startup_focus: EditorStartupFocus::Tokens,
        }
    }
}

#[derive(Debug, Error)]
pub enum EditorConfigError {
    #[error("platform does not expose a standard config directory")]
    UnsupportedPlatform,
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Parse(String),
}

pub fn load_editor_config() -> Result<EditorConfig, EditorConfigError> {
    let path = editor_config_path()?;
    load_editor_config_from_path(&path)
}

pub fn save_editor_config(config: &EditorConfig) -> Result<PathBuf, EditorConfigError> {
    let path = editor_config_path()?;
    save_editor_config_to_path(&path, config)?;
    Ok(path)
}

pub fn editor_config_path() -> Result<PathBuf, EditorConfigError> {
    let dirs =
        ProjectDirs::from("io", "ysl", "theme").ok_or(EditorConfigError::UnsupportedPlatform)?;
    Ok(dirs.config_dir().join("editor-config.toml"))
}

pub fn load_editor_config_from_path(path: &Path) -> Result<EditorConfig, EditorConfigError> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(EditorConfig::default());
        }
        Err(err) => return Err(EditorConfigError::Io(err.to_string())),
    };

    toml::from_str(&content).map_err(|err| EditorConfigError::Parse(err.to_string()))
}

pub fn save_editor_config_to_path(
    path: &Path,
    config: &EditorConfig,
) -> Result<(), EditorConfigError> {
    let content =
        toml::to_string_pretty(config).map_err(|err| EditorConfigError::Parse(err.to_string()))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| EditorConfigError::Io(err.to_string()))?;
    }

    fs::write(path, content).map_err(|err| EditorConfigError::Io(err.to_string()))
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::{
        EditorConfig, EditorStartupFocus, load_editor_config_from_path, save_editor_config_to_path,
    };

    #[test]
    fn editor_config_round_trips() {
        let file = NamedTempFile::new().unwrap();
        let config = EditorConfig {
            project_path: "projects/custom.toml".into(),
            auto_load_project_on_startup: true,
            auto_save_project_on_export: true,
            startup_focus: EditorStartupFocus::Inspector,
        };

        save_editor_config_to_path(file.path(), &config).unwrap();
        let loaded = load_editor_config_from_path(file.path()).unwrap();

        assert_eq!(loaded, config);
    }

    #[test]
    fn missing_editor_config_returns_default() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        drop(file);

        let loaded = load_editor_config_from_path(&path).unwrap();
        assert_eq!(loaded, EditorConfig::default());
    }
}
