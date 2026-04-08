use serde::Serialize;

use crate::app::controls::{ControlId, ReferenceField};
use crate::app::state::AppState;
use crate::app::update::current_source_for_control;
use crate::domain::preview::{sample_document, PreviewFrame};
use crate::domain::rules::{available_source_options, AdjustOp, Rule, RuleKind, SourceRef};
use crate::domain::tokens::{PaletteSlot, TokenRole};
use crate::i18n::{self, UiText};
use crate::persistence::editor_config::{EditorKeymapPreset, EditorLocale};

#[derive(Debug, Clone, Serialize)]
pub struct AppSnapshot {
    pub window_title: String,
    pub status: String,
    pub ui_text: GuiChromeSnapshot,
    pub project: ProjectSnapshot,
    pub config_sheet: ConfigSheetSnapshot,
    pub theme: ThemeSnapshot,
    pub tokens: Vec<TokenItemSnapshot>,
    pub params: Vec<ScalarFieldSnapshot>,
    pub editor_config: EditorConfigSnapshot,
    pub inspector: InspectorSnapshot,
    pub palette: Vec<SwatchSnapshot>,
    pub resolved_tokens: Vec<SwatchSnapshot>,
    pub preview: Vec<PreviewLineSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuiChromeSnapshot {
    pub theme_parameters_title: String,
    pub palette_title: String,
    pub preview_title: String,
    pub inspector_title: String,
    pub editor_preferences_title: String,
    pub actions_title: String,
    pub config_button_title: String,
    pub save_button_title: String,
    pub load_button_title: String,
    pub export_button_title: String,
    pub reset_button_title: String,
    pub config_sheet_title: String,
    pub config_sheet_subtitle: String,
    pub config_sheet_done_title: String,
    pub config_sheet_project_title: String,
    pub config_sheet_export_targets_title: String,
    pub config_sheet_editor_preferences_title: String,
    pub config_output_label: String,
    pub config_template_label: String,
    pub fixed_hex_placeholder: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSnapshot {
    pub name: String,
    pub project_path: String,
    pub export_targets_summary: String,
    pub enabled_outputs_summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigSheetSnapshot {
    pub project_name: TextFieldSnapshot,
    pub export_targets: Vec<ExportTargetSnapshot>,
    pub editor_fields: Vec<ConfigFieldSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportTargetSnapshot {
    pub index: usize,
    pub label: String,
    pub enabled: bool,
    pub output_path: String,
    pub template_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThemeSnapshot {
    pub background_hex: String,
    pub surface_hex: String,
    pub border_hex: String,
    pub selection_hex: String,
    pub text_hex: String,
    pub muted_hex: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenItemSnapshot {
    pub index: usize,
    pub id: String,
    pub label: String,
    pub category: String,
    pub color_hex: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScalarFieldSnapshot {
    pub id: String,
    pub label: String,
    pub value_text: String,
    pub current: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EditorConfigSnapshot {
    pub fields: Vec<ConfigFieldSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConfigFieldSnapshot {
    Text(TextFieldSnapshot),
    Choice(ChoiceFieldSnapshot),
}

#[derive(Debug, Clone, Serialize)]
pub struct TextFieldSnapshot {
    pub id: String,
    pub label: String,
    pub value_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectorSnapshot {
    pub token_id: String,
    pub token_label: String,
    pub token_color_hex: String,
    pub rule_summary: String,
    pub fields: Vec<EditorFieldSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EditorFieldSnapshot {
    Choice(ChoiceFieldSnapshot),
    Scalar(ScalarFieldSnapshot),
    Color(ColorFieldSnapshot),
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceFieldSnapshot {
    pub id: String,
    pub label: String,
    pub value_text: String,
    pub selected_key: String,
    pub options: Vec<ChoiceOptionSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceOptionSnapshot {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColorFieldSnapshot {
    pub id: String,
    pub label: String,
    pub value_text: String,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwatchSnapshot {
    pub label: String,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewLineSnapshot {
    pub segments: Vec<PreviewSegmentSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewSegmentSnapshot {
    pub text: String,
    pub foreground_hex: String,
    pub background_hex: String,
}

pub fn build_snapshot(state: &AppState) -> AppSnapshot {
    let locale = state.locale();
    let ui_text = GuiChromeSnapshot {
        theme_parameters_title: i18n::text(locale, UiText::GuiSectionThemeParameters),
        palette_title: i18n::text(locale, UiText::GuiSectionPalette),
        preview_title: i18n::text(locale, UiText::GuiSectionPreview),
        inspector_title: i18n::text(locale, UiText::GuiSectionInspector),
        editor_preferences_title: i18n::text(locale, UiText::GuiSectionEditorPreferences),
        actions_title: i18n::text(locale, UiText::GuiSectionActions),
        config_button_title: i18n::text(locale, UiText::GuiButtonConfig),
        save_button_title: i18n::text(locale, UiText::GuiButtonSave),
        load_button_title: i18n::text(locale, UiText::GuiButtonLoad),
        export_button_title: i18n::text(locale, UiText::GuiButtonExport),
        reset_button_title: i18n::text(locale, UiText::GuiButtonReset),
        config_sheet_title: i18n::text(locale, UiText::GuiSheetTitle),
        config_sheet_subtitle: i18n::text(locale, UiText::GuiSheetSubtitle),
        config_sheet_done_title: i18n::text(locale, UiText::GuiButtonDone),
        config_sheet_project_title: i18n::text(locale, UiText::GuiSheetSectionProject),
        config_sheet_export_targets_title: i18n::text(locale, UiText::GuiSheetSectionExportTargets),
        config_sheet_editor_preferences_title: i18n::text(
            locale,
            UiText::GuiSheetSectionEditorPreferences,
        ),
        config_output_label: i18n::text(locale, UiText::ConfigLabelOutput),
        config_template_label: i18n::text(locale, UiText::ConfigLabelTemplate),
        fixed_hex_placeholder: i18n::text(locale, UiText::GuiColorPlaceholder),
    };
    let project = ProjectSnapshot {
        name: state.project.name.clone(),
        project_path: state.editor.project_path.display().to_string(),
        export_targets_summary: export_targets_summary(state),
        enabled_outputs_summary: export_outputs_summary(state),
    };
    let config_sheet = ConfigSheetSnapshot {
        project_name: TextFieldSnapshot {
            id: "project_name".to_string(),
            label: i18n::text(locale, UiText::ConfigLabelProjectName),
            value_text: state.project.name.clone(),
        },
        export_targets: state
            .project
            .export_profiles
            .iter()
            .enumerate()
            .map(|(index, profile)| ExportTargetSnapshot {
                index,
                label: format!("{} ({})", profile.name, profile.format_label()),
                enabled: profile.enabled,
                output_path: profile.output_path.display().to_string(),
                template_path: Some(profile.configured_template_path().display().to_string()),
            })
            .collect(),
        editor_fields: editor_config_fields(state),
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

    let editor_config = EditorConfigSnapshot {
        fields: editor_config_fields(state),
    };

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

    let preview_document = match state.preview.active_mode {
        crate::domain::preview::PreviewMode::Code => {
            sample_document(|role| state.theme_color(role))
        }
        _ => match &state.preview.runtime_frame {
            PreviewFrame::Document(document) => document.clone(),
            PreviewFrame::Placeholder(message) | PreviewFrame::Error(message) => {
                crate::domain::preview::PreviewDocument {
                    lines: vec![
                        crate::domain::preview::PreviewLine {
                            spans: vec![crate::domain::preview::PreviewSpan {
                                text: message.title.clone(),
                                style: crate::domain::preview::PreviewSpanStyle {
                                    fg: Some(state.theme_color(TokenRole::TextMuted)),
                                    bg: Some(state.theme_color(TokenRole::Background)),
                                    bold: true,
                                    italic: false,
                                },
                            }],
                        },
                        crate::domain::preview::PreviewLine {
                            spans: vec![crate::domain::preview::PreviewSpan {
                                text: message.detail.clone(),
                                style: crate::domain::preview::PreviewSpanStyle {
                                    fg: Some(state.theme_color(TokenRole::TextMuted)),
                                    bg: Some(state.theme_color(TokenRole::Background)),
                                    bold: false,
                                    italic: false,
                                },
                            }],
                        },
                    ],
                }
            }
        },
    };
    let preview = preview_document
        .lines
        .into_iter()
        .map(|line| PreviewLineSnapshot {
            segments: line
                .spans
                .into_iter()
                .map(|span| PreviewSegmentSnapshot {
                    text: span.text,
                    foreground_hex: span
                        .style
                        .fg
                        .unwrap_or_else(|| state.theme_color(TokenRole::Text))
                        .to_hex(),
                    background_hex: span
                        .style
                        .bg
                        .unwrap_or_else(|| state.theme_color(TokenRole::Background))
                        .to_hex(),
                })
                .collect(),
        })
        .collect();

    AppSnapshot {
        window_title: i18n::window_title(locale, &state.project.name),
        status: state.ui.status.clone(),
        ui_text,
        project,
        config_sheet,
        theme,
        tokens,
        params,
        editor_config,
        inspector,
        palette,
        resolved_tokens,
        preview,
    }
}

fn export_targets_summary(state: &AppState) -> String {
    let locale = state.locale();
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .map(|profile| profile.name.as_str())
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => i18n::text(locale, UiText::SummaryNoneEnabled),
        [name] => i18n::format1(locale, UiText::SummaryOneEnabledNamed, "name", name),
        names if names.len() <= 3 => i18n::format2(
            locale,
            UiText::SummaryManyEnabledNamed,
            "count",
            names.len(),
            "names",
            names.join(", "),
        ),
        names => i18n::format1(
            locale,
            UiText::SummaryManyEnabledCount,
            "count",
            names.len(),
        ),
    }
}

fn export_outputs_summary(state: &AppState) -> String {
    let locale = state.locale();
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => i18n::text(locale, UiText::OutputsNoneEnabled),
        [profile] => i18n::format1(
            locale,
            UiText::OutputsOnePath,
            "path",
            profile.output_path.display(),
        ),
        profiles => i18n::format1(locale, UiText::OutputsManyPaths, "count", profiles.len()),
    }
}

fn editor_config_fields(state: &AppState) -> Vec<ConfigFieldSnapshot> {
    let locale = state.locale();
    vec![
        ConfigFieldSnapshot::Text(TextFieldSnapshot {
            id: "project_path".to_string(),
            label: i18n::text(locale, UiText::ConfigLabelProjectFile),
            value_text: state.editor.project_path.display().to_string(),
        }),
        ConfigFieldSnapshot::Choice(ChoiceFieldSnapshot {
            id: "keymap_preset".to_string(),
            label: i18n::text(locale, UiText::ConfigLabelKeymap),
            value_text: i18n::keymap_preset_label(locale, state.editor.keymap_preset),
            selected_key: encode_keymap_preset(state.editor.keymap_preset),
            options: [EditorKeymapPreset::Standard, EditorKeymapPreset::Vim]
                .into_iter()
                .map(|preset| ChoiceOptionSnapshot {
                    key: encode_keymap_preset(preset),
                    label: i18n::keymap_preset_label(locale, preset),
                })
                .collect(),
        }),
        ConfigFieldSnapshot::Choice(ChoiceFieldSnapshot {
            id: "locale".to_string(),
            label: i18n::text(locale, UiText::ConfigLabelLanguage),
            value_text: i18n::locale_label(locale, state.editor.locale),
            selected_key: encode_locale(state.editor.locale),
            options: EditorLocale::ALL
                .into_iter()
                .map(|choice| ChoiceOptionSnapshot {
                    key: encode_locale(choice),
                    label: i18n::locale_label(locale, choice),
                })
                .collect(),
        }),
    ]
}

fn inspector_fields(state: &AppState) -> Vec<EditorFieldSnapshot> {
    let locale = state.locale();
    let role = state.selected_role();
    let rule = state.selected_rule();
    let mut fields = vec![EditorFieldSnapshot::Choice(ChoiceFieldSnapshot {
        id: encode_control_id(ControlId::RuleKind(role)),
        label: i18n::text(locale, UiText::InspectorRuleType),
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
                &i18n::text(locale, UiText::InspectorSource),
            ));
        }
        Rule::Mix { ratio, .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(role, ReferenceField::MixA),
                &i18n::text(locale, UiText::InspectorColorA),
            ));
            fields.push(reference_field(
                state,
                ControlId::Reference(role, ReferenceField::MixB),
                &i18n::text(locale, UiText::InspectorColorB),
            ));
            fields.push(EditorFieldSnapshot::Scalar(ScalarFieldSnapshot {
                id: encode_control_id(ControlId::MixRatio(role)),
                label: i18n::text(locale, UiText::InspectorBlend),
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
                &i18n::text(locale, UiText::InspectorSource),
            ));
            fields.push(EditorFieldSnapshot::Choice(ChoiceFieldSnapshot {
                id: encode_control_id(ControlId::AdjustOp(role)),
                label: i18n::text(locale, UiText::InspectorOperation),
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
                label: i18n::text(locale, UiText::InspectorAmount),
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
                label: i18n::text(locale, UiText::InspectorHex),
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
    key.key().to_string()
}

pub fn encode_token_role(role: TokenRole) -> String {
    role.key().to_string()
}

pub fn encode_reference_field(field: ReferenceField) -> String {
    field.key().to_string()
}

pub fn encode_keymap_preset(preset: EditorKeymapPreset) -> String {
    preset.key().to_string()
}

pub fn encode_rule_kind(kind: RuleKind) -> String {
    kind.key().to_string()
}

pub fn encode_adjust_op(op: AdjustOp) -> String {
    op.key().to_string()
}

pub fn encode_locale(locale: EditorLocale) -> String {
    locale.key().to_string()
}

pub fn encode_source_ref(source: &SourceRef) -> String {
    match source {
        SourceRef::Token(role) => format!("token:{}", encode_token_role(*role)),
        SourceRef::Palette(slot) => format!("palette:{}", slot.key()),
        SourceRef::Literal(color) => format!("literal:{}", color.to_hex()),
    }
}
