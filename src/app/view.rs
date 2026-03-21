use crate::app::controls::{
    ChoiceControlSpec, ColorControlSpec, ControlId, ControlSpec, DisplayFieldSpec, ReferenceField,
    ReferencePickerControlSpec, ScalarControlSpec,
};
use crate::app::state::{AppState, FocusPane};
use crate::app::update::{default_input_buffer, filtered_source_options};
use crate::domain::color::Color;
use crate::domain::preview::sample_code;
use crate::domain::rules::Rule;
use crate::domain::tokens::{PaletteSlot, TokenRole};

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Length(u16),
    Min(u16),
    Percentage(u16),
}

#[derive(Debug, Clone)]
pub struct ViewTheme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub selection: Color,
    pub text: Color,
    pub text_muted: Color,
}

#[derive(Debug, Clone)]
pub struct ViewTree {
    pub theme: ViewTheme,
    pub root: ViewNode,
    pub overlays: Vec<OverlayView>,
}

#[derive(Debug, Clone)]
pub enum ViewNode {
    Split(SplitView),
    Panel(PanelView),
    StatusBar(StatusBarView),
}

#[derive(Debug, Clone)]
pub struct SplitView {
    pub axis: Axis,
    pub constraints: Vec<Size>,
    pub children: Vec<ViewNode>,
}

#[derive(Debug, Clone)]
pub struct PanelView {
    pub title: String,
    pub active: bool,
    pub body: PanelBody,
}

#[derive(Debug, Clone)]
pub enum PanelBody {
    SelectionList(SelectionListView),
    Form(FormView),
    CodePreview(CodePreviewView),
    SwatchList(SwatchListView),
}

#[derive(Debug, Clone)]
pub struct SelectionListView {
    pub rows: Vec<SelectionRowView>,
}

#[derive(Debug, Clone)]
pub enum SelectionRowView {
    Header(String),
    Item {
        label: String,
        color: Color,
        selected: bool,
    },
}

