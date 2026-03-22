use crate::app::controls::{
    ChoiceControlSpec, ColorControlSpec, ControlId, ControlSpec, DisplayFieldSpec, ReferenceField,
    ReferencePickerControlSpec, ScalarControlSpec,
};
use crate::app::state::{AppState, ConfigFieldId, FocusPane, TextInputTarget};
use crate::app::update::{config_fields, default_input_buffer, filtered_source_options};
use crate::domain::color::Color;
use crate::domain::preview::sample_code;
use crate::domain::rules::Rule;
use crate::domain::tokens::{PaletteSlot, TokenRole};

mod layout;

#[allow(unused_imports)]
pub use layout::{
    LayoutChild, WorkspaceLayout, WorkspaceSlot, child, column, compose_layout,
    default_workspace_layout, panel, preview_focus_layout, row, status_bar,
};

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
    Config(ConfigOverlayView),
    NumericEditor(NumericEditorOverlayView),
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
pub struct ConfigOverlayView {
    pub title: String,
    pub rows: Vec<ConfigRowView>,
    pub footer_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigRowView {
    pub label: String,
    pub value_text: String,
    pub selected: bool,
    pub is_header: bool,
}

#[derive(Debug, Clone)]
pub struct NumericEditorOverlayView {
    pub title: String,
    pub preferred_width: u16,
    pub preferred_height: u16,
    pub body_lines: Vec<StyledLine>,
    pub footer_lines: Vec<String>,
}

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
    build_view_with_layout(state, &default_workspace_layout())
}

pub fn build_view_with_layout(state: &AppState, workspace_layout: &WorkspaceLayout) -> ViewTree {
    let theme = ViewTheme {
        background: state.theme_color(TokenRole::Background),
        surface: state.theme_color(TokenRole::Surface),
        border: state.theme_color(TokenRole::Border),
        selection: state.theme_color(TokenRole::Selection),
        text: state.theme_color(TokenRole::Text),
        text_muted: state.theme_color(TokenRole::TextMuted),
    };

    let mut panel_for_slot = |slot| build_panel_for_slot(state, slot);
    let mut status_bar_view = || build_status_bar_view(state);
    let root = compose_layout(workspace_layout, &mut panel_for_slot, &mut status_bar_view);

    let mut overlays = Vec::new();
    if let Some(picker) = build_picker_overlay(state) {
        overlays.push(OverlayView::Picker(picker));
    }
    if let Some(config) = build_config_overlay(state) {
        overlays.push(OverlayView::Config(config));
    }
    if let Some(editor) = build_numeric_editor_overlay(state) {
        overlays.push(OverlayView::NumericEditor(editor));
    }

    ViewTree {
        theme,
        root,
        overlays,
    }
}

fn build_panel_for_slot(state: &AppState, slot: WorkspaceSlot) -> PanelView {
    match slot {
        WorkspaceSlot::Tokens => build_token_panel(state),
        WorkspaceSlot::Params => build_params_panel(state),
        WorkspaceSlot::Preview => build_code_panel(state),
        WorkspaceSlot::Palette => build_palette_panel(state),
        WorkspaceSlot::ResolvedPrimary => {
            build_token_swatch_panel(state, "Resolved Tokens", &TokenRole::ALL[..10])
        }
        WorkspaceSlot::ResolvedSecondary => {
            build_token_swatch_panel(state, "Resolved Tokens II", &TokenRole::ALL[10..])
        }
        WorkspaceSlot::Inspector => build_inspector_panel(state),
    }
}

