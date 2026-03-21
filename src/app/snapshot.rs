use crate::app::controls::{ControlId, ReferenceField};
use crate::app::state::AppState;
use crate::app::update::current_source_for_control;
use crate::domain::preview::sample_code;
use crate::domain::rules::{AdjustOp, Rule, RuleKind, SourceRef, available_source_options};
use crate::domain::tokens::{PaletteSlot, TokenRole};

#[derive(Debug, Clone)]
pub struct AppSnapshot {
    pub window_title: String,
    pub status: String,
    pub project: ProjectSnapshot,
    pub theme: ThemeSnapshot,
    pub tokens: Vec<TokenItemSnapshot>,
    pub params: Vec<ScalarFieldSnapshot>,
    pub inspector: InspectorSnapshot,
    pub palette: Vec<SwatchSnapshot>,
    pub resolved_tokens: Vec<SwatchSnapshot>,
    pub preview: Vec<PreviewLineSnapshot>,
}

#[derive(Debug, Clone)]
pub struct ProjectSnapshot {
    pub name: String,
    pub project_path: String,
    pub export_profile_name: String,
    pub export_format: String,
    pub export_output_path: String,
}

#[derive(Debug, Clone)]
pub struct ThemeSnapshot {
    pub background_hex: String,
    pub surface_hex: String,
    pub border_hex: String,
    pub selection_hex: String,
    pub text_hex: String,
    pub muted_hex: String,
}

