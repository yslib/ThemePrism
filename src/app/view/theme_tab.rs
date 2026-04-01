use crate::app::controls::{
    ChoiceControlSpec, ColorControlSpec, ControlId, ControlSpec, DisplayFieldSpec, ReferenceField,
    ReferencePickerControlSpec, ScalarControlSpec,
};
use crate::app::hint_nav::preview_tab_hint_label;
use crate::app::interaction::{SurfaceId, has_active_capture};
use crate::app::state::AppState;
use crate::app::workspace::PanelId;
use crate::domain::preview::{PreviewFrame, PreviewMode, PreviewSpanStyle, sample_document};
use crate::domain::rules::Rule;
use crate::domain::tokens::{PaletteSlot, TokenRole};
use crate::i18n::{self, UiText};

use super::helpers::{display_text_for_control, export_outputs_summary, export_targets_summary};
use super::styled::{colored_span, line_pair, plain_span, swatch_span};
use super::{
    DocumentView, FormFieldView, FormView, PanelBody, PanelTabView, PanelView, SelectionListView,
    SelectionRowView, SpanStyle, StyledLine, StyledSpan, SwatchItemView, SwatchListView,
};

pub(crate) fn build_token_panel(state: &AppState) -> PanelView {
    let mut rows = Vec::new();
    let mut current_category = None;

    for role in TokenRole::ALL {
        if current_category != Some(role.category()) {
            current_category = Some(role.category());
            rows.push(SelectionRowView::Header(
                role.category().label().to_string(),
            ));
        }
        rows.push(SelectionRowView::Item {
            label: role.label().to_string(),
            color: state.theme_color(role),
            selected: role == state.selected_role(),
        });
    }

    PanelView {
        id: PanelId::Tokens,
        title: i18n::panel_label(state.locale(), PanelId::Tokens),
        active: false,
        hint_navigation_active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::SelectionList(SelectionListView { rows }),
    }
}

pub(crate) fn build_params_panel(state: &AppState) -> PanelView {
    let locale = state.locale();
    let mut fields = crate::domain::params::ParamKey::ALL
        .into_iter()
        .map(|key| FormFieldView {
            control: ControlSpec::Scalar(ScalarControlSpec {
                id: ControlId::Param(key),
                label: key.label().to_string(),
                value_text: display_text_for_control(state, ControlId::Param(key)),
                current: key.get(&state.domain.params),
                min: key.range().0,
                max: key.range().1,
                step: key.step(),
            }),
            selected: state.active_panel() == PanelId::Params && key == state.selected_param_key(),
        })
        .collect::<Vec<_>>();

    fields.push(FormFieldView {
        control: ControlSpec::Display(DisplayFieldSpec {
            label: i18n::text(locale, UiText::FieldProject),
            value_text: state.project.name.clone(),
            swatch: None,
        }),
        selected: false,
    });
    fields.push(FormFieldView {
        control: ControlSpec::Display(DisplayFieldSpec {
            label: i18n::text(locale, UiText::FieldExports),
            value_text: export_targets_summary(state),
            swatch: None,
        }),
        selected: false,
    });
    fields.push(FormFieldView {
        control: ControlSpec::Display(DisplayFieldSpec {
            label: i18n::text(locale, UiText::FieldOutputs),
            value_text: export_outputs_summary(state),
            swatch: None,
        }),
        selected: false,
    });

    PanelView {
        id: PanelId::Params,
        title: i18n::panel_label(locale, PanelId::Params),
        active: false,
        hint_navigation_active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: None,
        }),
    }
}

pub(crate) fn build_preview_panel(state: &AppState) -> PanelView {
    let document = match state.preview.active_mode {
        PreviewMode::Code => sample_document(|role| state.theme_color(role)),
        _ => runtime_preview_document(&state.preview.runtime_frame, state),
    };
    let lines = document
        .lines
        .into_iter()
        .map(preview_line_to_view_line)
        .collect::<Vec<_>>();
    PanelView {
        id: PanelId::Preview,
        title: i18n::panel_label(state.locale(), PanelId::Preview),
        active: false,
        hint_navigation_active: false,
        shortcut: None,
        tabs: PreviewMode::ALL
            .iter()
            .copied()
            .map(|mode| PanelTabView {
                shortcut: preview_tab_hint_label(state, mode),
                label: i18n::preview_mode_label(state.locale(), mode),
                active: mode == state.preview.active_mode,
            })
            .collect(),
        header_lines: preview_header_lines(state),
        body: PanelBody::Document(DocumentView { lines, scroll: 0 }),
    }
}