fn build_status_bar_view(state: &AppState) -> StatusBarView {
    StatusBarView {
        focus_label: state.ui.focus.label().to_string(),
        help_text: status_help_text(state).to_string(),
        status_text: format!(
            "{}  |  Exports: {}",
            state.ui.status,
            export_status_summary(state)
        ),
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

fn build_config_overlay(state: &AppState) -> Option<ConfigOverlayView> {
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

fn build_numeric_editor_overlay(state: &AppState) -> Option<NumericEditorOverlayView> {
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

fn config_field_label(field: ConfigFieldId) -> String {
    match field {
        ConfigFieldId::ProjectName => "Project Name".to_string(),
        ConfigFieldId::ExportEnabled(index) => format!("Target {}", index + 1),
        ConfigFieldId::ExportOutputPath(index) => format!("Output {}", index + 1),
        ConfigFieldId::ExportTemplatePath(index) => format!("Template {}", index + 1),
        ConfigFieldId::EditorProjectPath => "Project File".to_string(),
        ConfigFieldId::EditorAutoLoadProject => "Auto Load".to_string(),
        ConfigFieldId::EditorAutoSaveOnExport => "Auto Save".to_string(),
        ConfigFieldId::EditorStartupFocus => "Startup Focus".to_string(),
    }
}

fn config_field_value(state: &AppState, field: ConfigFieldId) -> String {
    if let Some(input) = &state.ui.text_input {
        if input.target == TextInputTarget::Config(field) {
            return input_preview(&input.buffer);
        }
    }

    match field {
        ConfigFieldId::ProjectName => state.project.name.clone(),
        ConfigFieldId::EditorProjectPath => state.editor.project_path.display().to_string(),
        ConfigFieldId::EditorAutoLoadProject => {
            if state.editor.auto_load_project_on_startup {
                "[x] Load project on startup".to_string()
            } else {
                "[ ] Load project on startup".to_string()
            }
        }
        ConfigFieldId::EditorAutoSaveOnExport => {
            if state.editor.auto_save_project_on_export {
                "[x] Save project before export".to_string()
            } else {
                "[ ] Save project before export".to_string()
            }
        }
        ConfigFieldId::EditorStartupFocus => state.editor.startup_focus.label().to_string(),
        ConfigFieldId::ExportEnabled(index) => state
            .project
            .export_profiles
            .get(index)
            .map(|profile| profile.summary_label())
            .unwrap_or_else(|| "Missing export target".to_string()),
        ConfigFieldId::ExportOutputPath(index) => state
            .project
            .export_profiles
            .get(index)
            .map(|profile| profile.output_path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
        ConfigFieldId::ExportTemplatePath(index) => state
            .project
            .export_profiles
            .get(index)
            .and_then(|profile| match &profile.format {
                crate::export::ExportFormat::Template { template_path } => {
                    Some(template_path.display().to_string())
                }
                crate::export::ExportFormat::Alacritty => None,
            })
            .unwrap_or_else(|| "-".to_string()),
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

fn display_text_for_control(state: &AppState, control: ControlId) -> String {
    if let Some(input) = &state.ui.text_input {
        if input.target == TextInputTarget::Control(control) {
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
        _ => default_input_buffer(state, TextInputTarget::Control(control)),
    }
}

fn status_help_text(state: &AppState) -> &'static str {
    if state.ui.source_picker.is_some() {
        "↑↓ select  |  type to filter  |  Enter apply  |  Esc close"
    } else if let Some(input) = &state.ui.text_input {
        match input.target {
            TextInputTarget::Control(control) if control.supports_numeric_editor() => {
                "←→ nudge live  |  type exact value  |  Enter apply  |  Esc close  |  Del clear"
            }
            _ => "Enter apply  |  Esc cancel  |  Backspace delete",
        }
    } else if state.ui.config_modal.is_some() {
        "↑↓ select  |  Enter edit/toggle  |  Space toggle  |  Esc close"
    } else {
        "Tab focus  |  ↑↓ select  |  ←→ adjust  |  Enter/i activate  |  c config  |  s save  |  o load  |  e export  |  r reset  |  q quit"
    }
}

fn inspector_footer_text(state: &AppState) -> &'static str {
    if state.ui.source_picker.is_some() {
        "Filter sources by name. Tokens are common sources; palette slots are advanced."
    } else if let Some(input) = &state.ui.text_input {
        match input.target {
            TextInputTarget::Control(control) if control.supports_numeric_editor() => {
                "Numeric editor is open. Left/right nudges live; Enter applies the typed value."
            }
            TextInputTarget::Control(ControlId::FixedColor(_)) => {
                "Type #C586C0 or #C586C080. Enter applies, Esc cancels."
            }
            TextInputTarget::Config(_) => "Type text. Enter applies, Esc cancels.",
            TextInputTarget::Control(_) => "Press Enter to apply or Esc to cancel.",
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

fn export_targets_summary(state: &AppState) -> String {
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .map(|profile| profile.name.as_str())
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => "None enabled".to_string(),
        [name] => format!("1 enabled: {name}"),
        names if names.len() <= 3 => format!("{} enabled: {}", names.len(), names.join(", ")),
        names => format!("{} enabled", names.len()),
    }
}

fn export_outputs_summary(state: &AppState) -> String {
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => "No export targets enabled".to_string(),
        [profile] => profile.output_path.display().to_string(),
        profiles => format!("{} output targets", profiles.len()),
    }
}

fn export_status_summary(state: &AppState) -> String {
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .map(|profile| profile.name.as_str())
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => "none enabled".to_string(),
        [name] => format!("{name} enabled"),
        names if names.len() <= 3 => format!("{} enabled", names.join(", ")),
        names => format!("{} targets enabled", names.len()),
    }
}