#[derive(Debug, Clone)]
pub struct TokenItemSnapshot {
    pub index: usize,
    pub id: String,
    pub label: String,
    pub category: String,
    pub color_hex: String,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct ScalarFieldSnapshot {
    pub id: String,
    pub label: String,
    pub value_text: String,
    pub current: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

#[derive(Debug, Clone)]
pub struct InspectorSnapshot {
    pub token_id: String,
    pub token_label: String,
    pub token_color_hex: String,
    pub rule_summary: String,
    pub fields: Vec<EditorFieldSnapshot>,
}

#[derive(Debug, Clone)]
pub enum EditorFieldSnapshot {
    Choice(ChoiceFieldSnapshot),
    Scalar(ScalarFieldSnapshot),
    Color(ColorFieldSnapshot),
}

#[derive(Debug, Clone)]
pub struct ChoiceFieldSnapshot {
    pub id: String,
    pub label: String,
    pub value_text: String,
    pub selected_key: String,
    pub options: Vec<ChoiceOptionSnapshot>,
}

#[derive(Debug, Clone)]
pub struct ChoiceOptionSnapshot {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct ColorFieldSnapshot {
    pub id: String,
    pub label: String,
    pub value_text: String,
    pub color_hex: String,
}

#[derive(Debug, Clone)]
pub struct SwatchSnapshot {
    pub label: String,
    pub color_hex: String,
}

#[derive(Debug, Clone)]
pub struct PreviewLineSnapshot {
    pub segments: Vec<PreviewSegmentSnapshot>,
}

#[derive(Debug, Clone)]
pub struct PreviewSegmentSnapshot {
    pub text: String,
    pub foreground_hex: String,
    pub background_hex: String,
}

pub fn build_snapshot(state: &AppState) -> AppSnapshot {
    let project = ProjectSnapshot {
        name: state.project.name.clone(),
        project_path: state.project.project_path.display().to_string(),
        export_profile_name: state.project.export_profile.name.clone(),
        export_format: state.project.export_profile.format_label().to_string(),
        export_output_path: state
            .project
            .export_profile
            .output_path
            .display()
            .to_string(),
    };
    let theme = ThemeSnapshot {
        background_hex: state.theme_color(TokenRole::Background).to_hex(),
        surface_hex: state.theme_color(TokenRole::Surface).to_hex(),
        border_hex: state.theme_color(TokenRole::Border).to_hex(),
        selection_hex: state.theme_color(TokenRole::Selection).to_hex(),
        text_hex: state.theme_color(TokenRole::Text).to_hex(),
        muted_hex: state.theme_color(TokenRole::TextMuted).to_hex(),
    };

    let tokens = TokenRole::ALL
        .into_iter()
        .enumerate()
        .map(|(index, role)| TokenItemSnapshot {
            index,
            id: encode_token_role(role),
            label: role.label().to_string(),
            category: role.category().label().to_string(),
            color_hex: state.theme_color(role).to_hex(),
            selected: role == state.selected_role(),
        })
        .collect();

    let params = crate::domain::params::ParamKey::ALL
        .into_iter()
        .map(|key| ScalarFieldSnapshot {
            id: encode_control_id(ControlId::Param(key)),
            label: key.label().to_string(),
            value_text: key.format_value(&state.domain.params),
            current: key.get(&state.domain.params),
            min: key.range().0,
            max: key.range().1,
            step: key.step(),
        })
        .collect();

    let inspector = InspectorSnapshot {
        token_id: encode_token_role(state.selected_role()),
        token_label: state.selected_role().label().to_string(),
        token_color_hex: state.current_token_color().to_hex(),
        rule_summary: state.selected_rule().summary(),
        fields: inspector_fields(state),
    };

    let palette = PaletteSlot::ALL
        .into_iter()
        .map(|slot| SwatchSnapshot {
            label: slot.label().to_string(),
            color_hex: state.palette_color(slot).to_hex(),
        })
        .collect();

    let resolved_tokens = TokenRole::ALL
        .into_iter()
        .map(|role| SwatchSnapshot {
            label: role.label().to_string(),
            color_hex: state.theme_color(role).to_hex(),
        })
        .collect();

    let background_hex = state.theme_color(TokenRole::Background).to_hex();
    let preview = sample_code()
        .into_iter()
        .map(|line| PreviewLineSnapshot {
            segments: line
                .into_iter()
                .map(|segment| {
                    let role = segment.role.unwrap_or(TokenRole::Text);
                    PreviewSegmentSnapshot {
                        text: segment.text.to_string(),
                        foreground_hex: state.theme_color(role).to_hex(),
                        background_hex: background_hex.clone(),
                    }
                })
                .collect(),
        })
        .collect();

    AppSnapshot {
        window_title: format!("Theme Generator - {}", state.project.name),
        status: state.ui.status.clone(),
        project,
        theme,
        tokens,
        params,
        inspector,
        palette,
        resolved_tokens,
        preview,
    }
}

fn inspector_fields(state: &AppState) -> Vec<EditorFieldSnapshot> {
    let role = state.selected_role();
    let rule = state.selected_rule();
    let mut fields = vec![EditorFieldSnapshot::Choice(ChoiceFieldSnapshot {
        id: encode_control_id(ControlId::RuleKind(role)),
        label: "Rule Type".to_string(),
        value_text: rule.kind().label().to_string(),
        selected_key: encode_rule_kind(rule.kind()),
        options: RuleKind::ALL
            .into_iter()
            .map(|kind| ChoiceOptionSnapshot {
                key: encode_rule_kind(kind),
                label: kind.label().to_string(),
            })
            .collect(),
    })];

    match rule {
        Rule::Alias { .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(role, ReferenceField::AliasSource),
                "Source",
            ));
        }
        Rule::Mix { ratio, .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(role, ReferenceField::MixA),
                "Color A",
            ));
            fields.push(reference_field(
                state,
                ControlId::Reference(role, ReferenceField::MixB),
                "Color B",
            ));
            fields.push(EditorFieldSnapshot::Scalar(ScalarFieldSnapshot {
                id: encode_control_id(ControlId::MixRatio(role)),
                label: "Blend".to_string(),
                value_text: format!("{:>3.0}%", ratio * 100.0),
                current: *ratio,
                min: 0.0,
                max: 1.0,
                step: 0.05,
            }));
        }
        Rule::Adjust { op, amount, .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(role, ReferenceField::AdjustSource),
                "Source",
            ));
            fields.push(EditorFieldSnapshot::Choice(ChoiceFieldSnapshot {
                id: encode_control_id(ControlId::AdjustOp(role)),
                label: "Operation".to_string(),
                value_text: op.label().to_string(),
                selected_key: encode_adjust_op(*op),
                options: AdjustOp::ALL
                    .into_iter()
                    .map(|choice| ChoiceOptionSnapshot {
                        key: encode_adjust_op(choice),
                        label: choice.label().to_string(),
                    })
                    .collect(),
            }));
            fields.push(EditorFieldSnapshot::Scalar(ScalarFieldSnapshot {
                id: encode_control_id(ControlId::AdjustAmount(role)),
                label: "Amount".to_string(),
                value_text: format!("{:>3.0}%", amount * 100.0),
                current: *amount,
                min: 0.0,
                max: 1.0,
                step: 0.02,
            }));
        }
        Rule::Fixed { color } => {
            fields.push(EditorFieldSnapshot::Color(ColorFieldSnapshot {
                id: encode_control_id(ControlId::FixedColor(role)),
                label: "Hex".to_string(),
                value_text: color.to_hex(),
                color_hex: color.to_hex(),
            }));
        }
    }

    fields
}

