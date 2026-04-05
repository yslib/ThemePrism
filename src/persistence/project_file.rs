use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::app::effect::ProjectData;
use crate::color::Color;
use crate::export::{default_export_profiles, ExportProfile};
use crate::params::ThemeParams;
use crate::rules::{AdjustOp, Rule, RuleSet, SourceRef};
use crate::tokens::{PaletteSlot, TokenRole};

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Parse(String),
    #[error("{0}")]
    InvalidData(String),
}

const CURRENT_PROJECT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectFile {
    version: u32,
    project: ProjectMeta,
    params: ProjectParams,
    rules: BTreeMap<String, RuleFile>,
    #[serde(default)]
    exports: Vec<ExportProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    export: Option<ExportProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectMeta {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectParams {
    background_hue: f32,
    background_lightness: f32,
    background_saturation: f32,
    contrast: f32,
    accent_hue: f32,
    accent_saturation: f32,
    accent_lightness: f32,
    selection_mix: f32,
    vibrancy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RuleFile {
    Alias {
        source: String,
    },
    Mix {
        a: String,
        b: String,
        ratio: f32,
    },
    Adjust {
        source: String,
        op: String,
        amount: f32,
    },
    Fixed {
        color: String,
    },
}

pub fn save_project(path: &Path, project: &ProjectData) -> Result<(), ProjectError> {
    let file = ProjectFile {
        version: CURRENT_PROJECT_VERSION,
        project: ProjectMeta {
            name: project.name.clone(),
        },
        params: ProjectParams::from(&project.params),
        rules: project
            .rules
            .rules
            .iter()
            .map(|(role, rule)| (role.label().to_string(), RuleFile::from(rule)))
            .collect(),
        exports: project
            .export_profiles
            .iter()
            .map(ExportProfile::normalize)
            .collect(),
        export: None,
    };

    let output =
        toml::to_string_pretty(&file).map_err(|err| ProjectError::Parse(err.to_string()))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| ProjectError::Io(err.to_string()))?;
    }
    fs::write(path, output).map_err(|err| ProjectError::Io(err.to_string()))
}

pub fn load_project(path: &Path) -> Result<ProjectData, ProjectError> {
    let content = fs::read_to_string(path).map_err(|err| ProjectError::Io(err.to_string()))?;
    let file: ProjectFile =
        toml::from_str(&content).map_err(|err| ProjectError::Parse(err.to_string()))?;
    if file.version != CURRENT_PROJECT_VERSION {
        return Err(ProjectError::InvalidData(format!(
            "unsupported project version {}",
            file.version
        )));
    }

    let params = ThemeParams::from(file.params);
    let mut rules = BTreeMap::new();

    for role in TokenRole::ALL {
        let key = role.label();
        let rule_file = file
            .rules
            .get(key)
            .ok_or_else(|| ProjectError::InvalidData(format!("missing rule for {key}")))?;
        rules.insert(role, rule_file.to_rule()?);
    }

    let export_profiles = if !file.exports.is_empty() {
        file.exports
            .into_iter()
            .map(|profile| profile.normalize())
            .collect()
    } else if let Some(profile) = file.export {
        vec![profile.normalize()]
    } else {
        default_export_profiles()
    };

    Ok(ProjectData {
        name: file.project.name,
        params,
        rules: RuleSet { rules },
        export_profiles,
    })
}

impl From<&ThemeParams> for ProjectParams {
    fn from(value: &ThemeParams) -> Self {
        Self {
            background_hue: value.background_hue,
            background_lightness: value.background_lightness,
            background_saturation: value.background_saturation,
            contrast: value.contrast,
            accent_hue: value.accent_hue,
            accent_saturation: value.accent_saturation,
            accent_lightness: value.accent_lightness,
            selection_mix: value.selection_mix,
            vibrancy: value.vibrancy,
        }
    }
}

impl From<ProjectParams> for ThemeParams {
    fn from(value: ProjectParams) -> Self {
        Self {
            background_hue: value.background_hue,
            background_lightness: value.background_lightness,
            background_saturation: value.background_saturation,
            contrast: value.contrast,
            accent_hue: value.accent_hue,
            accent_saturation: value.accent_saturation,
            accent_lightness: value.accent_lightness,
            selection_mix: value.selection_mix,
            vibrancy: value.vibrancy,
        }
    }
}

impl From<&Rule> for RuleFile {
    fn from(value: &Rule) -> Self {
        match value {
            Rule::Alias { source } => Self::Alias {
                source: encode_source(source),
            },
            Rule::Mix { a, b, ratio } => Self::Mix {
                a: encode_source(a),
                b: encode_source(b),
                ratio: *ratio,
            },
            Rule::Adjust { source, op, amount } => Self::Adjust {
                source: encode_source(source),
                op: encode_adjust_op(*op).to_string(),
                amount: *amount,
            },
            Rule::Fixed { color } => Self::Fixed {
                color: color.to_hex(),
            },
        }
    }
}

impl RuleFile {
    fn to_rule(&self) -> Result<Rule, ProjectError> {
        Ok(match self {
            Self::Alias { source } => Rule::Alias {
                source: decode_source(source)?,
            },
            Self::Mix { a, b, ratio } => Rule::Mix {
                a: decode_source(a)?,
                b: decode_source(b)?,
                ratio: *ratio,
            },
            Self::Adjust { source, op, amount } => Rule::Adjust {
                source: decode_source(source)?,
                op: decode_adjust_op(op)?,
                amount: *amount,
            },
            Self::Fixed { color } => Rule::Fixed {
                color: Color::from_hex(color)
                    .map_err(|_| ProjectError::InvalidData(format!("invalid color {color}")))?,
            },
        })
    }
}

fn encode_source(source: &SourceRef) -> String {
    match source {
        SourceRef::Token(role) => role.label().to_string(),
        SourceRef::Palette(slot) => slot.key().to_string(),
        SourceRef::Literal(color) => color.to_hex(),
    }
}

fn decode_source(value: &str) -> Result<SourceRef, ProjectError> {
    if value.starts_with('#') {
        return Color::from_hex(value)
            .map(SourceRef::Literal)
            .map_err(|_| ProjectError::InvalidData(format!("invalid literal color {value}")));
    }

    if let Some(role) = decode_token_role(value) {
        return Ok(SourceRef::Token(role));
    }

    if let Some(slot) = decode_palette_slot(value) {
        return Ok(SourceRef::Palette(slot));
    }

    Err(ProjectError::InvalidData(format!(
        "unknown source reference {value}"
    )))
}

fn encode_adjust_op(op: AdjustOp) -> &'static str {
    op.label()
}

