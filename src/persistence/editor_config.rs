use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unic_langid::LanguageIdentifier;

use crate::branding::EDITOR_CONFIG_APP_ID;
use crate::enum_meta::define_key_enum;

pub const DEFAULT_PROJECT_PATH: &str = "projects/themeprism-project.toml";

define_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
    #[serde(rename_all = "snake_case")]
    pub enum EditorKeymapPreset {
        #[default]
        Standard => "standard",
        Vim => "vim",
    }
}

impl EditorKeymapPreset {
    pub const fn next(self) -> Self {
        match self {
            Self::Standard => Self::Vim,
            Self::Vim => Self::Standard,
        }
    }
}

define_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
    #[serde(rename_all = "snake_case")]
    pub enum EditorLocale {
        #[default]
        EnUs => "en_us",
        ZhCn => "zh_cn",
    }
}

impl EditorLocale {
    pub const fn next(self) -> Self {
        match self {
            Self::EnUs => Self::ZhCn,
            Self::ZhCn => Self::EnUs,
        }
    }

    pub fn language_identifier(self) -> LanguageIdentifier {
        let raw = match self {
            Self::EnUs => "en-US",
            Self::ZhCn => "zh-CN",
        };
        raw.parse().expect("locale identifier should be valid")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorConfig {
    pub project_path: PathBuf,
    #[serde(default)]
    pub keymap_preset: EditorKeymapPreset,
    #[serde(default)]
    pub locale: EditorLocale,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::from(DEFAULT_PROJECT_PATH),
            keymap_preset: EditorKeymapPreset::Standard,
            locale: EditorLocale::EnUs,
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
    let dirs = ProjectDirs::from("io", "ysl", EDITOR_CONFIG_APP_ID)
        .ok_or(EditorConfigError::UnsupportedPlatform)?;
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
        EditorConfig, EditorKeymapPreset, EditorLocale, load_editor_config_from_path,
        save_editor_config_to_path,
    };

    #[test]
    fn editor_config_round_trips() {
        let file = NamedTempFile::new().unwrap();
        let config = EditorConfig {
            project_path: "projects/custom.toml".into(),
            keymap_preset: EditorKeymapPreset::Vim,
            locale: EditorLocale::ZhCn,
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

    #[test]
    fn editor_choice_enums_round_trip_through_stable_keys() {
        for preset in EditorKeymapPreset::ALL {
            assert_eq!(EditorKeymapPreset::from_key(preset.key()), Some(preset));
        }

        for locale in EditorLocale::ALL {
            assert_eq!(EditorLocale::from_key(locale.key()), Some(locale));
        }
    }
}
