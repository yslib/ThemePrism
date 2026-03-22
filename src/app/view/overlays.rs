use crate::app::actions::shortcut_help_sections;
use crate::app::controls::ControlId;
use crate::app::state::{AppState, ConfigFieldId, TextInputTarget};
use crate::app::update::{config_fields, filtered_source_options};
use crate::domain::color::Color;
use crate::domain::rules::Rule;
use crate::domain::tokens::TokenRole;
use crate::i18n::{self, UiText};

use super::helpers::{config_field_label, config_field_value, input_preview};
use super::styled::{colored_span, plain_span};
use super::{
    OverlayView, PickerOverlayView, PickerRowView, SpanStyle, StyledLine, StyledSpan, SurfaceBody,
    SurfaceSize, SurfaceView,
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
        title: i18n::format1(
            state.locale(),
            UiText::SourcePickerTitle,
            "label",
            picker.control.label(),
        ),
        filter: picker.filter.clone(),
        rows,
        selected_row,
        total_matches: options.len(),
    })
}

pub(crate) fn build_config_overlay(state: &AppState) -> Option<OverlayView> {
    let modal = state.ui.config_modal.as_ref()?;
    let fields = config_fields(state);
    let selected = fields
        .get(modal.selected_field.min(fields.len().saturating_sub(1)))
        .copied();

    let mut lines = vec![
        section_header_line(
            state,
            &i18n::text(state.locale(), UiText::ConfigSectionProjectSaved),
        ),
        config_field_line(state, ConfigFieldId::ProjectName, selected),
        section_header_line(
            state,
            &i18n::text(state.locale(), UiText::ConfigSectionExportSaved),
        ),
    ];

    for (index, profile) in state.project.export_profiles.iter().enumerate() {
        lines.push(config_field_line(
            state,
            ConfigFieldId::ExportEnabled(index),
            selected,
        ));
        lines.push(config_field_line(
            state,
            ConfigFieldId::ExportOutputPath(index),
            selected,
        ));
        if matches!(
            &profile.format,
            crate::export::ExportFormat::Template { .. }
        ) {
            lines.push(config_field_line(
                state,
                ConfigFieldId::ExportTemplatePath(index),
                selected,
            ));
        }
    }

    lines.push(section_header_line(
        state,
        &i18n::text(state.locale(), UiText::ConfigSectionEditorLocal),
    ));
    lines.push(config_field_line(
        state,
        ConfigFieldId::EditorProjectPath,
        selected,
    ));
    lines.push(config_field_line(
        state,
        ConfigFieldId::EditorAutoLoadProject,
        selected,
    ));
    lines.push(config_field_line(
        state,
        ConfigFieldId::EditorAutoSaveOnExport,
        selected,
    ));
    lines.push(config_field_line(
        state,
        ConfigFieldId::EditorStartupFocus,
        selected,
    ));
    lines.push(config_field_line(
        state,
        ConfigFieldId::EditorKeymapPreset,
        selected,
    ));
    lines.push(config_field_line(
        state,
        ConfigFieldId::EditorLocale,
        selected,
    ));

    let config_footer_edit = i18n::text(state.locale(), UiText::ConfigFooterEditHint);
    let config_footer_saved = i18n::text(state.locale(), UiText::ConfigFooterProjectSaved);
    let config_footer_local = i18n::text(state.locale(), UiText::ConfigFooterEditorLocal);
    let config_footer_close = i18n::text(state.locale(), UiText::ConfigFooterCloseHint);

    Some(OverlayView::Surface(SurfaceView {
        title: i18n::text(state.locale(), UiText::ConfigTitle),
        size: SurfaceSize::Percent {
            width: 68,
            height: 74,
        },
        body: SurfaceBody::Lines { lines, scroll: 0 },
        footer_lines: surface_footer_lines(
            state,
            &[
                &config_footer_edit,
                &config_footer_saved,
                &config_footer_local,
                &config_footer_close,
            ],
        ),
    }))
}

pub(crate) fn build_numeric_editor_overlay(state: &AppState) -> Option<OverlayView> {
    let input = state.ui.text_input.as_ref()?;
    let TextInputTarget::Control(control) = input.target else {
        return None;
    };
    let spec = numeric_editor_spec(state, control)?;
    let body_lines = vec![build_numeric_track_line(state, &spec, &input.buffer)];
    let numeric_footer = i18n::text(state.locale(), UiText::NumericFooter);
    let footer_lines = surface_footer_lines(state, &[&numeric_footer]);
    let preferred_height = body_lines.len() as u16 + footer_lines.len() as u16 + 4;

    Some(OverlayView::Surface(SurfaceView {
        title: i18n::format1(
            state.locale(),
            UiText::NumericEditorTitle,
            "label",
            &spec.label,
        ),
        size: SurfaceSize::Absolute {
            width: 54,
            height: preferred_height,
        },
        body: SurfaceBody::Lines {
            lines: body_lines,
            scroll: 0,
        },
        footer_lines,
    }))
}