#[derive(Debug, Clone)]
pub struct FormView {
    pub header_lines: Vec<StyledLine>,
    pub fields: Vec<FormFieldView>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FormFieldView {
    pub control: ControlSpec,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct CodePreviewView {
    pub lines: Vec<StyledLine>,
}

#[derive(Debug, Clone)]
pub struct SwatchListView {
    pub items: Vec<SwatchItemView>,
}

#[derive(Debug, Clone)]
pub struct SwatchItemView {
    pub label: String,
    pub color: Color,
    pub value_text: String,
}

#[derive(Debug, Clone)]
pub struct StatusBarView {
    pub focus_label: String,
    pub help_text: String,
    pub status_text: String,
}

#[derive(Debug, Clone)]
pub enum OverlayView {
    Picker(PickerOverlayView),
}

#[derive(Debug, Clone)]
pub struct PickerOverlayView {
    pub title: String,
    pub filter: String,
    pub rows: Vec<PickerRowView>,
    pub selected_row: Option<usize>,
    pub total_matches: usize,
}

#[derive(Debug, Clone)]
pub struct PickerRowView {
    pub label: String,
    pub is_header: bool,
}

#[derive(Debug, Clone)]
pub struct StyledLine {
    pub spans: Vec<StyledSpan>,
}

#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub style: SpanStyle,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SpanStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

pub fn build_view(state: &AppState) -> ViewTree {
    let theme = ViewTheme {
        background: state.theme_color(TokenRole::Background),
        surface: state.theme_color(TokenRole::Surface),
        border: state.theme_color(TokenRole::Border),
        selection: state.theme_color(TokenRole::Selection),
        text: state.theme_color(TokenRole::Text),
        text_muted: state.theme_color(TokenRole::TextMuted),
    };

    let root = ViewNode::Split(SplitView {
        axis: Axis::Vertical,
        constraints: vec![Size::Min(12), Size::Length(2)],
        children: vec![
            ViewNode::Split(SplitView {
                axis: Axis::Horizontal,
                constraints: vec![Size::Length(34), Size::Min(48), Size::Length(38)],
                children: vec![
                    ViewNode::Split(SplitView {
                        axis: Axis::Vertical,
                        constraints: vec![Size::Percentage(58), Size::Percentage(42)],
                        children: vec![
                            ViewNode::Panel(build_token_panel(state)),
                            ViewNode::Panel(build_params_panel(state)),
                        ],
                    }),
                    ViewNode::Split(SplitView {
                        axis: Axis::Vertical,
                        constraints: vec![
                            Size::Percentage(45),
                            Size::Percentage(28),
                            Size::Percentage(27),
                        ],
                        children: vec![
                            ViewNode::Panel(build_code_panel(state)),
                            ViewNode::Panel(build_palette_panel(state)),
                            ViewNode::Split(SplitView {
                                axis: Axis::Horizontal,
                                constraints: vec![Size::Percentage(50), Size::Percentage(50)],
                                children: vec![
                                    ViewNode::Panel(build_token_swatch_panel(
                                        state,
                                        "Resolved Tokens",
                                        &TokenRole::ALL[..10],
                                    )),
                                    ViewNode::Panel(build_token_swatch_panel(
                                        state,
                                        "Resolved Tokens II",
                                        &TokenRole::ALL[10..],
                                    )),
                                ],
                            }),
                        ],
                    }),
                    ViewNode::Panel(build_inspector_panel(state)),
                ],
            }),
            ViewNode::StatusBar(StatusBarView {
                focus_label: state.ui.focus.label().to_string(),
                help_text: status_help_text(state).to_string(),
                status_text: format!(
                    "{}  |  Export: {} -> {}",
                    state.ui.status,
                    state.project.export_profile.name,
                    state.project.export_profile.output_path.display()
                ),
            }),
        ],
    });

    let mut overlays = Vec::new();
    if let Some(picker) = build_picker_overlay(state) {
        overlays.push(OverlayView::Picker(picker));
    }

    ViewTree {
        theme,
        root,
        overlays,
    }
}

fn build_token_panel(state: &AppState) -> PanelView {
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
        title: "Token List".to_string(),
        active: state.ui.focus == FocusPane::Tokens,
        body: PanelBody::SelectionList(SelectionListView { rows }),
    }
}

fn build_params_panel(state: &AppState) -> PanelView {
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
            selected: state.ui.focus == FocusPane::Params && key == state.selected_param_key(),
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
            label: "Exporter".to_string(),
            value_text: format!(
                "{} ({})",
                state.project.export_profile.name,
                state.project.export_profile.format_label()
            ),
            swatch: None,
        }),
        selected: false,
    });
    fields.push(FormFieldView {
        control: ControlSpec::Display(DisplayFieldSpec {
            label: "Output".to_string(),
            value_text: state.project.export_profile.output_path.display().to_string(),
            swatch: None,
        }),
        selected: false,
    });

    PanelView {
        title: "Theme Params".to_string(),
        active: state.ui.focus == FocusPane::Params,
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: None,
        }),
    }
}

fn build_code_panel(state: &AppState) -> PanelView {
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
        title: "Preview / Sample Code".to_string(),
        active: false,
        body: PanelBody::CodePreview(CodePreviewView { lines }),
    }
}

fn build_palette_panel(state: &AppState) -> PanelView {
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
        title: "Palette".to_string(),
        active: false,
        body: PanelBody::SwatchList(SwatchListView { items }),
    }
}

fn build_token_swatch_panel(state: &AppState, title: &str, roles: &[TokenRole]) -> PanelView {
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
        title: title.to_string(),
        active: false,
        body: PanelBody::SwatchList(SwatchListView { items }),
    }
}

fn build_inspector_panel(state: &AppState) -> PanelView {
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
        title: "Inspector".to_string(),
        active: state.ui.focus == FocusPane::Inspector,
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
        selected: state.ui.focus == FocusPane::Inspector && state.ui.inspector_field == 0,
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
                selected: state.ui.focus == FocusPane::Inspector && state.ui.inspector_field == 3,
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
                selected: state.ui.focus == FocusPane::Inspector && state.ui.inspector_field == 2,
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
                selected: state.ui.focus == FocusPane::Inspector && state.ui.inspector_field == 3,
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
                selected: state.ui.focus == FocusPane::Inspector && state.ui.inspector_field == 1,
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
        selected: state.ui.focus == FocusPane::Inspector && state.ui.inspector_field == index,
    }
}

