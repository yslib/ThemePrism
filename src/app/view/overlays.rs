use crate::app::controls::ControlId;
use crate::app::state::{AppState, ConfigFieldId, TextInputTarget};
use crate::app::update::{config_fields, filtered_source_options};
use crate::domain::color::Color;
use crate::domain::rules::Rule;
use crate::domain::tokens::TokenRole;

use super::helpers::{config_field_label, config_field_value, input_preview};
use super::styled::{colored_span, plain_span};
use super::{
    ConfigOverlayView, ConfigRowView, NumericEditorOverlayView, PickerOverlayView, PickerRowView,
    SpanStyle, StyledLine, StyledSpan,
};

#[derive(Debug, Clone, Copy)]
enum NumericTrackKind {
    Hue,
    Scalar,
}

#[derive(Debug, Clone)]
struct NumericEditorSpec {
    label: String,
    current: f32,
    min: f32,
    max: f32,
    track_kind: NumericTrackKind,
}

pub(crate) fn build_picker_overlay(state: &AppState) -> Option<PickerOverlayView> {
    let picker = state.ui.source_picker.as_ref()?;
    let options = filtered_source_options(picker);
    let selected_index = if options.is_empty() {
        0
    } else {
        picker.selected.min(options.len().saturating_sub(1))
    };

    let mut rows = Vec::new();
    let mut selected_row = None;
    let mut current_group = None;

    for (index, option) in options.iter().enumerate() {
        if current_group != Some(option.group) {
            current_group = Some(option.group);
            rows.push(PickerRowView {
                label: option.group.label().to_string(),
                is_header: true,
            });
        }

        if index == selected_index {
            selected_row = Some(rows.len());
        }

        rows.push(PickerRowView {
            label: option.source.label(),
            is_header: false,
        });
    }

    Some(PickerOverlayView {
        title: format!("Source Picker / {}", picker.control.label()),
        filter: picker.filter.clone(),
        rows,
        selected_row,
        total_matches: options.len(),
    })
}

pub(crate) fn build_config_overlay(state: &AppState) -> Option<ConfigOverlayView> {
    let modal = state.ui.config_modal.as_ref()?;
    let fields = config_fields(state);
    let selected = fields
        .get(modal.selected_field.min(fields.len().saturating_sub(1)))
        .copied();

    let mut rows = vec![
        ConfigRowView {
            label: "Project Config (saved with project)".to_string(),
            value_text: String::new(),
            selected: false,
            is_header: true,
        },
        config_field_row(state, ConfigFieldId::ProjectName, selected),
        ConfigRowView {
            label: "Export Targets (saved with project)".to_string(),
            value_text: String::new(),
            selected: false,
            is_header: true,
        },
    ];

    for (index, profile) in state.project.export_profiles.iter().enumerate() {
        rows.push(config_field_row(
            state,
            ConfigFieldId::ExportEnabled(index),
            selected,
        ));
        rows.push(config_field_row(
            state,
            ConfigFieldId::ExportOutputPath(index),
            selected,
        ));
        if matches!(
            &profile.format,
            crate::export::ExportFormat::Template { .. }
        ) {
            rows.push(config_field_row(
                state,
                ConfigFieldId::ExportTemplatePath(index),
                selected,
            ));
        }
    }

    rows.push(ConfigRowView {
        label: "Editor Config (local only)".to_string(),
        value_text: String::new(),
        selected: false,
        is_header: true,
    });
    rows.push(config_field_row(
        state,
        ConfigFieldId::EditorProjectPath,
        selected,
    ));
    rows.push(config_field_row(
        state,
        ConfigFieldId::EditorAutoLoadProject,
        selected,
    ));
    rows.push(config_field_row(
        state,
        ConfigFieldId::EditorAutoSaveOnExport,
        selected,
    ));
    rows.push(config_field_row(
        state,
        ConfigFieldId::EditorStartupFocus,
        selected,
    ));

    Some(ConfigOverlayView {
        title: "Configuration".to_string(),
        rows,
        footer_lines: vec![
            "Enter edits text fields. Enter or Space toggles targets and cycles editor prefs."
                .to_string(),
            "Project config is saved with the project file.".to_string(),
            "Editor config stays local to this machine and editor instance.".to_string(),
            "Esc closes the panel.".to_string(),
        ],
    })
}

