use crate::app::controls::{ControlSpec, DisplayFieldSpec};
use crate::app::state::{AppState, ConfigFieldId};
use crate::app::workspace::PanelId;

use super::helpers::{config_field_label, config_field_value};
use super::{FormFieldView, FormView, PanelBody, PanelView};

pub(crate) fn build_project_config_panel(state: &AppState) -> PanelView {
    let fields = config_panel_fields(
        state,
        PanelId::ProjectConfig,
        state.project_fields().into_iter(),
        state.ui.project_field,
    );

    PanelView {
        id: PanelId::ProjectConfig,
        title: "Project Config".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: Some("Project data is saved with the project file.".to_string()),
        }),
    }
}

pub(crate) fn build_export_targets_panel(state: &AppState) -> PanelView {
    let fields = config_panel_fields(
        state,
        PanelId::ExportTargets,
        state.export_fields().into_iter(),
        state.ui.export_field,
    );

    PanelView {
        id: PanelId::ExportTargets,
        title: "Export Targets".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: Some("Enable targets and edit output/template paths here.".to_string()),
        }),
    }
}

pub(crate) fn build_editor_preferences_panel(state: &AppState) -> PanelView {
    let fields = config_panel_fields(
        state,
        PanelId::EditorPreferences,
        state.editor_fields().into_iter(),
        state.ui.editor_field,
    );

    PanelView {
        id: PanelId::EditorPreferences,
        title: "Editor Preferences".to_string(),
        active: false,
        shortcut: None,
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: Some("Editor settings stay local to this machine.".to_string()),
        }),
    }
}

fn config_panel_fields(
    state: &AppState,
    panel_id: PanelId,
    fields: impl IntoIterator<Item = ConfigFieldId>,
    selected_index: usize,
) -> Vec<FormFieldView> {
    fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| FormFieldView {
            control: ControlSpec::Display(DisplayFieldSpec {
                label: config_field_label(field),
                value_text: config_field_value(state, field),
                swatch: None,
            }),
            selected: state.active_panel() == panel_id && index == selected_index,
        })
        .collect()
}