fn build_picker_overlay(state: &AppState) -> Option<PickerOverlayView> {
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

fn display_text_for_control(state: &AppState, control: ControlId) -> String {
    if let Some(input) = &state.ui.text_input {
        if input.control == control {
            return input_preview(&input.buffer);
        }
    }

    match control {
        ControlId::Param(key) => key.format_value(&state.domain.params),
        ControlId::MixRatio(role) => match state.domain.rules.get(role) {
            Some(Rule::Mix { ratio, .. }) => format!("{:>3.0}%", ratio * 100.0),
            _ => String::new(),
        },
        ControlId::AdjustAmount(role) => match state.domain.rules.get(role) {
            Some(Rule::Adjust { amount, .. }) => format!("{:>3.0}%", amount * 100.0),
            _ => String::new(),
        },
        ControlId::FixedColor(role) => match state.domain.rules.get(role) {
            Some(Rule::Fixed { color }) => color.to_hex(),
            _ => String::new(),
        },
        _ => default_input_buffer(state, control),
    }
}

fn status_help_text(state: &AppState) -> &'static str {
    if state.ui.source_picker.is_some() {
        "↑↓ select  |  type to filter  |  Enter apply  |  Esc close"
    } else if state.ui.text_input.is_some() {
        "Enter apply  |  Esc cancel  |  Backspace delete"
    } else {
        "Tab focus  |  ↑↓ select  |  ←→ adjust  |  Enter/i activate  |  s save  |  o load  |  e export  |  r reset  |  q quit"
    }
}

fn inspector_footer_text(state: &AppState) -> &'static str {
    if state.ui.source_picker.is_some() {
        "Filter sources by name. Tokens are common sources; palette slots are advanced."
    } else if let Some(input) = &state.ui.text_input {
        match input.control {
            ControlId::Param(_) => "Type a number. Percent fields accept 35 or 35%.",
            ControlId::MixRatio(_) | ControlId::AdjustAmount(_) => {
                "Type 0.35 or 35%. Enter applies, Esc cancels."
            }
            ControlId::FixedColor(_) => {
                "Type a hex color like #C586C0. Enter applies, Esc cancels."
            }
            _ => "Press Enter to apply or Esc to cancel.",
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
                "Adjust ratio with left/right, or type a value on Blend."
            }
            Some(ControlId::AdjustAmount(_)) => {
                "Adjust amount with left/right, or type a value on Amount."
            }
            Some(ControlId::RuleKind(_)) => "Left/right cycles rule type.",
            Some(ControlId::AdjustOp(_)) => "Left/right cycles adjust operations.",
            Some(ControlId::Param(_)) => "Left/right steps values. Enter opens typed input.",
            None => "Use left/right for quick edits and Enter to activate supported fields.",
        }
    }
}

fn line_pair(
    prefix: &str,
    value: &str,
    prefix_color: Color,
    value_color: Color,
    swatch: Option<Color>,
    bold: bool,
) -> StyledLine {
    let mut spans = vec![colored_span(prefix, prefix_color, false, false)];
    if let Some(color) = swatch {
        spans.push(swatch_span(color, 4));
        spans.push(plain_span(" "));
    }
    spans.push(colored_span(value.to_string(), value_color, bold, false));
    StyledLine { spans }
}

fn colored_span(text: impl Into<String>, fg: Color, bold: bool, italic: bool) -> StyledSpan {
    StyledSpan {
        text: text.into(),
        style: SpanStyle {
            fg: Some(fg),
            bg: None,
            bold,
            italic,
        },
    }
}

fn plain_span(text: &str) -> StyledSpan {
    StyledSpan {
        text: text.to_string(),
        style: SpanStyle::default(),
    }
}

fn swatch_span(color: Color, width: usize) -> StyledSpan {
    StyledSpan {
        text: " ".repeat(width),
        style: SpanStyle {
            fg: None,
            bg: Some(color),
            bold: false,
            italic: false,
        },
    }
}

fn input_preview(buffer: &str) -> String {
    if buffer.is_empty() {
        "_".to_string()
    } else {
        format!("{buffer}_")
    }
}