pub(crate) fn build_palette_panel(state: &AppState) -> PanelView {
    let items = PaletteSlot::ALL
        .into_iter()
        .map(|slot| {
            let color = state.palette_color(slot);
            SwatchItemView {
                label: slot.label().to_string(),
                color,
                value_text: color.to_hex(),
            }
        })
        .collect();

    PanelView {
        id: PanelId::Palette,
        title: i18n::panel_label(state.locale(), PanelId::Palette),
        active: false,
        hint_navigation_active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::SwatchList(SwatchListView { items }),
    }
}

pub(crate) fn build_token_swatch_panel(
    state: &AppState,
    panel_id: PanelId,
    roles: &[TokenRole],
) -> PanelView {
    let items = roles
        .iter()
        .map(|role| {
            let color = state.theme_color(*role);
            SwatchItemView {
                label: role.label().to_string(),
                color,
                value_text: color.to_hex(),
            }
        })
        .collect();

    PanelView {
        id: panel_id,
        title: i18n::panel_label(state.locale(), panel_id),
        active: false,
        hint_navigation_active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::SwatchList(SwatchListView { items }),
    }
}

pub(crate) fn build_inspector_panel(state: &AppState) -> PanelView {
    let locale = state.locale();
    let color = state.current_token_color();
    let header_lines = vec![
        line_pair(
            &i18n::text(locale, UiText::InspectorToken),
            state.selected_role().label(),
            state.theme_color(TokenRole::TextMuted),
            state.theme_color(TokenRole::Text),
            Some(color),
            false,
        ),
        StyledLine {
            spans: vec![
                colored_span(
                    i18n::text(locale, UiText::InspectorColor),
                    state.theme_color(TokenRole::TextMuted),
                    false,
                    false,
                ),
                swatch_span(color, 4),
                plain_span(" "),
                colored_span(
                    color.to_hex(),
                    state.theme_color(TokenRole::Text),
                    false,
                    false,
                ),
            ],
        },
        line_pair(
            &i18n::text(locale, UiText::InspectorSummary),
            &state.selected_rule().summary(),
            state.theme_color(TokenRole::TextMuted),
            state.theme_color(TokenRole::Text),
            None,
            false,
        ),
        StyledLine { spans: Vec::new() },
    ];

    let fields = build_inspector_fields(state);

    PanelView {
        id: PanelId::Inspector,
        title: i18n::panel_label(locale, PanelId::Inspector),
        active: false,
        hint_navigation_active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::Form(FormView {
            header_lines,
            fields,
            footer: Some(inspector_footer_text(state)),
        }),
    }
}

fn preview_header_lines(state: &AppState) -> Vec<StyledLine> {
    let detail = if has_active_capture(state, SurfaceId::PreviewBody) {
        i18n::text(state.locale(), UiText::PreviewHeaderCaptureActive)
    } else if state.preview.active_mode == PreviewMode::Code {
        i18n::text(state.locale(), UiText::PreviewHeaderSemanticSample)
    } else if !state.preview.runtime_status.is_empty() {
        state.preview.runtime_status.clone()
    } else {
        i18n::text(state.locale(), UiText::PreviewHeaderSwitchModes)
    };

    vec![StyledLine {
        spans: vec![StyledSpan {
            text: detail,
            style: SpanStyle {
                fg: Some(state.theme_color(TokenRole::TextMuted)),
                bg: None,
                bold: false,
                italic: false,
            },
        }],
    }]
}

fn runtime_preview_document(
    frame: &PreviewFrame,
    state: &AppState,
) -> crate::domain::preview::PreviewDocument {
    match frame {
        PreviewFrame::Document(document) => document.clone(),
        PreviewFrame::Placeholder(message) => placeholder_document(
            state,
            &message.title,
            &message.detail,
            state.theme_color(TokenRole::TextMuted),
        ),
        PreviewFrame::Error(message) => placeholder_document(
            state,
            &message.title,
            &message.detail,
            state.theme_color(TokenRole::Error),
        ),
    }
}