pub(crate) fn build_help_overlay(state: &AppState) -> Option<OverlayView> {
    if !state.ui.shortcut_help_open {
        return None;
    }

    let mut lines = Vec::new();
    for (section_index, section) in
        shortcut_help_sections(state.locale(), state.editor.keymap_preset)
            .into_iter()
            .enumerate()
    {
        if section_index > 0 {
            lines.push(StyledLine { spans: Vec::new() });
        }
        lines.push(section_header_line(state, &section.title));
        for entry in section.entries {
            lines.push(help_entry_line(
                state,
                &entry.shortcut,
                &entry.label,
                &entry.description,
            ));
        }
    }

    let help_footer_scroll = i18n::text(state.locale(), UiText::HelpFooterScroll);
    let help_footer_keymap = i18n::text(state.locale(), UiText::HelpFooterKeymap);
    Some(OverlayView::Surface(SurfaceView {
        title: i18n::text(state.locale(), UiText::HelpTitle),
        size: SurfaceSize::Percent {
            width: 78,
            height: 80,
        },
        body: SurfaceBody::Lines {
            lines,
            scroll: state.ui.shortcut_help_scroll,
        },
        footer_lines: surface_footer_lines(state, &[&help_footer_scroll, &help_footer_keymap]),
    }))
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
                label: format!(
                    "{} / {}",
                    role.label(),
                    i18n::text(state.locale(), UiText::InspectorBlend)
                ),
                current: *ratio,
                min: 0.0,
                max: 1.0,
                track_kind: NumericTrackKind::Scalar,
            }),
            _ => None,
        },
        ControlId::AdjustAmount(role) => match state.domain.rules.get(role) {
            Some(Rule::Adjust { amount, .. }) => Some(NumericEditorSpec {
                label: format!(
                    "{} / {}",
                    role.label(),
                    i18n::text(state.locale(), UiText::InspectorAmount)
                ),
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

fn config_field_line(
    state: &AppState,
    field: ConfigFieldId,
    selected: Option<ConfigFieldId>,
) -> StyledLine {
    let is_selected = selected == Some(field);
    let label = format!("{:<12}", config_field_label(state.locale(), field));
    let value = config_field_value(state, field);
    let fg = if is_selected {
        state.theme_color(TokenRole::Background)
    } else {
        state.theme_color(TokenRole::Text)
    };
    let bg = is_selected.then_some(state.theme_color(TokenRole::Selection));

    StyledLine {
        spans: vec![
            styled_text_span(
                if is_selected { "> " } else { "  " },
                fg,
                bg,
                is_selected,
                false,
            ),
            styled_text_span(label, fg, bg, is_selected, false),
            styled_text_span(value, fg, bg, is_selected, false),
        ],
    }
}

fn section_header_line(state: &AppState, title: &str) -> StyledLine {
    StyledLine {
        spans: vec![colored_span(
            title.to_string(),
            state.theme_color(TokenRole::TextMuted),
            true,
            false,
        )],
    }
}

fn help_entry_line(state: &AppState, shortcut: &str, label: &str, description: &str) -> StyledLine {
    StyledLine {
        spans: vec![
            colored_span(
                format!("{:<16}", shortcut),
                state.theme_color(TokenRole::Selection),
                true,
                false,
            ),
            colored_span(
                format!("{:<16}", label),
                state.theme_color(TokenRole::Text),
                false,
                false,
            ),
            colored_span(
                description.to_string(),
                state.theme_color(TokenRole::TextMuted),
                false,
                false,
            ),
        ],
    }
}

fn surface_footer_lines(state: &AppState, lines: &[&str]) -> Vec<StyledLine> {
    lines
        .iter()
        .map(|line| StyledLine {
            spans: vec![colored_span(
                (*line).to_string(),
                state.theme_color(TokenRole::TextMuted),
                false,
                false,
            )],
        })
        .collect()
}

fn styled_text_span(
    text: impl Into<String>,
    fg: Color,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
) -> StyledSpan {
    StyledSpan {
        text: text.into(),
        style: SpanStyle {
            fg: Some(fg),
            bg,
            bold,
            italic,
        },
    }
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