pub(crate) fn build_numeric_editor_overlay(state: &AppState) -> Option<NumericEditorOverlayView> {
    let input = state.ui.text_input.as_ref()?;
    let TextInputTarget::Control(control) = input.target else {
        return None;
    };
    let spec = numeric_editor_spec(state, control)?;
    let body_lines = vec![build_numeric_track_line(state, &spec, &input.buffer)];
    let footer_lines = vec!["←→ nudge  |  Enter apply  |  Esc close".to_string()];
    let preferred_height = body_lines.len() as u16 + footer_lines.len() as u16 + 4;

    Some(NumericEditorOverlayView {
        title: format!("Numeric Editor / {}", spec.label),
        preferred_width: 54,
        preferred_height,
        body_lines,
        footer_lines,
    })
}

fn config_field_row(
    state: &AppState,
    field: ConfigFieldId,
    selected: Option<ConfigFieldId>,
) -> ConfigRowView {
    ConfigRowView {
        label: config_field_label(field),
        value_text: config_field_value(state, field),
        selected: selected == Some(field),
        is_header: false,
    }
}

fn numeric_editor_spec(state: &AppState, control: ControlId) -> Option<NumericEditorSpec> {
    match control {
        ControlId::Param(key) => Some(NumericEditorSpec {
            label: key.label().to_string(),
            current: key.get(&state.domain.params),
            min: key.range().0,
            max: key.range().1,
            track_kind: match key {
                crate::domain::params::ParamKey::BackgroundHue
                | crate::domain::params::ParamKey::AccentHue => NumericTrackKind::Hue,
                _ => NumericTrackKind::Scalar,
            },
        }),
        ControlId::MixRatio(role) => match state.domain.rules.get(role) {
            Some(Rule::Mix { ratio, .. }) => Some(NumericEditorSpec {
                label: format!("{} / Blend", role.label()),
                current: *ratio,
                min: 0.0,
                max: 1.0,
                track_kind: NumericTrackKind::Scalar,
            }),
            _ => None,
        },
        ControlId::AdjustAmount(role) => match state.domain.rules.get(role) {
            Some(Rule::Adjust { amount, .. }) => Some(NumericEditorSpec {
                label: format!("{} / Amount", role.label()),
                current: *amount,
                min: 0.0,
                max: 1.0,
                track_kind: NumericTrackKind::Scalar,
            }),
            _ => None,
        },
        _ => None,
    }
}

fn build_numeric_track_line(
    state: &AppState,
    spec: &NumericEditorSpec,
    input_buffer: &str,
) -> StyledLine {
    const TRACK_WIDTH: usize = 28;

    let normalized = if (spec.max - spec.min).abs() < f32::EPSILON {
        0.0
    } else {
        ((spec.current - spec.min) / (spec.max - spec.min)).clamp(0.0, 1.0)
    };
    let marker_index = (normalized * (TRACK_WIDTH - 1) as f32).round() as usize;

    let mut spans = (0..TRACK_WIDTH)
        .map(|index| {
            let t = index as f32 / (TRACK_WIDTH - 1) as f32;
            let color = match spec.track_kind {
                NumericTrackKind::Hue => Color::from_hsl(t * 360.0, 0.72, 0.56),
                NumericTrackKind::Scalar => scalar_track_color(state, t, normalized),
            };
            StyledSpan {
                text: if index == marker_index {
                    "│".to_string()
                } else {
                    " ".to_string()
                },
                style: SpanStyle {
                    fg: if index == marker_index {
                        Some(state.theme_color(TokenRole::Background))
                    } else {
                        None
                    },
                    bg: Some(color),
                    bold: index == marker_index,
                    ..SpanStyle::default()
                },
            }
        })
        .collect::<Vec<_>>();
    spans.push(plain_span("  "));
    spans.push(colored_span(
        input_preview(input_buffer),
        state.theme_color(TokenRole::Text),
        true,
        false,
    ));
    StyledLine { spans }
}

fn scalar_track_color(state: &AppState, position: f32, fill: f32) -> Color {
    let filled_start = state.theme_color(TokenRole::Border);
    let filled_end = state.theme_color(TokenRole::Selection);
    let empty = state
        .theme_color(TokenRole::Surface)
        .mix(state.theme_color(TokenRole::Border), 0.45);

    if position <= fill {
        let segment = if fill <= 0.0 {
            0.0
        } else {
            (position / fill).clamp(0.0, 1.0)
        };
        filled_start.mix(filled_end, segment)
    } else {
        empty
    }
}