fn placeholder_document(
    state: &AppState,
    title: &str,
    detail: &str,
    accent: crate::domain::color::Color,
) -> crate::domain::preview::PreviewDocument {
    use crate::domain::preview::{PreviewDocument, PreviewLine, PreviewSpan};

    PreviewDocument {
        lines: vec![
            PreviewLine {
                spans: vec![PreviewSpan {
                    text: title.to_string(),
                    style: PreviewSpanStyle {
                        fg: Some(accent),
                        bg: Some(state.theme_color(TokenRole::Background)),
                        bold: true,
                        italic: false,
                    },
                }],
            },
            PreviewLine {
                spans: vec![PreviewSpan {
                    text: detail.to_string(),
                    style: PreviewSpanStyle {
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

fn preview_line_to_view_line(line: crate::domain::preview::PreviewLine) -> StyledLine {
    StyledLine {
        spans: line
            .spans
            .into_iter()
            .map(|span| StyledSpan {
                text: span.text,
                style: SpanStyle {
                    fg: span.style.fg,
                    bg: span.style.bg,
                    bold: span.style.bold,
                    italic: span.style.italic,
                },
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::build_preview_panel;
    use crate::app::AppState;
    use crate::app::hint_nav::preview_tab_hint_label;
    use crate::app::interaction::{InteractionMode, SurfaceId};
    use crate::i18n;
    use crate::preview::PreviewMode;

    #[test]
    fn preview_panel_tabs_show_hint_shortcuts_during_navigation_mode() {
        let mut state = AppState::new().expect("state");
        state
            .ui
            .interaction
            .set_mode(InteractionMode::NavigateScope(SurfaceId::MainWindow));

        let panel = build_preview_panel(&state);

        assert_eq!(panel.tabs.len(), PreviewMode::ALL.len());
        for (tab, mode) in panel.tabs.iter().zip(PreviewMode::ALL.iter().copied()) {
            assert_eq!(tab.shortcut, preview_tab_hint_label(&state, mode));
            assert_eq!(tab.label, i18n::preview_mode_label(state.locale(), mode));
        }
    }

    #[test]
    fn preview_panel_tabs_hide_hint_shortcuts_outside_navigation_mode() {
        let state = AppState::new().expect("state");

        let panel = build_preview_panel(&state);

        assert_eq!(panel.tabs.len(), PreviewMode::ALL.len());
        for (tab, mode) in panel.tabs.iter().zip(PreviewMode::ALL.iter().copied()) {
            assert_eq!(tab.shortcut, None);
            assert_eq!(tab.label, i18n::preview_mode_label(state.locale(), mode));
        }
    }
}

fn build_inspector_fields(state: &AppState) -> Vec<FormFieldView> {
    let locale = state.locale();
    let selected_role = state.selected_role();
    let mut fields = vec![FormFieldView {
        control: ControlSpec::Choice(ChoiceControlSpec {
            id: ControlId::RuleKind(selected_role),
            label: i18n::text(locale, UiText::InspectorRuleType),
            value_text: state.selected_rule().kind().label().to_string(),
        }),
        selected: state.active_panel() == PanelId::Inspector && state.ui.inspector_field == 0,
    }];

    match state.selected_rule() {
        Rule::Alias { .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(selected_role, ReferenceField::AliasSource),
                &i18n::text(locale, UiText::InspectorSource),
                state.selected_rule(),
                1,
            ));
        }
        Rule::Mix { ratio, .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(selected_role, ReferenceField::MixA),
                &i18n::text(locale, UiText::InspectorColorA),
                state.selected_rule(),
                1,
            ));
            fields.push(reference_field(
                state,
                ControlId::Reference(selected_role, ReferenceField::MixB),
                &i18n::text(locale, UiText::InspectorColorB),
                state.selected_rule(),
                2,
            ));
            fields.push(FormFieldView {
                control: ControlSpec::Scalar(ScalarControlSpec {
                    id: ControlId::MixRatio(selected_role),
                    label: i18n::text(locale, UiText::InspectorBlend),
                    value_text: display_text_for_control(state, ControlId::MixRatio(selected_role)),
                    current: *ratio,
                    min: 0.0,
                    max: 1.0,
                    step: 0.05,
                }),
                selected: state.active_panel() == PanelId::Inspector
                    && state.ui.inspector_field == 3,
            });
        }
        Rule::Adjust { op, amount, .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(selected_role, ReferenceField::AdjustSource),
                &i18n::text(locale, UiText::InspectorSource),
                state.selected_rule(),
                1,
            ));
            fields.push(FormFieldView {
                control: ControlSpec::Choice(ChoiceControlSpec {
                    id: ControlId::AdjustOp(selected_role),
                    label: i18n::text(locale, UiText::InspectorOperation),
                    value_text: op.label().to_string(),
                }),
                selected: state.active_panel() == PanelId::Inspector
                    && state.ui.inspector_field == 2,
            });
            fields.push(FormFieldView {
                control: ControlSpec::Scalar(ScalarControlSpec {
                    id: ControlId::AdjustAmount(selected_role),
                    label: i18n::text(locale, UiText::InspectorAmount),
                    value_text: display_text_for_control(
                        state,
                        ControlId::AdjustAmount(selected_role),
                    ),
                    current: *amount,
                    min: 0.0,
                    max: 1.0,
                    step: 0.02,
                }),
                selected: state.active_panel() == PanelId::Inspector
                    && state.ui.inspector_field == 3,
            });
        }
        Rule::Fixed { color } => {
            fields.push(FormFieldView {
                control: ControlSpec::Color(ColorControlSpec {
                    id: ControlId::FixedColor(selected_role),
                    label: i18n::text(locale, UiText::InspectorHex),
                    value_text: display_text_for_control(
                        state,
                        ControlId::FixedColor(selected_role),
                    ),
                    color: *color,
                }),
                selected: state.active_panel() == PanelId::Inspector
                    && state.ui.inspector_field == 1,
            });
        }
    }

    fields
}

fn reference_field(
    state: &AppState,
    control: ControlId,
    label: &str,
    rule: &Rule,
    index: usize,
) -> FormFieldView {
    let value_text = crate::app::update::current_source_for_control(control, rule)
        .map(|source| source.label())
        .unwrap_or_default();

    FormFieldView {
        control: ControlSpec::ReferencePicker(ReferencePickerControlSpec {
            id: control,
            label: label.to_string(),
            value_text,
            picker_open: matches!(
                state.ui.source_picker.as_ref().map(|picker| picker.control),
                Some(active) if active == control
            ),
        }),
        selected: state.active_panel() == PanelId::Inspector && state.ui.inspector_field == index,
    }
}

fn inspector_footer_text(state: &AppState) -> String {
    if state.ui.source_picker.is_some() {
        i18n::text(state.locale(), UiText::FooterFilterSources)
    } else if let Some(input) = &state.ui.text_input {
        match input.target {
            crate::app::state::TextInputTarget::Control(control)
                if control.supports_numeric_editor() =>
            {
                i18n::text(state.locale(), UiText::FooterNumericEditorOpen)
            }
            crate::app::state::TextInputTarget::Control(ControlId::FixedColor(_)) => {
                i18n::text(state.locale(), UiText::FooterFixedColorInput)
            }
            crate::app::state::TextInputTarget::Config(_) => {
                i18n::text(state.locale(), UiText::FooterTextInput)
            }
            crate::app::state::TextInputTarget::Control(_) => {
                i18n::text(state.locale(), UiText::FooterGenericInput)
            }
        }
    } else {
        match state.active_control() {
            Some(ControlId::Reference(_, _)) => {
                i18n::text(state.locale(), UiText::FooterReferenceQuick)
            }
            Some(ControlId::FixedColor(_)) => {
                i18n::text(state.locale(), UiText::FooterFixedColorQuick)
            }
            Some(ControlId::MixRatio(_)) => i18n::text(state.locale(), UiText::FooterMixQuick),
            Some(ControlId::AdjustAmount(_)) => {
                i18n::text(state.locale(), UiText::FooterAdjustQuick)
            }
            Some(ControlId::RuleKind(_)) => i18n::text(state.locale(), UiText::FooterRuleKindQuick),
            Some(ControlId::AdjustOp(_)) => i18n::text(state.locale(), UiText::FooterAdjustOpQuick),
            Some(ControlId::Param(_)) => i18n::text(state.locale(), UiText::FooterParamQuick),
            None => i18n::text(state.locale(), UiText::FooterDefaultQuick),
        }
    }
}