fn reference_field(state: &AppState, control: ControlId, label: &str) -> EditorFieldSnapshot {
    let role = match control {
        ControlId::Reference(role, _) => role,
        _ => unreachable!("reference field requires a reference control"),
    };
    let current = current_source_for_control(control, state.selected_rule())
        .expect("selected rule should contain the requested source field");

    EditorFieldSnapshot::Choice(ChoiceFieldSnapshot {
        id: encode_control_id(control),
        label: label.to_string(),
        value_text: current.label(),
        selected_key: encode_source_ref(&current),
        options: available_source_options(role)
            .into_iter()
            .map(|option| ChoiceOptionSnapshot {
                key: encode_source_ref(&option.source),
                label: option.source.label(),
            })
            .collect(),
    })
}

pub fn encode_control_id(control: ControlId) -> String {
    match control {
        ControlId::Param(key) => format!("param:{}", encode_param_key(key)),
        ControlId::RuleKind(role) => format!("rule_kind:{}", encode_token_role(role)),
        ControlId::Reference(role, field) => {
            format!(
                "reference:{}:{}",
                encode_token_role(role),
                encode_reference_field(field)
            )
        }
        ControlId::MixRatio(role) => format!("mix_ratio:{}", encode_token_role(role)),
        ControlId::AdjustOp(role) => format!("adjust_op:{}", encode_token_role(role)),
        ControlId::AdjustAmount(role) => format!("adjust_amount:{}", encode_token_role(role)),
        ControlId::FixedColor(role) => format!("fixed_color:{}", encode_token_role(role)),
    }
}

pub fn encode_param_key(key: crate::domain::params::ParamKey) -> String {
    match key {
        crate::domain::params::ParamKey::BackgroundHue => "background_hue",
        crate::domain::params::ParamKey::BackgroundLightness => "background_lightness",
        crate::domain::params::ParamKey::BackgroundSaturation => "background_saturation",
        crate::domain::params::ParamKey::Contrast => "contrast",
        crate::domain::params::ParamKey::AccentHue => "accent_hue",
        crate::domain::params::ParamKey::AccentSaturation => "accent_saturation",
        crate::domain::params::ParamKey::AccentLightness => "accent_lightness",
        crate::domain::params::ParamKey::SelectionMix => "selection_mix",
        crate::domain::params::ParamKey::Vibrancy => "vibrancy",
    }
    .to_string()
}

pub fn encode_token_role(role: TokenRole) -> String {
    role.label().to_ascii_lowercase()
}

pub fn encode_reference_field(field: ReferenceField) -> String {
    match field {
        ReferenceField::AliasSource => "alias_source",
        ReferenceField::MixA => "mix_a",
        ReferenceField::MixB => "mix_b",
        ReferenceField::AdjustSource => "adjust_source",
    }
    .to_string()
}

pub fn encode_rule_kind(kind: RuleKind) -> String {
    kind.label().to_ascii_lowercase()
}

pub fn encode_adjust_op(op: AdjustOp) -> String {
    op.label().to_ascii_lowercase()
}

pub fn encode_source_ref(source: &SourceRef) -> String {
    match source {
        SourceRef::Token(role) => format!("token:{}", encode_token_role(*role)),
        SourceRef::Palette(slot) => format!("palette:{}", slot.label()),
        SourceRef::Literal(color) => format!("literal:{}", color.to_hex()),
    }
}
