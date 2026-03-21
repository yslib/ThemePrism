use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::params::ThemeParams;
use crate::rules::{AdjustOp, Rule, RuleSet, SourceRef};
use crate::tokens::{PaletteSlot, TokenRole};

#[derive(Debug)]
pub enum ProjectError {
    Io(String),
    Parse(String),
    InvalidData(String),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) | Self::Parse(message) | Self::InvalidData(message) => {
                f.write_str(message)
            }
        }
    }
}

impl Error for ProjectError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectFile {
    params: ProjectParams,
    rules: BTreeMap<String, RuleFile>,
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

pub fn save_project(
    path: &Path,
    params: &ThemeParams,
    rules: &RuleSet,
) -> Result<(), ProjectError> {
    let file = ProjectFile {
        params: ProjectParams::from(params),
        rules: rules
            .rules
            .iter()
            .map(|(role, rule)| (role.label().to_string(), RuleFile::from(rule)))
            .collect(),
    };

    let output =
        toml::to_string_pretty(&file).map_err(|err| ProjectError::Parse(err.to_string()))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| ProjectError::Io(err.to_string()))?;
    }
    fs::write(path, output).map_err(|err| ProjectError::Io(err.to_string()))
}

pub fn load_project(path: &Path) -> Result<(ThemeParams, RuleSet), ProjectError> {
    let content = fs::read_to_string(path).map_err(|err| ProjectError::Io(err.to_string()))?;
    let file: ProjectFile =
        toml::from_str(&content).map_err(|err| ProjectError::Parse(err.to_string()))?;

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

    Ok((params, RuleSet { rules }))
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
        SourceRef::Palette(slot) => slot.label().to_string(),
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
    match op {
        AdjustOp::Lighten => "Lighten",
        AdjustOp::Darken => "Darken",
        AdjustOp::Saturate => "Saturate",
        AdjustOp::Desaturate => "Desaturate",
    }
}

fn decode_adjust_op(value: &str) -> Result<AdjustOp, ProjectError> {
    match value {
        "Lighten" => Ok(AdjustOp::Lighten),
        "Darken" => Ok(AdjustOp::Darken),
        "Saturate" => Ok(AdjustOp::Saturate),
        "Desaturate" => Ok(AdjustOp::Desaturate),
        _ => Err(ProjectError::InvalidData(format!(
            "unknown adjust operation {value}"
        ))),
    }
}

fn decode_token_role(value: &str) -> Option<TokenRole> {
    match value {
        "Background" => Some(TokenRole::Background),
        "Surface" => Some(TokenRole::Surface),
        "SurfaceAlt" => Some(TokenRole::SurfaceAlt),
        "Text" => Some(TokenRole::Text),
        "TextMuted" => Some(TokenRole::TextMuted),
        "Border" => Some(TokenRole::Border),
        "Selection" => Some(TokenRole::Selection),
        "Cursor" => Some(TokenRole::Cursor),
        "Comment" => Some(TokenRole::Comment),
        "Keyword" => Some(TokenRole::Keyword),
        "String" => Some(TokenRole::String),
        "Number" => Some(TokenRole::Number),
        "Type" => Some(TokenRole::Type),
        "Function" => Some(TokenRole::Function),
        "Variable" => Some(TokenRole::Variable),
        "Error" => Some(TokenRole::Error),
        "Warning" => Some(TokenRole::Warning),
        "Info" => Some(TokenRole::Info),
        "Hint" => Some(TokenRole::Hint),
        "Success" => Some(TokenRole::Success),
        _ => None,
    }
}

fn decode_palette_slot(value: &str) -> Option<PaletteSlot> {
    match value {
        "bg_0" | "Bg0" => Some(PaletteSlot::Bg0),
        "bg_1" | "Bg1" => Some(PaletteSlot::Bg1),
        "bg_2" | "Bg2" => Some(PaletteSlot::Bg2),
        "fg_0" | "Fg0" => Some(PaletteSlot::Fg0),
        "fg_1" | "Fg1" => Some(PaletteSlot::Fg1),
        "fg_2" | "Fg2" => Some(PaletteSlot::Fg2),
        "accent_0" | "Accent0" => Some(PaletteSlot::Accent0),
        "accent_1" | "Accent1" => Some(PaletteSlot::Accent1),
        "accent_2" | "Accent2" => Some(PaletteSlot::Accent2),
        "accent_3" | "Accent3" => Some(PaletteSlot::Accent3),
        "accent_4" | "Accent4" => Some(PaletteSlot::Accent4),
        "accent_5" | "Accent5" => Some(PaletteSlot::Accent5),
        _ => None,
    }
}
