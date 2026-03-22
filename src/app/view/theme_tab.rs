use crate::app::controls::{
    ChoiceControlSpec, ColorControlSpec, ControlId, ControlSpec, DisplayFieldSpec, ReferenceField,
    ReferencePickerControlSpec, ScalarControlSpec,
};
use crate::app::state::AppState;
use crate::app::workspace::PanelId;
use crate::domain::preview::sample_code;
use crate::domain::rules::Rule;
use crate::domain::tokens::{PaletteSlot, TokenRole};

use super::helpers::{display_text_for_control, export_outputs_summary, export_targets_summary};
use super::styled::{colored_span, line_pair, plain_span, swatch_span};
use super::{
    CodePreviewView, FormFieldView, FormView, PanelBody, PanelView, SelectionListView,
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
        title: "Token List".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::SelectionList(SelectionListView { rows }),
    }
}

pub(crate) fn build_params_panel(state: &AppState) -> PanelView {
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
            label: "Project".to_string(),
            value_text: state.project.name.clone(),
            swatch: None,
        }),
        selected: false,
    });
    fields.push(FormFieldView {
        control: ControlSpec::Display(DisplayFieldSpec {
            label: "Exports".to_string(),
            value_text: export_targets_summary(state),
            swatch: None,
        }),
        selected: false,
    });
    fields.push(FormFieldView {
        control: ControlSpec::Display(DisplayFieldSpec {
            label: "Outputs".to_string(),
            value_text: export_outputs_summary(state),
            swatch: None,
        }),
        selected: false,
    });

    PanelView {
        id: PanelId::Params,
        title: "Theme Params".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: None,
        }),
    }
}

pub(crate) fn build_code_panel(state: &AppState) -> PanelView {
    let background = state.theme_color(TokenRole::Background);
    let lines = sample_code()
        .into_iter()
        .map(|segments| StyledLine {
            spans: segments
                .into_iter()
                .map(|segment| {
                    let role = segment.role.unwrap_or(TokenRole::Text);
                    StyledSpan {
                        text: segment.text.to_string(),
                        style: SpanStyle {
                            fg: Some(state.theme_color(role)),
                            bg: Some(background),
                            ..SpanStyle::default()
                        },
                    }
                })
                .collect(),
        })
        .collect();

    PanelView {
        id: PanelId::Preview,
        title: "Preview / Sample Code".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::CodePreview(CodePreviewView { lines }),
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
        title: "Palette".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::SwatchList(SwatchListView { items }),
    }
}

pub(crate) fn build_token_swatch_panel(
    state: &AppState,
    title: &str,
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
        id: PanelId::ResolvedPrimary,
        title: title.to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::SwatchList(SwatchListView { items }),
    }
}

pub(crate) fn build_inspector_panel(state: &AppState) -> PanelView {
    let color = state.current_token_color();
    let header_lines = vec![
        line_pair(
            "Token: ",
            state.selected_role().label(),
            state.theme_color(TokenRole::TextMuted),
            state.theme_color(TokenRole::Text),
            Some(color),
            false,
        ),
        StyledLine {
            spans: vec![
                colored_span(
                    "Color: ",
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
            "Summary: ",
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
        title: "Inspector".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::Form(FormView {
            header_lines,
            fields,
            footer: Some(inspector_footer_text(state).to_string()),
        }),
    }
}

fn build_inspector_fields(state: &AppState) -> Vec<FormFieldView> {
    let selected_role = state.selected_role();
    let mut fields = vec![FormFieldView {
        control: ControlSpec::Choice(ChoiceControlSpec {
            id: ControlId::RuleKind(selected_role),
            label: "Rule Type".to_string(),
            value_text: state.selected_rule().kind().label().to_string(),
        }),
        selected: state.active_panel() == PanelId::Inspector && state.ui.inspector_field == 0,
    }];

    match state.selected_rule() {
        Rule::Alias { .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(selected_role, ReferenceField::AliasSource),
                "Source",
                state.selected_rule(),
                1,
            ));
        }
        Rule::Mix { ratio, .. } => {
            fields.push(reference_field(
                state,
                ControlId::Reference(selected_role, ReferenceField::MixA),
                "Color A",
                state.selected_rule(),
                1,
            ));
            fields.push(reference_field(
                state,
                ControlId::Reference(selected_role, ReferenceField::MixB),
                "Color B",
                state.selected_rule(),
                2,
            ));
            fields.push(FormFieldView {
                control: ControlSpec::Scalar(ScalarControlSpec {
                    id: ControlId::MixRatio(selected_role),
                    label: "Blend".to_string(),
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
                "Source",
                state.selected_rule(),
                1,
            ));
            fields.push(FormFieldView {
                control: ControlSpec::Choice(ChoiceControlSpec {
                    id: ControlId::AdjustOp(selected_role),
                    label: "Operation".to_string(),
                    value_text: op.label().to_string(),
                }),
                selected: state.active_panel() == PanelId::Inspector
                    && state.ui.inspector_field == 2,
            });
            fields.push(FormFieldView {
                control: ControlSpec::Scalar(ScalarControlSpec {
                    id: ControlId::AdjustAmount(selected_role),
                    label: "Amount".to_string(),
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
                    label: "Hex".to_string(),
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

fn inspector_footer_text(state: &AppState) -> &'static str {
    if state.ui.source_picker.is_some() {
        "Filter sources by name. Tokens are common sources; palette slots are advanced."
    } else if let Some(input) = &state.ui.text_input {
        match input.target {
            crate::app::state::TextInputTarget::Control(control)
                if control.supports_numeric_editor() =>
            {
                "Numeric editor is open. Left/right nudges live; Enter applies the typed value."
            }
            crate::app::state::TextInputTarget::Control(ControlId::FixedColor(_)) => {
                "Type #C586C0 or #C586C080. Enter applies, Esc cancels."
            }
            crate::app::state::TextInputTarget::Config(_) => {
                "Type text. Enter applies, Esc cancels."
            }
            crate::app::state::TextInputTarget::Control(_) => {
                "Press Enter to apply or Esc to cancel."
            }
        }
    } else {
        match state.active_control() {
            Some(ControlId::Reference(_, _)) => {
                "Left/right cycles sources. Enter opens source picker with filter."
            }
            Some(ControlId::FixedColor(_)) => {
                "Left/right cycles colors. Enter or i opens hex input."
            }
            Some(ControlId::MixRatio(_)) => {
                "Left/right nudges the value. Enter opens the numeric editor."
            }
            Some(ControlId::AdjustAmount(_)) => {
                "Left/right nudges the value. Enter opens the numeric editor."
            }
            Some(ControlId::RuleKind(_)) => "Left/right cycles rule type.",
            Some(ControlId::AdjustOp(_)) => "Left/right cycles adjust operations.",
            Some(ControlId::Param(_)) => "Left/right steps values. Enter opens the numeric editor.",
            None => "Use left/right for quick edits and Enter to activate supported fields.",
        }
    }
}