fn decode_adjust_op(value: &str) -> Result<AdjustOp, ProjectError> {
    AdjustOp::from_label(value)
        .or_else(|| AdjustOp::from_key(value))
        .ok_or_else(|| ProjectError::InvalidData(format!("unknown adjust operation {value}")))
}

fn decode_token_role(value: &str) -> Option<TokenRole> {
    TokenRole::from_label(value).or_else(|| TokenRole::from_key(value))
}

fn decode_palette_slot(value: &str) -> Option<PaletteSlot> {
    PaletteSlot::from_label(value).or_else(|| match value {
        "Bg0" => Some(PaletteSlot::Bg0),
        "Bg1" => Some(PaletteSlot::Bg1),
        "Bg2" => Some(PaletteSlot::Bg2),
        "Fg0" => Some(PaletteSlot::Fg0),
        "Fg1" => Some(PaletteSlot::Fg1),
        "Fg2" => Some(PaletteSlot::Fg2),
        "Accent0" => Some(PaletteSlot::Accent0),
        "Accent1" => Some(PaletteSlot::Accent1),
        "Accent2" => Some(PaletteSlot::Accent2),
        "Accent3" => Some(PaletteSlot::Accent3),
        "Accent4" => Some(PaletteSlot::Accent4),
        "Accent5" => Some(PaletteSlot::Accent5),
        _ => PaletteSlot::from_key(value),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::app::effect::ProjectData;
    use crate::export::{default_export_profiles, ExportFormat, ExportProfile};
    use crate::params::ThemeParams;
    use crate::persistence::project_file::{load_project, save_project};
    use crate::rules::RuleSet;
    use tempfile::NamedTempFile;

    fn project_with_exports(export_profiles: Vec<ExportProfile>) -> ProjectData {
        ProjectData {
            name: "Integration Theme".to_string(),
            params: ThemeParams::default(),
            rules: RuleSet::default(),
            export_profiles,
        }
    }

    fn bundled_alacritty_profile() -> ExportProfile {
        ExportProfile {
            name: "Alacritty".to_string(),
            enabled: true,
            output_path: "exports/alacritty-theme.toml".into(),
            format: ExportFormat::Template {
                template_path: "templates/alacritty.toml".into(),
            },
        }
    }

    #[test]
    fn load_project_normalizes_legacy_alacritty_export_profiles_to_template_profiles() {
        let file = NamedTempFile::new().unwrap();
        let project = project_with_exports(vec![bundled_alacritty_profile()]);

        save_project(file.path(), &project).unwrap();
        let saved = fs::read_to_string(file.path()).unwrap();
        let legacy = saved.replace(
            "type = \"template\"\ntemplate_path = \"templates/alacritty.toml\"\n",
            "type = \"alacritty\"\n",
        );
        fs::write(file.path(), legacy).unwrap();

        let loaded = load_project(file.path()).unwrap();

        assert_eq!(loaded.name, project.name);
        assert_eq!(loaded.export_profiles, vec![bundled_alacritty_profile()]);
    }

    #[test]
    fn load_project_uses_template_backed_default_export_profiles_when_exports_are_missing() {
        let file = NamedTempFile::new().unwrap();
        let project = project_with_exports(Vec::new());

        save_project(file.path(), &project).unwrap();
        let loaded = load_project(file.path()).unwrap();

        assert_eq!(loaded.name, project.name);
        assert_eq!(loaded.export_profiles, default_export_profiles());
        assert!(loaded
            .export_profiles
            .iter()
            .all(|profile| matches!(profile.format, ExportFormat::Template { .. })));
    }

    #[test]
    fn project_file_round_trips_template_export_profiles() {
        let file = NamedTempFile::new().unwrap();
        let project = project_with_exports(vec![
            bundled_alacritty_profile(),
            ExportProfile {
                name: "Custom Template".to_string(),
                enabled: true,
                output_path: "exports/custom.conf".into(),
                format: ExportFormat::Template {
                    template_path: "templates/custom.txt".into(),
                },
            },
        ]);

        save_project(file.path(), &project).unwrap();
        let saved = fs::read_to_string(file.path()).unwrap();
        let loaded = load_project(file.path()).unwrap();

        assert_eq!(loaded.name, project.name);
        assert_eq!(loaded.export_profiles, project.export_profiles);
        assert!(!saved.contains("type = \"alacritty\""));
    }
}
