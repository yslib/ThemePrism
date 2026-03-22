use crate::app::controls::{ControlSpec, DisplayFieldSpec};
use crate::app::state::{AppState, ConfigFieldId};
use crate::app::workspace::PanelId;
use crate::i18n::{self, UiText};

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
        title: i18n::text(state.locale(), UiText::PanelProjectConfig),
        active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: Some(i18n::text(state.locale(), UiText::ProjectConfigFooter)),
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
        title: i18n::text(state.locale(), UiText::PanelExportTargets),
        active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: Some(i18n::text(state.locale(), UiText::ExportTargetsFooter)),
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
        title: i18n::text(state.locale(), UiText::PanelEditorPreferences),
        active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::Form(FormView {
            header_lines: Vec::new(),
            fields,
            footer: Some(i18n::text(state.locale(), UiText::EditorPreferencesFooter)),
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
                label: config_field_label(state.locale(), field),
                value_text: config_field_value(state, field),
                swatch: None,
            }),
            selected: state.active_panel() == panel_id && index == selected_index,
        })
        .collect()
}
