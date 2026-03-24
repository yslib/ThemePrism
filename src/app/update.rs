use crate::app::controls::{ControlId, ReferenceField};
use crate::app::effect::{EditorConfigData, Effect, ProjectData};
use crate::app::intent::Intent;
use crate::app::interaction::{InteractionMode, SurfaceId, build_interaction_tree, surface_label};
use crate::app::state::{
    AppState, ConfigFieldId, ConfigModalState, FocusPane, SourcePickerState, TextInputState,
    TextInputTarget,
};
use crate::app::view::{interaction_panel_max_scroll, panel_order, workspace_layout_for_tab};
use crate::app::workspace::PanelId;
use crate::domain::color::Color;
use crate::domain::params::{ParamKey, ThemeParams};
use crate::domain::preview::{PreviewFrame, PreviewRuntimeEvent};
use crate::domain::rules::{
    AdjustOp, Rule, RuleKind, RuleSet, SourceOption, SourceRef, available_source_options,
    available_sources, starter_rule,
};
use crate::domain::tokens::TokenRole;
use crate::export::{ExportFormat, default_export_profiles};
use crate::i18n::{self, UiText};
use crate::persistence::editor_config::EditorKeymapPreset;

pub fn update(state: &mut AppState, intent: Intent) -> Vec<Effect> {
    match intent {
        Intent::QuitRequested => {
            state.ui.should_quit = true;
            Vec::new()
        }
        Intent::CycleWorkspaceTab(delta) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
                || state.ui.shortcut_help_open
            {
                return Vec::new();
            }
            cycle_workspace_tab(state, delta);
            Vec::new()
        }
        Intent::FocusPanelByNumber(number) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
                || state.ui.shortcut_help_open
            {
                return Vec::new();
            }
            focus_panel_by_number(state, number);
            Vec::new()
        }
        Intent::FocusSurface(surface) => {
            focus_surface(state, surface);
            Vec::new()
        }
        Intent::SetInteractionMode(mode) => {
            set_interaction_mode(state, mode);
            Vec::new()
        }
        Intent::MoveSelection(delta) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
                || state.ui.shortcut_help_open
            {
                return Vec::new();
            }
            move_selection(state, delta);
            Vec::new()
        }
        Intent::SelectToken(index) => {
            select_token(state, index);
            Vec::new()
        }
        Intent::AdjustControlByStep(control, delta) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
                || state.ui.shortcut_help_open
            {
                return Vec::new();
            }
            adjust_control_by_step(state, control, delta);
            Vec::new()
        }
        Intent::ActivateControl(control) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
                || state.ui.shortcut_help_open
            {
                return Vec::new();
            }
            activate_control(state, control);
            Vec::new()
        }
        Intent::AdjustActiveNumericInputByStep(delta) => {
            adjust_active_numeric_input(state, delta);
            Vec::new()
        }
        Intent::CyclePreviewMode(delta) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
                || state.ui.shortcut_help_open
            {
                return Vec::new();
            }
            cycle_preview_mode(state, delta);
            Vec::new()
        }
        Intent::SetPreviewCapture(active) => {
            set_preview_capture(state, active);
            Vec::new()
        }
        Intent::SetParamValue(key, value) => {
            set_param_value(state, key, value);
            Vec::new()
        }
        Intent::SetRuleKind(role, kind) => {
            set_rule_kind_for_role(state, role, kind);
            Vec::new()
        }
        Intent::SetReferenceSource(control, source) => {
            set_reference_source(state, control, source);
            Vec::new()
        }
        Intent::SetMixRatio(role, ratio) => {
            set_mix_ratio(state, role, ratio);
            Vec::new()
        }
        Intent::SetAdjustOp(role, op) => {
            set_adjust_op(state, role, op);
            Vec::new()
        }
        Intent::SetAdjustAmount(role, amount) => {
            set_adjust_amount(state, role, amount);
            Vec::new()
        }
        Intent::SetFixedColor(role, color) => {
            set_fixed_color(state, role, color);
            Vec::new()
        }
        Intent::SetProjectName(name) => set_project_name(state, name),
        Intent::SetExportEnabled(index, enabled) => set_export_enabled(state, index, enabled),
        Intent::SetExportOutputPath(index, path) => set_export_output_path(state, index, path),
        Intent::SetExportTemplatePath(index, path) => set_export_template_path(state, index, path),
        Intent::SetEditorProjectPath(path) => set_editor_project_path(state, path),
        Intent::SetEditorAutoLoadProject(enabled) => set_editor_auto_load_project(state, enabled),
        Intent::SetEditorAutoSaveOnExport(enabled) => {
            set_editor_auto_save_on_export(state, enabled)
        }
        Intent::SetEditorStartupFocus(focus) => set_editor_startup_focus(state, focus),
        Intent::SetEditorKeymapPreset(preset) => set_editor_keymap_preset(state, preset),
        Intent::SetEditorLocale(locale) => set_editor_locale(state, locale),
        Intent::AppendTextInput(ch) => {
            append_text_input(state, ch);
            Vec::new()
        }
        Intent::BackspaceTextInput => {
            if let Some(input) = &mut state.ui.text_input {
                input.buffer.pop();
            }
            Vec::new()
        }
        Intent::ClearTextInput => {
            if let Some(input) = &mut state.ui.text_input {
                input.buffer.clear();
            }
            Vec::new()
        }
        Intent::CommitTextInput => commit_text_input(state),
        Intent::CancelTextInput => {
            close_text_input_surface(state);
            state.ui.status = tr(state, UiText::StatusInputCancelled);
            Vec::new()
        }
        Intent::AppendSourcePickerFilter(ch) => {
            if let Some(picker) = &mut state.ui.source_picker {
                picker.filter.push(ch);
                picker.selected = 0;
            }
            Vec::new()
        }
        Intent::BackspaceSourcePickerFilter => {
            if let Some(picker) = &mut state.ui.source_picker {
                picker.filter.pop();
                picker.selected = 0;
            }
            Vec::new()
        }
        Intent::ClearSourcePickerFilter => {
            if let Some(picker) = &mut state.ui.source_picker {
                picker.filter.clear();
                picker.selected = 0;
            }
            Vec::new()
        }
        Intent::MoveSourcePickerSelection(delta) => {
            move_source_picker_selection(state, delta);
            Vec::new()
        }
        Intent::ApplySourcePickerSelection => {
            apply_source_picker_selection(state);
            Vec::new()
        }
        Intent::CloseSourcePicker => {
            close_source_picker_surface(state);
            state.ui.status = tr(state, UiText::StatusSourcePickerClosed);
            Vec::new()
        }
        Intent::OpenConfigRequested => {
            open_config_modal(state);
            Vec::new()
        }
        Intent::CloseConfigRequested => {
            close_config_modal(state);
            Vec::new()
        }
        Intent::ToggleShortcutHelpRequested => {
            toggle_shortcut_help(state);
            Vec::new()
        }
        Intent::ScrollShortcutHelp(delta) => {
            scroll_shortcut_help(state, delta);
            Vec::new()
        }
        Intent::MoveConfigSelection(delta) => {
            move_config_selection(state, delta);
            Vec::new()
        }
        Intent::ActivateConfigField => activate_config_field(state),
        Intent::SaveProjectRequested => vec![save_project_effect(state)],
        Intent::LoadProjectRequested => vec![Effect::LoadProject {
            path: state.editor.project_path.clone(),
        }],
        Intent::ExportThemeRequested => {
            let mut effects = Vec::new();
            if state.editor.auto_save_project_on_export {
                effects.push(save_project_effect(state));
            }
            effects.push(Effect::ExportTheme {
                profiles: state.project.export_profiles.clone(),
                theme: state.domain.resolved.clone(),
            });
            effects
        }
        Intent::ResetRequested => {
            reset_state(state);
            Vec::new()
        }
        Intent::ProjectSaved(result) => {
            state.ui.status = match result {
                Ok(path) => tr1(state, UiText::StatusSavedProject, "path", path.display()),
                Err(err) => tr1(state, UiText::StatusSaveFailed, "error", err),
            };
            Vec::new()
        }
        Intent::ProjectLoaded(result) => {
            match result {
                Ok(project) => match state.apply_project_data(project) {
                    Ok(()) => {
                        state.ui.status = tr1(
                            state,
                            UiText::StatusLoadedProject,
                            "path",
                            state.editor.project_path.display(),
                        );
                    }
                    Err(err) => {
                        state.ui.status =
                            tr1(state, UiText::StatusLoadRecomputeFailed, "error", err);
                    }
                },
                Err(err) => {
                    state.ui.status = tr1(state, UiText::StatusLoadFailed, "error", err);
                }
            }
            Vec::new()
        }
        Intent::ThemeExported(result) => {
            state.ui.status = match result {
                Ok(artifacts) if artifacts.is_empty() => tr(state, UiText::StatusExportNoOutput),
                Ok(artifacts) if artifacts.len() == 1 => tr2(
                    state,
                    UiText::StatusExportedSingle,
                    "profile",
                    &artifacts[0].profile_name,
                    "path",
                    artifacts[0].output_path.display(),
                ),
                Ok(artifacts) => tr1(state, UiText::StatusExportedCount, "count", artifacts.len()),
                Err(err) => tr1(state, UiText::StatusExportFailed, "error", err),
            };
            Vec::new()
        }
        Intent::PreviewRuntimeEvent(event) => {
            apply_preview_runtime_event(state, event);
            Vec::new()
        }
        Intent::EditorConfigSaved(result) => {
            if let Err(err) = result {
                state.ui.status = tr1(state, UiText::StatusEditorConfigSaveFailed, "error", err);
            }
            Vec::new()
        }
    }
}

fn cycle_preview_mode(state: &mut AppState, delta: i32) {
    if state.active_panel() != PanelId::Preview {
        return;
    }

    state.preview.active_mode = if delta >= 0 {
        state.preview.active_mode.next()
    } else {
        state.preview.active_mode.previous()
    };
    state.preview.capture_active = false;
    pop_capture_owner(state, SurfaceId::PreviewBody);
    state.preview.runtime_status.clear();
    if state.preview.active_mode.is_runtime_backed() {
        state.preview.runtime_frame = PreviewFrame::placeholder(
            tr(state, UiText::PreviewWaitingTitle),
            tr(state, UiText::PreviewWaitingDetail),
        );
        state.preview.runtime_status = tr(state, UiText::PreviewWaitingDetail);
    }
    state.ui.status = tr1(
        state,
        UiText::StatusPreviewModeChanged,
        "mode",
        i18n::preview_mode_label(state.locale(), state.preview.active_mode),
    );
}

fn set_preview_capture(state: &mut AppState, active: bool) {
    if active && !state.preview.active_mode.is_interactive() {
        return;
    }

    state.preview.capture_active = active;
    if active {
        push_capture_owner(state, SurfaceId::PreviewBody);
    } else {
        pop_capture_owner(state, SurfaceId::PreviewBody);
    }
    state.ui.status = if active {
        tr1(
            state,
            UiText::StatusPreviewCaptureActive,
            "mode",
            i18n::preview_mode_label(state.locale(), state.preview.active_mode),
        )
    } else {
        tr(state, UiText::StatusPreviewCaptureReleased)
    };
}

fn apply_preview_runtime_event(state: &mut AppState, event: PreviewRuntimeEvent) {
    match event {
        PreviewRuntimeEvent::FrameUpdated(frame) => {
            state.preview.runtime_frame = frame;
            state.preview.runtime_status.clear();
        }
        PreviewRuntimeEvent::StatusUpdated(status) => {
            state.preview.runtime_status = status;
        }
        PreviewRuntimeEvent::Exited { message } => {
            state.preview.capture_active = false;
            pop_capture_owner(state, SurfaceId::PreviewBody);
            state.preview.runtime_status = tr1(
                state,
                UiText::StatusPreviewProcessExited,
                "message",
                &message,
            );
            state.preview.runtime_frame =
                PreviewFrame::error(tr(state, UiText::PreviewExitedTitle), &message);
            state.ui.status = state.preview.runtime_status.clone();
        }
    }
}

fn tr(state: &AppState, key: UiText) -> String {
    i18n::text(state.locale(), key)
}

fn tr1(state: &AppState, key: UiText, name: &str, value: impl ToString) -> String {
    i18n::format1(state.locale(), key, name, value)
}

fn tr2(
    state: &AppState,
    key: UiText,
    name1: &str,
    value1: impl ToString,
    name2: &str,
    value2: impl ToString,
) -> String {
    i18n::format2(state.locale(), key, name1, value1, name2, value2)
}

fn cycle_workspace_tab(state: &mut AppState, delta: i32) {
    state.ui.active_tab = if delta >= 0 {
        state.ui.active_tab.next()
    } else {
        state.ui.active_tab.previous()
    };
    if state
        .ui
        .interaction
        .focused_workspace_surface()
        .is_workspace_panel()
    {
        state.ui.interaction.focus_panel(state.active_panel());
    }
    let panel = state.active_panel();
    state.ui.status = tr2(
        state,
        UiText::StatusSwitchedTab,
        "tab",
        i18n::workspace_tab_label(state.locale(), state.ui.active_tab),
        "panel",
        i18n::panel_label(state.locale(), panel),
    );
}

fn focus_panel_by_number(state: &mut AppState, number: u8) {
    let layout = workspace_layout_for_tab(state.ui.active_tab);
    let panels = panel_order(&layout);
    let index = number.saturating_sub(1) as usize;

    if let Some(panel) = panels.get(index).copied() {
        state.set_active_panel(panel);
        state.ui.interaction.focus_panel(panel);
        state.ui.status = tr1(
            state,
            UiText::StatusFocusedPanel,
            "panel",
            i18n::panel_label(state.locale(), panel),
        );
    } else {
        state.ui.status = tr2(
            state,
            UiText::StatusTabOnlyHasPanels,
            "tab",
            i18n::workspace_tab_label(state.locale(), state.ui.active_tab),
            "count",
            panels.len(),
        );
    }
}

fn move_selection(state: &mut AppState, delta: i32) {
    match state.active_panel() {
        PanelId::Tokens => {
            state.ui.selected_token =
                cycle_index(state.ui.selected_token, TokenRole::ALL.len(), delta);
            state.ui.inspector_field = state
                .ui
                .inspector_field
                .min(state.inspector_field_count().saturating_sub(1));
            close_source_picker_surface(state);
            state.ui.status = tr1(
                state,
                UiText::StatusSelectedToken,
                "token",
                state.selected_role().label(),
            );
        }
        PanelId::Params => {
            state.ui.selected_param =
                cycle_index(state.ui.selected_param, ParamKey::ALL.len(), delta);
            close_source_picker_surface(state);
            state.ui.status = tr1(
                state,
                UiText::StatusSelectedParam,
                "param",
                state.selected_param_key().label(),
            );
        }
        PanelId::Inspector => {
            state.ui.inspector_field = cycle_index(
                state.ui.inspector_field,
                state.inspector_field_count(),
                delta,
            );
            close_source_picker_surface(state);
        }
        PanelId::ProjectConfig => {
            move_project_field_selection(state, PanelId::ProjectConfig, delta)
        }
        PanelId::ExportTargets => {
            move_project_field_selection(state, PanelId::ExportTargets, delta)
        }
        PanelId::EditorPreferences => {
            move_project_field_selection(state, PanelId::EditorPreferences, delta)
        }
        PanelId::InteractionInspector => {
            let next_scroll = if delta < 0 {
                state.ui
                    .interaction_inspector_scroll
                    .saturating_sub(delta.unsigned_abs() as u16)
            } else {
                state.ui
                    .interaction_inspector_scroll
                    .saturating_add(delta as u16)
            };
            state.ui.interaction_inspector_scroll =
                next_scroll.min(interaction_panel_max_scroll(state));
        }
        PanelId::Preview
        | PanelId::Palette
        | PanelId::ResolvedPrimary
        | PanelId::ResolvedSecondary => {
            state.ui.status = tr1(
                state,
                UiText::StatusPanelNoListSelection,
                "panel",
                i18n::panel_label(state.locale(), state.active_panel()),
            );
        }
    }
}

fn focus_surface(state: &mut AppState, surface: SurfaceId) {
    match surface {
        SurfaceId::AppRoot => {}
        SurfaceId::MainWindow => {
            state.ui.interaction.focus_root();
            state.ui.status = tr1(
                state,
                UiText::StatusFocusedSurface,
                "surface",
                tr(state, UiText::SurfaceMainWindow),
            );
        }
        SurfaceId::TokensPanel
        | SurfaceId::ParamsPanel
        | SurfaceId::PreviewPanel
        | SurfaceId::PreviewTabs
        | SurfaceId::PreviewBody
        | SurfaceId::PalettePanel
        | SurfaceId::ResolvedPrimaryPanel
        | SurfaceId::ResolvedSecondaryPanel
        | SurfaceId::InspectorPanel
        | SurfaceId::InteractionInspectorPanel
        | SurfaceId::ProjectConfigPanel
        | SurfaceId::ExportTargetsPanel
        | SurfaceId::EditorPreferencesPanel => {
            let panel = surface.panel_id().expect("workspace surface");
            state.set_active_panel(panel);
            state.ui.interaction.focus_path = focus_path_for_surface(state, surface)
                .unwrap_or_else(|| {
                    vec![
                        SurfaceId::AppRoot,
                        SurfaceId::MainWindow,
                        SurfaceId::workspace_surface(panel),
                    ]
                });
            state.ui.status = tr1(
                state,
                UiText::StatusFocusedSurface,
                "surface",
                i18n::panel_label(state.locale(), panel),
            );
        }
        SurfaceId::NumericEditorSurface
        | SurfaceId::SourcePicker
        | SurfaceId::ConfigDialog
        | SurfaceId::ShortcutHelp => {
            if let Some(path) = focus_path_for_surface(state, surface) {
                state.ui.interaction.focus_path = path;
            }
        }
    }
}

fn focus_path_for_surface(state: &AppState, surface: SurfaceId) -> Option<Vec<SurfaceId>> {
    let tree = build_interaction_tree(state);
    if tree.node(surface).is_none() {
        return None;
    }

    let mut path = vec![surface];
    let mut current = surface;
    while let Some(parent) = tree.parent_of(current) {
        path.push(parent);
        current = parent;
    }
    path.reverse();
    Some(path)
}

fn set_interaction_mode(state: &mut AppState, mode: InteractionMode) {
    state.ui.interaction.set_mode(mode);
    if let InteractionMode::NavigateChildren(surface) = mode {
        state.ui.status = tr1(
            state,
            UiText::StatusSurfaceNavigationActive,
            "surface",
            surface_label(state, surface),
        );
    }
}

fn push_modal_owner(state: &mut AppState, owner: SurfaceId) {
    let mode = InteractionMode::Modal { owner };
    if state.ui.interaction.current_mode() != mode {
        state.ui.interaction.push_mode(mode);
    }
}

fn pop_modal_owner(state: &mut AppState, owner: SurfaceId) {
    state
        .ui
        .interaction
        .remove_mode(InteractionMode::Modal { owner });
}

fn push_capture_owner(state: &mut AppState, owner: SurfaceId) {
    let mode = InteractionMode::Capture { owner };
    if state.ui.interaction.current_mode() != mode {
        state.ui.interaction.push_mode(mode);
    }
}

fn pop_capture_owner(state: &mut AppState, owner: SurfaceId) {
    state
        .ui
        .interaction
        .remove_mode(InteractionMode::Capture { owner });
}

fn close_text_input_surface(state: &mut AppState) {
    if state.ui.text_input.take().is_some() {
        pop_modal_owner(state, SurfaceId::NumericEditorSurface);
    }
}

fn close_source_picker_surface(state: &mut AppState) {
    if state.ui.source_picker.take().is_some() {
        pop_modal_owner(state, SurfaceId::SourcePicker);
    }
}

fn close_config_surface(state: &mut AppState) {
    if state.ui.config_modal.take().is_some() {
        pop_modal_owner(state, SurfaceId::ConfigDialog);
    }
}

fn close_shortcut_help_surface(state: &mut AppState) {
    if state.ui.shortcut_help_open {
        state.ui.shortcut_help_open = false;
        state.ui.shortcut_help_scroll = 0;
        pop_modal_owner(state, SurfaceId::ShortcutHelp);
    }
}

fn move_project_field_selection(state: &mut AppState, panel: PanelId, delta: i32) {
    let len = match panel {
        PanelId::ProjectConfig => state.project_fields().len(),
        PanelId::ExportTargets => state.export_fields().len(),
        PanelId::EditorPreferences => state.editor_fields().len(),
        _ => 0,
    };
    if len == 0 {
        state.ui.status = tr1(
            state,
            UiText::StatusPanelNoEditableFields,
            "panel",
            i18n::panel_label(state.locale(), panel),
        );
        return;
    }

    match panel {
        PanelId::ProjectConfig => {
            state.ui.project_field = cycle_index(state.ui.project_field.min(len - 1), len, delta)
        }
        PanelId::ExportTargets => {
            state.ui.export_field = cycle_index(state.ui.export_field.min(len - 1), len, delta)
        }
        PanelId::EditorPreferences => {
            state.ui.editor_field = cycle_index(state.ui.editor_field.min(len - 1), len, delta)
        }
        _ => {}
    }
    if let Some(field) = state.active_config_field() {
        state.ui.status = tr1(
            state,
            UiText::StatusSelectedField,
            "field",
            input_target_label(state, TextInputTarget::Config(field)),
        );
    }
}

fn select_token(state: &mut AppState, index: usize) {
    state.ui.selected_token = index.min(TokenRole::ALL.len().saturating_sub(1));
    state.ui.inspector_field = state
        .ui
        .inspector_field
        .min(state.inspector_field_count().saturating_sub(1));
    close_source_picker_surface(state);
    close_text_input_surface(state);
    close_shortcut_help_surface(state);
    state.ui.status = tr1(
        state,
        UiText::StatusSelectedToken,
        "token",
        state.selected_role().label(),
    );
}

fn adjust_control_by_step(state: &mut AppState, control: ControlId, delta: i32) {
    match control {
        ControlId::Param(key) => adjust_param_by_step(state, key, delta),
        ControlId::RuleKind(role) => cycle_rule_kind_for_role(state, role, delta),
        ControlId::Reference(role, _) => cycle_reference_source(state, control, role, delta),
        ControlId::MixRatio(role) => set_mix_ratio_by_step(state, role, delta),
        ControlId::AdjustOp(role) => cycle_adjust_op_for_role(state, role, delta),
        ControlId::AdjustAmount(role) => set_adjust_amount_by_step(state, role, delta),
        ControlId::FixedColor(role) => cycle_fixed_color_for_role(state, role, delta),
    }
}

fn activate_control(state: &mut AppState, control: ControlId) {
    if control.supports_source_picker() {
        open_source_picker(state, control);
    } else if control.supports_text_input() {
        open_text_input(state, TextInputTarget::Control(control));
    } else {
        state.ui.status = tr1(
            state,
            UiText::StatusControlNoActivation,
            "control",
            control.label(),
        );
    }
}

fn set_param_value(state: &mut AppState, key: ParamKey, value: f32) {
    let mut params = state.domain.params.clone();
    key.set(&mut params, value);
    state.domain.params = params;
    if let Err(err) = state.recompute() {
        state.ui.status = tr2(
            state,
            UiText::StatusFailedToUpdateField,
            "field",
            key.label(),
            "error",
            err,
        );
    } else {
        state.ui.status = tr2(
            state,
            UiText::StatusUpdatedFieldValue,
            "field",
            key.label(),
            "value",
            key.format_value(&state.domain.params),
        );
    }
}

fn adjust_param_by_step(state: &mut AppState, key: ParamKey, delta: i32) {
    let mut params = state.domain.params.clone();
    key.adjust(&mut params, delta);
    state.domain.params = params;
    if let Err(err) = state.recompute() {
        state.ui.status = tr2(
            state,
            UiText::StatusFailedToUpdateField,
            "field",
            key.label(),
            "error",
            err,
        );
    } else {
        state.ui.status = tr2(
            state,
            UiText::StatusUpdatedFieldValue,
            "field",
            key.label(),
            "value",
            key.format_value(&state.domain.params),
        );
    }
}

fn set_rule_kind_for_role(state: &mut AppState, role: TokenRole, kind: RuleKind) {
    let current_color = state
        .domain
        .resolved
        .token(role)
        .expect("resolved theme should contain every token");
    let selection_mix = state.domain.params.selection_mix;
    let result = try_mutate_rules(state, |rules| {
        let rule = rules
            .get_mut(role)
            .expect("selected token should have rule");
        *rule = starter_rule(kind, role, current_color, selection_mix);
    });

    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn cycle_rule_kind_for_role(state: &mut AppState, role: TokenRole, delta: i32) {
    let current_color = state
        .domain
        .resolved
        .token(role)
        .expect("resolved theme should contain every token");
    let selection_mix = state.domain.params.selection_mix;
    let result = try_mutate_rules(state, |rules| {
        let rule = rules
            .get_mut(role)
            .expect("selected token should have rule");
        let current = rule.kind();
        let next = cycle_rule_kind(current, delta);
        *rule = starter_rule(next, role, current_color, selection_mix);
    });

    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_reference_source(state: &mut AppState, control: ControlId, source: SourceRef) {
    let result = try_mutate_rules(state, |rules| {
        let role =
            role_for_reference_control(control).expect("control should be reference control");
        let rule = rules
            .get_mut(role)
            .expect("selected token should have a rule");
        apply_source_to_control(control, rule, source.clone());
    });

    state.ui.status = match result {
        Ok(()) => tr1(
            state,
            UiText::StatusUpdatedEntity,
            "entity",
            control.label(),
        ),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn cycle_reference_source(state: &mut AppState, control: ControlId, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        let rule = rules
            .get_mut(role)
            .expect("selected token should have rule");
        let source = current_source_for_control(control, rule).expect("control should match rule");
        let next = cycle_source(&source, role, delta);
        apply_source_to_control(control, rule, next);
    });

    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_mix_ratio(state: &mut AppState, role: TokenRole, ratio: f32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio: current, .. }) = rules.get_mut(role) {
            *current = ratio.clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_mix_ratio_by_step(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio, .. }) = rules.get_mut(role) {
            *ratio = (*ratio + delta as f32 * 0.05).clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_adjust_op(state: &mut AppState, role: TokenRole, next: AdjustOp) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { op, .. }) = rules.get_mut(role) {
            *op = next;
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn cycle_adjust_op_for_role(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { op, .. }) = rules.get_mut(role) {
            *op = cycle_adjust_op(*op, delta);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_adjust_amount(state: &mut AppState, role: TokenRole, amount: f32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust {
            amount: current, ..
        }) = rules.get_mut(role)
        {
            *current = amount.clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_adjust_amount_by_step(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { amount, .. }) = rules.get_mut(role) {
            *amount = (*amount + delta as f32 * 0.02).clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_fixed_color(state: &mut AppState, role: TokenRole, next: Color) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color }) = rules.get_mut(role) {
            *color = next;
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn cycle_fixed_color_for_role(state: &mut AppState, role: TokenRole, delta: i32) {
    let options = state.fixed_color_options();
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color }) = rules.get_mut(role) {
            *color = cycle_fixed_color(*color, &options, delta);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn open_text_input(state: &mut AppState, target: TextInputTarget) {
    let buffer = default_input_buffer(state, target);
    close_shortcut_help_surface(state);
    state.ui.text_input = Some(TextInputState { target, buffer });
    push_modal_owner(state, SurfaceId::NumericEditorSurface);
    focus_surface(state, SurfaceId::NumericEditorSurface);
    state.ui.status = match target {
        TextInputTarget::Control(control) if control.supports_numeric_editor() => tr1(
            state,
            UiText::StatusEditingNumeric,
            "field",
            input_target_label(state, target),
        ),
        _ => tr1(
            state,
            UiText::StatusEditingText,
            "field",
            input_target_label(state, target),
        ),
    };
}

fn open_source_picker(state: &mut AppState, control: ControlId) {
    let role = role_for_reference_control(control).expect("control should be reference control");
    let current = state
        .domain
        .rules
        .get(role)
        .and_then(|rule| current_source_for_control(control, rule));
    let options = available_source_options(role);
    let selected = current
        .as_ref()
        .and_then(|source| options.iter().position(|option| option.source == *source))
        .unwrap_or_default();

    close_shortcut_help_surface(state);
    state.ui.source_picker = Some(SourcePickerState {
        control,
        filter: String::new(),
        selected,
    });
    push_modal_owner(state, SurfaceId::SourcePicker);
    focus_surface(state, SurfaceId::SourcePicker);
    state.ui.status = tr1(
        state,
        UiText::StatusSelectingSource,
        "control",
        control.label(),
    );
}

fn adjust_active_numeric_input(state: &mut AppState, delta: i32) {
    let Some(target) = state.ui.text_input.as_ref().map(|input| input.target) else {
        return;
    };
    let TextInputTarget::Control(control) = target else {
        return;
    };
    if !control.supports_numeric_editor() {
        return;
    }

    adjust_control_by_step(state, control, delta);
    let buffer = default_input_buffer(state, target);
    if let Some(input) = &mut state.ui.text_input {
        input.buffer = buffer;
    }
}

fn append_text_input(state: &mut AppState, ch: char) {
    let label = state
        .ui
        .text_input
        .as_ref()
        .map(|input| input_target_label(state, input.target))
        .unwrap_or_else(|| "input field".to_string());

    if let Some(input) = &mut state.ui.text_input {
        if accepts_input_char(input.target, ch, &input.buffer) {
            let ch = match input.target {
                TextInputTarget::Control(ControlId::FixedColor(_)) => ch.to_ascii_uppercase(),
                _ => ch,
            };
            input.buffer.push(ch);
        } else {
            state.ui.status = tr2(
                state,
                UiText::StatusInvalidCharacter,
                "char",
                ch,
                "field",
                label,
            );
        }
    }
}

fn commit_text_input(state: &mut AppState) -> Vec<Effect> {
    let Some(input) = state.ui.text_input.clone() else {
        return Vec::new();
    };

    let result = match input.target {
        TextInputTarget::Control(control) => match control {
            ControlId::Param(key) => apply_param_input(state, key, &input.buffer),
            ControlId::MixRatio(role) => apply_mix_ratio_input(state, role, &input.buffer),
            ControlId::AdjustAmount(role) => apply_adjust_amount_input(state, role, &input.buffer),
            ControlId::FixedColor(role) => apply_fixed_color_input(state, role, &input.buffer),
            _ => Err(tr1(
                state,
                UiText::ErrorControlNoTextInput,
                "control",
                control.label(),
            )),
        },
        TextInputTarget::Config(field) => apply_config_input(state, field, &input.buffer),
    };

    match result {
        Ok(status) => {
            close_text_input_surface(state);
            state.ui.status = status;
            effects_for_text_target(state, input.target)
        }
        Err(err) => {
            state.ui.status = err;
            Vec::new()
        }
    }
}

fn move_source_picker_selection(state: &mut AppState, delta: i32) {
    let len = state
        .ui
        .source_picker
        .as_ref()
        .map(|picker| filtered_source_options(picker).len())
        .unwrap_or_default();
    if len == 0 {
        return;
    }

    if let Some(picker) = &mut state.ui.source_picker {
        picker.selected = cycle_index(picker.selected.min(len - 1), len, delta);
    }
}

fn apply_source_picker_selection(state: &mut AppState) {
    let Some(picker) = state.ui.source_picker.clone() else {
        return;
    };

    let options = filtered_source_options(&picker);
    if options.is_empty() {
        state.ui.status = tr(state, UiText::ErrorNoSourcesMatch);
        return;
    }

    let selected = options[picker.selected.min(options.len() - 1)].clone();
    let source_label = selected.source.label();
    let result = try_mutate_rules(state, |rules| {
        let role =
            role_for_reference_control(picker.control).expect("picker control should be reference");
        let rule = rules
            .get_mut(role)
            .expect("selected token should have a rule");
        apply_source_to_control(picker.control, rule, selected.source.clone());
    });

    match result {
        Ok(()) => {
            close_source_picker_surface(state);
            state.ui.status = tr2(
                state,
                UiText::StatusSourceApplied,
                "control",
                picker.control.label(),
                "source",
                source_label,
            );
        }
        Err(err) => {
            state.ui.status = tr1(state, UiText::StatusSourceChangeRejected, "error", err);
        }
    }
}

fn apply_param_input(state: &mut AppState, key: ParamKey, buffer: &str) -> Result<String, String> {
    let value = parse_param_input(state, key, buffer)?;
    let mut params = state.domain.params.clone();
    key.set(&mut params, value);
    state.domain.params = params;
    state.recompute().map_err(|err| err.to_string())?;
    Ok(tr2(
        state,
        UiText::StatusUpdatedFieldValue,
        "field",
        key.label(),
        "value",
        key.format_value(&state.domain.params),
    ))
}

fn apply_mix_ratio_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let ratio = parse_fraction_input(state, buffer)?;
    try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio: current, .. }) = rules.get_mut(role) {
            *current = ratio;
        }
    })?;
    Ok(tr2(
        state,
        UiText::StatusBlendUpdated,
        "token",
        role.label(),
        "value",
        format!("{:>3.0}%", ratio * 100.0),
    ))
}

fn apply_adjust_amount_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let amount = parse_fraction_input(state, buffer)?;
    try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust {
            amount: current, ..
        }) = rules.get_mut(role)
        {
            *current = amount;
        }
    })?;
    Ok(tr2(
        state,
        UiText::StatusAmountUpdated,
        "token",
        role.label(),
        "value",
        format!("{:>3.0}%", amount * 100.0),
    ))
}

fn apply_fixed_color_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let color = Color::from_hex(buffer).map_err(|_| tr(state, UiText::ErrorInvalidHexColor))?;
    try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color: current }) = rules.get_mut(role) {
            *current = color;
        }
    })?;
    Ok(tr2(
        state,
        UiText::StatusFixedColorUpdated,
        "token",
        role.label(),
        "value",
        color.to_hex(),
    ))
}

fn try_mutate_rules(state: &mut AppState, update: impl FnOnce(&mut RuleSet)) -> Result<(), String> {
    let previous_rules = state.domain.rules.clone();
    update(&mut state.domain.rules);

    if let Err(err) = state.recompute() {
        state.domain.rules = previous_rules;
        let _ = state.recompute();
        Err(err.to_string())
    } else {
        state.ui.inspector_field = state
            .ui
            .inspector_field
            .min(state.inspector_field_count().saturating_sub(1));
        Ok(())
    }
}

fn reset_state(state: &mut AppState) {
    state.domain.params = ThemeParams::default();
    state.domain.rules = RuleSet::default();
    state.project.name = "Untitled Theme".to_string();
    state.project.export_profiles = default_export_profiles();
    state.ui.selected_token = 0;
    state.ui.selected_param = 0;
    state.ui.inspector_field = 0;
    close_text_input_surface(state);
    close_source_picker_surface(state);
    close_config_surface(state);
    close_shortcut_help_surface(state);
    state.preview.capture_active = false;
    pop_capture_owner(state, SurfaceId::PreviewBody);
    match state.recompute() {
        Ok(()) => state.ui.status = tr(state, UiText::StatusResetDefaults),
        Err(err) => state.ui.status = tr1(state, UiText::StatusResetFailed, "error", err),
    }
}

fn save_project_effect(state: &AppState) -> Effect {
    Effect::SaveProject {
        path: state.editor.project_path.clone(),
        project: ProjectData {
            name: state.project.name.clone(),
            params: state.domain.params.clone(),
            rules: state.domain.rules.clone(),
            export_profiles: state.project.export_profiles.clone(),
        },
    }
}

fn save_editor_config_effects(state: &AppState) -> Vec<Effect> {
    vec![Effect::SaveEditorConfig {
        data: EditorConfigData {
            config: state.editor_config(),
        },
    }]
}

fn set_project_name(state: &mut AppState, name: String) -> Vec<Effect> {
    let value = name.trim();
    if value.is_empty() {
        state.ui.status = tr(state, UiText::ErrorProjectNameEmpty);
        return Vec::new();
    }

    state.project.name = value.to_string();
    state.ui.status = tr1(
        state,
        UiText::StatusProjectNameUpdated,
        "name",
        &state.project.name,
    );
    Vec::new()
}

fn set_export_enabled(state: &mut AppState, index: usize, enabled: bool) -> Vec<Effect> {
    let locale = state.locale();
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => {
            profile.enabled = enabled;
            state.ui.status = if enabled {
                i18n::format1(
                    locale,
                    UiText::StatusExportTargetEnabled,
                    "name",
                    &profile.name,
                )
            } else {
                i18n::format1(
                    locale,
                    UiText::StatusExportTargetDisabled,
                    "name",
                    &profile.name,
                )
            };
        }
        None => {
            state.ui.status = i18n::format1(
                locale,
                UiText::StatusMissingExportTarget,
                "index",
                index + 1,
            );
        }
    }
    Vec::new()
}

fn set_export_output_path(
    state: &mut AppState,
    index: usize,
    path: std::path::PathBuf,
) -> Vec<Effect> {
    let locale = state.locale();
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => {
            profile.output_path = path;
            state.ui.status = i18n::format2(
                locale,
                UiText::StatusExportOutputUpdated,
                "name",
                &profile.name,
                "path",
                profile.output_path.display(),
            );
        }
        None => {
            state.ui.status = i18n::format1(
                locale,
                UiText::StatusMissingExportTarget,
                "index",
                index + 1,
            );
        }
    }
    Vec::new()
}

fn set_export_template_path(
    state: &mut AppState,
    index: usize,
    path: std::path::PathBuf,
) -> Vec<Effect> {
    let locale = state.locale();
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => match &mut profile.format {
            ExportFormat::Template { template_path } => {
                *template_path = path;
                state.ui.status = i18n::format2(
                    locale,
                    UiText::StatusExportTemplateUpdated,
                    "name",
                    &profile.name,
                    "path",
                    template_path.display(),
                );
            }
            ExportFormat::Alacritty => {
                state.ui.status = i18n::format1(
                    locale,
                    UiText::ErrorExportNoTemplatePath,
                    "name",
                    &profile.name,
                );
            }
        },
        None => {
            state.ui.status = i18n::format1(
                locale,
                UiText::StatusMissingExportTarget,
                "index",
                index + 1,
            );
        }
    }
    Vec::new()
}

fn set_editor_project_path(state: &mut AppState, path: std::path::PathBuf) -> Vec<Effect> {
    state.editor.project_path = path;
    state.ui.status = tr1(
        state,
        UiText::StatusProjectFilePathUpdated,
        "path",
        state.editor.project_path.display(),
    );
    save_editor_config_effects(state)
}

fn set_editor_auto_load_project(state: &mut AppState, enabled: bool) -> Vec<Effect> {
    state.editor.auto_load_project_on_startup = enabled;
    state.ui.status = if enabled {
        tr(state, UiText::StatusAutoLoadEnabled)
    } else {
        tr(state, UiText::StatusAutoLoadDisabled)
    };
    save_editor_config_effects(state)
}

fn set_editor_auto_save_on_export(state: &mut AppState, enabled: bool) -> Vec<Effect> {
    state.editor.auto_save_project_on_export = enabled;
    state.ui.status = if enabled {
        tr(state, UiText::StatusAutoSaveEnabled)
    } else {
        tr(state, UiText::StatusAutoSaveDisabled)
    };
    save_editor_config_effects(state)
}

fn set_editor_startup_focus(state: &mut AppState, focus: FocusPane) -> Vec<Effect> {
    state.editor.startup_focus = focus;
    state.ui.theme_panel = focus.into();
    if state.ui.active_tab == crate::app::workspace::WorkspaceTab::Theme {
        state.ui.interaction.focus_panel(state.ui.theme_panel);
    }
    state.ui.status = tr1(
        state,
        UiText::StatusStartupFocusUpdated,
        "focus",
        i18n::focus_pane_label(state.locale(), focus),
    );
    save_editor_config_effects(state)
}

fn set_editor_keymap_preset(state: &mut AppState, preset: EditorKeymapPreset) -> Vec<Effect> {
    state.editor.keymap_preset = preset;
    state.ui.status = tr1(
        state,
        UiText::StatusKeymapUpdated,
        "preset",
        i18n::keymap_preset_label(state.locale(), preset),
    );
    save_editor_config_effects(state)
}

fn set_editor_locale(
    state: &mut AppState,
    locale: crate::persistence::editor_config::EditorLocale,
) -> Vec<Effect> {
    state.editor.locale = locale;
    state.ui.status = tr1(
        state,
        UiText::StatusLanguageUpdated,
        "locale",
        i18n::locale_label(state.locale(), locale),
    );
    save_editor_config_effects(state)
}

fn effects_for_text_target(state: &AppState, target: TextInputTarget) -> Vec<Effect> {
    match target {
        TextInputTarget::Config(
            ConfigFieldId::EditorProjectPath
            | ConfigFieldId::EditorAutoLoadProject
            | ConfigFieldId::EditorAutoSaveOnExport
            | ConfigFieldId::EditorStartupFocus
            | ConfigFieldId::EditorKeymapPreset
            | ConfigFieldId::EditorLocale,
        ) => save_editor_config_effects(state),
        _ => Vec::new(),
    }
}

fn input_target_label(state: &AppState, target: TextInputTarget) -> String {
    match target {
        TextInputTarget::Control(control) => control.label().to_string(),
        TextInputTarget::Config(field) => match field {
            ConfigFieldId::ProjectName => tr(state, UiText::FieldProjectNameLower),
            ConfigFieldId::ExportEnabled(index) => {
                tr1(state, UiText::FieldExportTarget, "index", index + 1)
            }
            ConfigFieldId::ExportOutputPath(index) => {
                tr1(state, UiText::FieldExportOutputPath, "index", index + 1)
            }
            ConfigFieldId::ExportTemplatePath(index) => {
                tr1(state, UiText::FieldExportTemplatePath, "index", index + 1)
            }
            ConfigFieldId::EditorProjectPath => tr(state, UiText::FieldProjectFilePath),
            ConfigFieldId::EditorAutoLoadProject => tr(state, UiText::FieldAutoLoadProject),
            ConfigFieldId::EditorAutoSaveOnExport => tr(state, UiText::FieldAutoSaveProject),
            ConfigFieldId::EditorStartupFocus => tr(state, UiText::FieldStartupFocusLower),
            ConfigFieldId::EditorKeymapPreset => tr(state, UiText::FieldKeymapPresetLower),
            ConfigFieldId::EditorLocale => tr(state, UiText::FieldLanguageLower),
        },
    }
}

fn open_config_modal(state: &mut AppState) {
    close_source_picker_surface(state);
    close_text_input_surface(state);
    close_shortcut_help_surface(state);
    state.ui.config_modal = Some(ConfigModalState { selected_field: 0 });
    push_modal_owner(state, SurfaceId::ConfigDialog);
    focus_surface(state, SurfaceId::ConfigDialog);
    state.ui.status = tr(state, UiText::StatusConfigOpened);
}

fn close_config_modal(state: &mut AppState) {
    let was_open = state.ui.config_modal.is_some();
    close_text_input_surface(state);
    close_config_surface(state);
    if was_open {
        state.ui.status = tr(state, UiText::StatusConfigClosed);
    }
}

fn toggle_shortcut_help(state: &mut AppState) {
    let next = !state.ui.shortcut_help_open;
    if next {
        close_source_picker_surface(state);
        close_text_input_surface(state);
        close_config_surface(state);
        state.ui.shortcut_help_open = true;
        state.ui.shortcut_help_scroll = 0;
        push_modal_owner(state, SurfaceId::ShortcutHelp);
        focus_surface(state, SurfaceId::ShortcutHelp);
        state.ui.status = tr(state, UiText::StatusHelpOpened);
    } else {
        close_shortcut_help_surface(state);
        state.ui.status = tr(state, UiText::StatusHelpClosed);
    }
}

fn scroll_shortcut_help(state: &mut AppState, delta: i32) {
    if !state.ui.shortcut_help_open {
        return;
    }

    let next = if delta < 0 {
        state
            .ui
            .shortcut_help_scroll
            .saturating_sub(delta.unsigned_abs() as u16)
    } else {
        state.ui.shortcut_help_scroll.saturating_add(delta as u16)
    };
    state.ui.shortcut_help_scroll = next;
}

fn move_config_selection(state: &mut AppState, delta: i32) {
    let len = config_fields(state).len();
    if len == 0 {
        return;
    }

    if let Some(modal) = &mut state.ui.config_modal {
        modal.selected_field = cycle_index(modal.selected_field.min(len - 1), len, delta);
    }
}

fn activate_config_field(state: &mut AppState) -> Vec<Effect> {
    let field = if let Some(modal) = &state.ui.config_modal {
        let fields = config_fields(state);
        match fields
            .get(modal.selected_field.min(fields.len().saturating_sub(1)))
            .copied()
        {
            Some(field) => field,
            None => return Vec::new(),
        }
    } else {
        match state.active_config_field() {
            Some(field) => field,
            None => return Vec::new(),
        }
    };

    activate_config_field_by_id(state, field)
}

fn activate_config_field_by_id(state: &mut AppState, field: ConfigFieldId) -> Vec<Effect> {
    if field.supports_text_input() {
        open_text_input(state, TextInputTarget::Config(field));
        return Vec::new();
    }

    match field {
        ConfigFieldId::ExportEnabled(index) => {
            let locale = state.locale();
            if let Some(profile) = state.project.export_profiles.get_mut(index) {
                profile.enabled = !profile.enabled;
                state.ui.status = if profile.enabled {
                    i18n::format1(
                        locale,
                        UiText::StatusExportTargetEnabled,
                        "name",
                        &profile.name,
                    )
                } else {
                    i18n::format1(
                        locale,
                        UiText::StatusExportTargetDisabled,
                        "name",
                        &profile.name,
                    )
                };
            } else {
                state.ui.status = i18n::format1(
                    locale,
                    UiText::StatusMissingExportTarget,
                    "index",
                    index + 1,
                );
            }
        }
        ConfigFieldId::EditorAutoLoadProject => {
            return set_editor_auto_load_project(state, !state.editor.auto_load_project_on_startup);
        }
        ConfigFieldId::EditorAutoSaveOnExport => {
            return set_editor_auto_save_on_export(
                state,
                !state.editor.auto_save_project_on_export,
            );
        }
        ConfigFieldId::EditorStartupFocus => {
            return set_editor_startup_focus(state, state.editor.startup_focus.next());
        }
        ConfigFieldId::EditorKeymapPreset => {
            return set_editor_keymap_preset(state, state.editor.keymap_preset.next());
        }
        ConfigFieldId::EditorLocale => {
            return set_editor_locale(state, state.editor.locale.next());
        }
        ConfigFieldId::ProjectName
        | ConfigFieldId::ExportOutputPath(_)
        | ConfigFieldId::ExportTemplatePath(_)
        | ConfigFieldId::EditorProjectPath => {}
    }

    Vec::new()
}

fn apply_config_input(
    state: &mut AppState,
    field: ConfigFieldId,
    buffer: &str,
) -> Result<String, String> {
    let value = buffer.trim();
    if value.is_empty() {
        return Err(tr(state, UiText::ErrorInputEmpty));
    }

    let locale = state.locale();
    match field {
        ConfigFieldId::ProjectName => {
            state.project.name = value.to_string();
            Ok(tr1(
                state,
                UiText::StatusProjectNameUpdated,
                "name",
                &state.project.name,
            ))
        }
        ConfigFieldId::EditorProjectPath => {
            state.editor.project_path = value.into();
            Ok(tr1(
                state,
                UiText::StatusProjectFilePathUpdated,
                "path",
                state.editor.project_path.display(),
            ))
        }
        ConfigFieldId::EditorAutoLoadProject
        | ConfigFieldId::EditorAutoSaveOnExport
        | ConfigFieldId::EditorStartupFocus
        | ConfigFieldId::EditorKeymapPreset
        | ConfigFieldId::EditorLocale => Err(tr(state, UiText::ErrorUseToggleChoicePreference)),
        ConfigFieldId::ExportOutputPath(index) => {
            let profile = state
                .project
                .export_profiles
                .get_mut(index)
                .ok_or_else(|| {
                    i18n::format1(
                        locale,
                        UiText::StatusMissingExportTarget,
                        "index",
                        index + 1,
                    )
                })?;
            profile.output_path = value.into();
            Ok(i18n::format2(
                locale,
                UiText::StatusExportOutputUpdated,
                "name",
                &profile.name,
                "path",
                profile.output_path.display(),
            ))
        }
        ConfigFieldId::ExportTemplatePath(index) => {
            let profile = state
                .project
                .export_profiles
                .get_mut(index)
                .ok_or_else(|| {
                    i18n::format1(
                        locale,
                        UiText::StatusMissingExportTarget,
                        "index",
                        index + 1,
                    )
                })?;
            match &mut profile.format {
                ExportFormat::Template { template_path } => {
                    *template_path = value.into();
                    Ok(i18n::format2(
                        locale,
                        UiText::StatusExportTemplateUpdated,
                        "name",
                        &profile.name,
                        "path",
                        template_path.display(),
                    ))
                }
                ExportFormat::Alacritty => Err(i18n::format1(
                    locale,
                    UiText::ErrorExportNoTemplatePath,
                    "name",
                    &profile.name,
                )),
            }
        }
        ConfigFieldId::ExportEnabled(index) => Err(tr1(
            state,
            UiText::ErrorToggleExportTarget,
            "index",
            index + 1,
        )),
    }
}

pub fn filtered_source_options(picker: &SourcePickerState) -> Vec<SourceOption> {
    let filter = picker.filter.trim().to_ascii_lowercase();
    let role =
        role_for_reference_control(picker.control).expect("picker control should be reference");
    available_source_options(role)
        .into_iter()
        .filter(|option| {
            filter.is_empty()
                || option.source.label().to_ascii_lowercase().contains(&filter)
                || option.group.label().to_ascii_lowercase().contains(&filter)
        })
        .collect()
}

pub fn config_fields(state: &AppState) -> Vec<ConfigFieldId> {
    let mut fields = vec![ConfigFieldId::ProjectName];
    for (index, profile) in state.project.export_profiles.iter().enumerate() {
        fields.push(ConfigFieldId::ExportEnabled(index));
        fields.push(ConfigFieldId::ExportOutputPath(index));
        if matches!(&profile.format, ExportFormat::Template { .. }) {
            fields.push(ConfigFieldId::ExportTemplatePath(index));
        }
    }
    fields.push(ConfigFieldId::EditorProjectPath);
    fields.push(ConfigFieldId::EditorAutoLoadProject);
    fields.push(ConfigFieldId::EditorAutoSaveOnExport);
    fields.push(ConfigFieldId::EditorStartupFocus);
    fields.push(ConfigFieldId::EditorKeymapPreset);
    fields.push(ConfigFieldId::EditorLocale);
    fields
}

pub fn default_input_buffer(state: &AppState, target: TextInputTarget) -> String {
    match target {
        TextInputTarget::Control(control) => match control {
            ControlId::Param(key) => match key {
                ParamKey::BackgroundHue | ParamKey::AccentHue => {
                    format!("{:.1}", key.get(&state.domain.params))
                }
                _ => format!("{:.0}", key.get(&state.domain.params) * 100.0),
            },
            ControlId::MixRatio(role) => match state.domain.rules.get(role) {
                Some(Rule::Mix { ratio, .. }) => format!("{:.0}", ratio * 100.0),
                _ => String::new(),
            },
            ControlId::AdjustAmount(role) => match state.domain.rules.get(role) {
                Some(Rule::Adjust { amount, .. }) => format!("{:.0}", amount * 100.0),
                _ => String::new(),
            },
            ControlId::FixedColor(role) => match state.domain.rules.get(role) {
                Some(Rule::Fixed { color }) => color.to_hex(),
                _ => String::new(),
            },
            _ => String::new(),
        },
        TextInputTarget::Config(field) => match field {
            ConfigFieldId::ProjectName => state.project.name.clone(),
            ConfigFieldId::EditorProjectPath => state.editor.project_path.display().to_string(),
            ConfigFieldId::EditorAutoLoadProject
            | ConfigFieldId::EditorAutoSaveOnExport
            | ConfigFieldId::EditorStartupFocus
            | ConfigFieldId::EditorKeymapPreset
            | ConfigFieldId::EditorLocale => String::new(),
            ConfigFieldId::ExportOutputPath(index) => state
                .project
                .export_profiles
                .get(index)
                .map(|profile| profile.output_path.display().to_string())
                .unwrap_or_default(),
            ConfigFieldId::ExportTemplatePath(index) => state
                .project
                .export_profiles
                .get(index)
                .and_then(|profile| match &profile.format {
                    ExportFormat::Template { template_path } => {
                        Some(template_path.display().to_string())
                    }
                    ExportFormat::Alacritty => None,
                })
                .unwrap_or_default(),
            ConfigFieldId::ExportEnabled(_) => String::new(),
        },
    }
}

fn parse_param_input(state: &AppState, key: ParamKey, buffer: &str) -> Result<f32, String> {
    match key {
        ParamKey::BackgroundHue | ParamKey::AccentHue => parse_float_input(state, buffer),
        _ => parse_fraction_input(state, buffer),
    }
}

fn parse_float_input(state: &AppState, buffer: &str) -> Result<f32, String> {
    let trimmed = buffer.trim().trim_end_matches('%').trim();
    if trimmed.is_empty() {
        return Err(tr(state, UiText::ErrorInputEmpty));
    }
    trimmed
        .parse::<f32>()
        .map_err(|_| tr1(state, UiText::ErrorInvalidNumber, "value", buffer))
}

fn parse_fraction_input(state: &AppState, buffer: &str) -> Result<f32, String> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Err(tr(state, UiText::ErrorInputEmpty));
    }

    let is_percent = trimmed.ends_with('%');
    let number = trimmed
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .map_err(|_| tr1(state, UiText::ErrorInvalidNumber, "value", buffer))?;

    let value = if is_percent || number > 1.0 {
        number / 100.0
    } else {
        number
    };

    Ok(value.clamp(0.0, 1.0))
}

fn accepts_input_char(target: TextInputTarget, ch: char, existing: &str) -> bool {
    match target {
        TextInputTarget::Control(control) => match control {
            ControlId::Param(_) | ControlId::MixRatio(_) | ControlId::AdjustAmount(_) => {
                ch.is_ascii_digit() || ch == '.' || ch == '%'
            }
            ControlId::FixedColor(_) => {
                let hex_len = existing.trim_start_matches('#').len();
                (ch.is_ascii_hexdigit() && hex_len < 8)
                    || (ch == '#' && !existing.contains('#') && existing.is_empty())
            }
            _ => false,
        },
        TextInputTarget::Config(_) => !ch.is_control(),
    }
}

pub fn cycle_index(current: usize, len: usize, delta: i32) -> usize {
    let len = len as i32;
    ((current as i32 + delta).rem_euclid(len)) as usize
}

fn cycle_rule_kind(current: RuleKind, delta: i32) -> RuleKind {
    let index = RuleKind::ALL
        .iter()
        .position(|kind| *kind == current)
        .unwrap_or_default();
    RuleKind::ALL[cycle_index(index, RuleKind::ALL.len(), delta)]
}

fn cycle_source(current: &SourceRef, role: TokenRole, delta: i32) -> SourceRef {
    let options = available_sources(role);
    let index = options
        .iter()
        .position(|option| option == current)
        .unwrap_or_default();
    options[cycle_index(index, options.len(), delta)].clone()
}

fn cycle_adjust_op(current: AdjustOp, delta: i32) -> AdjustOp {
    let index = AdjustOp::ALL
        .iter()
        .position(|op| *op == current)
        .unwrap_or_default();
    AdjustOp::ALL[cycle_index(index, AdjustOp::ALL.len(), delta)]
}

fn cycle_fixed_color(current: Color, options: &[Color], delta: i32) -> Color {
    let index = options
        .iter()
        .position(|candidate| candidate.approx_eq(current))
        .unwrap_or_default();
    options[cycle_index(index, options.len(), delta)]
}

fn role_for_reference_control(control: ControlId) -> Option<TokenRole> {
    match control {
        ControlId::Reference(role, _) => Some(role),
        _ => None,
    }
}

pub fn current_source_for_control(control: ControlId, rule: &Rule) -> Option<SourceRef> {
    match (control, rule) {
        (ControlId::Reference(_, ReferenceField::AliasSource), Rule::Alias { source }) => {
            Some(source.clone())
        }
        (ControlId::Reference(_, ReferenceField::MixA), Rule::Mix { a, .. }) => Some(a.clone()),
        (ControlId::Reference(_, ReferenceField::MixB), Rule::Mix { b, .. }) => Some(b.clone()),
        (ControlId::Reference(_, ReferenceField::AdjustSource), Rule::Adjust { source, .. }) => {
            Some(source.clone())
        }
        _ => None,
    }
}

pub fn apply_source_to_control(control: ControlId, rule: &mut Rule, source: SourceRef) {
    match (control, rule) {
        (ControlId::Reference(_, ReferenceField::AliasSource), Rule::Alias { source: current }) => {
            *current = source
        }
        (ControlId::Reference(_, ReferenceField::MixA), Rule::Mix { a, .. }) => *a = source,
        (ControlId::Reference(_, ReferenceField::MixB), Rule::Mix { b, .. }) => *b = source,
        (
            ControlId::Reference(_, ReferenceField::AdjustSource),
            Rule::Adjust {
                source: current, ..
            },
        ) => *current = source,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::controls::ControlId;
    use crate::app::interaction::{InteractionMode, SurfaceId, effective_focus_path};
    use crate::app::view::interaction_panel_max_scroll;
    use crate::app::workspace::{PanelId, WorkspaceTab};
    use crate::domain::params::ParamKey;
    use crate::domain::preview::PreviewRuntimeEvent;
    use crate::domain::tokens::TokenRole;

    #[test]
    fn active_numeric_input_steps_and_syncs_buffer() {
        let mut state = AppState::new().expect("state should build");
        open_text_input(
            &mut state,
            TextInputTarget::Control(ControlId::Param(ParamKey::AccentHue)),
        );

        let effects = update(&mut state, Intent::AdjustActiveNumericInputByStep(1));

        assert!(effects.is_empty());
        assert!((state.domain.params.accent_hue - 210.0).abs() < f32::EPSILON);
        assert_eq!(
            state
                .ui
                .text_input
                .as_ref()
                .map(|input| input.buffer.as_str()),
            Some("210.0")
        );
    }

    #[test]
    fn workspace_tabs_restore_panel_focus() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Inspector);
        state.ui.interaction.focus_panel(PanelId::Inspector);

        update(&mut state, Intent::CycleWorkspaceTab(1));
        assert_eq!(state.ui.active_tab, WorkspaceTab::Project);
        assert_eq!(state.active_panel(), PanelId::ProjectConfig);

        update(&mut state, Intent::CycleWorkspaceTab(1));
        assert_eq!(state.ui.active_tab, WorkspaceTab::Theme);
        assert_eq!(state.active_panel(), PanelId::Inspector);
    }

    #[test]
    fn digit_navigation_focuses_visible_panel_in_current_tab() {
        let mut state = AppState::new().expect("state should build");

        update(&mut state, Intent::FocusPanelByNumber(6));
        assert_eq!(state.active_panel(), PanelId::Preview);

        update(&mut state, Intent::FocusPanelByNumber(8));
        assert_eq!(state.active_panel(), PanelId::InteractionInspector);
        assert_eq!(
            state.ui.interaction.focused_surface(),
            SurfaceId::InteractionInspectorPanel
        );

        update(&mut state, Intent::CycleWorkspaceTab(1));
        update(&mut state, Intent::FocusPanelByNumber(3));
        assert_eq!(state.active_panel(), PanelId::EditorPreferences);
    }

    #[test]
    fn interaction_inspector_panel_scrolls_instead_of_reporting_no_list_selection() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::InteractionInspector);
        state.ui.interaction.focus_panel(PanelId::InteractionInspector);

        let previous_status = state.ui.status.clone();
        update(&mut state, Intent::MoveSelection(1));

        assert!(state.ui.interaction_inspector_scroll > 0);
        assert_eq!(state.ui.status, previous_status);
    }

    #[test]
    fn interaction_inspector_scroll_is_bounded_by_content_length() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::InteractionInspector);
        state.ui.interaction.focus_panel(PanelId::InteractionInspector);

        for _ in 0..256 {
            update(&mut state, Intent::MoveSelection(1));
        }

        assert_eq!(
            state.ui.interaction_inspector_scroll,
            interaction_panel_max_scroll(&state)
        );
    }

    #[test]
    fn cycling_preview_mode_prepares_runtime_placeholder() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        update(&mut state, Intent::CyclePreviewMode(1));

        assert_eq!(
            state.preview.active_mode,
            crate::preview::PreviewMode::Shell
        );
        assert!(matches!(
            state.preview.runtime_frame,
            PreviewFrame::Placeholder(_)
        ));
        assert!(!state.preview.runtime_status.is_empty());
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
    }

    #[test]
    fn preview_capture_requires_interactive_mode() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        update(&mut state, Intent::SetPreviewCapture(true));
        assert!(!state.preview.capture_active);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);

        state.preview.active_mode = crate::preview::PreviewMode::Shell;
        update(&mut state, Intent::SetPreviewCapture(true));
        assert!(state.preview.capture_active);
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Capture {
                owner: SurfaceId::PreviewBody,
            }
        );
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::PreviewPanel,
                SurfaceId::PreviewBody,
            ]
        );

        update(&mut state, Intent::SetPreviewCapture(false));
        assert!(!state.preview.capture_active);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
    }

    #[test]
    fn modal_flows_push_and_pop_owned_stack_entries() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Params);
        state.ui.interaction.focus_panel(PanelId::Params);

        update(
            &mut state,
            Intent::ActivateControl(ControlId::Param(ParamKey::BackgroundHue)),
        );
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::NumericEditorSurface,
            }
        );
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::ParamsPanel,
                SurfaceId::NumericEditorSurface,
            ]
        );

        update(&mut state, Intent::CancelTextInput);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::ParamsPanel,
            ]
        );

        update(
            &mut state,
            Intent::ActivateControl(ControlId::Reference(
                TokenRole::Text,
                crate::app::controls::ReferenceField::AliasSource,
            )),
        );
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::SourcePicker,
            }
        );

        update(&mut state, Intent::CloseSourcePicker);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::ParamsPanel,
            ]
        );
    }

    #[test]
    fn closing_numeric_editor_restores_focus_to_owner_surface() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Params);
        state.ui.interaction.focus_panel(PanelId::Params);

        open_text_input(
            &mut state,
            TextInputTarget::Control(ControlId::Param(ParamKey::AccentHue)),
        );
        assert_eq!(
            state.ui.interaction.focus_path,
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::ParamsPanel,
                SurfaceId::NumericEditorSurface,
            ]
        );
        assert_eq!(
            state.ui.interaction.focused_surface(),
            SurfaceId::NumericEditorSurface
        );

        update(&mut state, Intent::CancelTextInput);

        assert_eq!(
            state.ui.interaction.focused_surface(),
            SurfaceId::ParamsPanel
        );
        assert_eq!(
            state.ui.interaction.focus_path,
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::ParamsPanel,
            ]
        );
    }

    #[test]
    fn config_and_help_flows_use_stack_owners() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Inspector);
        state.ui.interaction.focus_panel(PanelId::Inspector);

        update(&mut state, Intent::OpenConfigRequested);
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::ConfigDialog,
            }
        );
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::InspectorPanel,
                SurfaceId::ConfigDialog,
            ]
        );

        update(&mut state, Intent::ToggleShortcutHelpRequested);
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::ShortcutHelp,
            }
        );
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::InspectorPanel,
                SurfaceId::ShortcutHelp,
            ]
        );

        update(&mut state, Intent::ToggleShortcutHelpRequested);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);

        update(&mut state, Intent::OpenConfigRequested);
        update(&mut state, Intent::CloseConfigRequested);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
    }

    #[test]
    fn preview_runtime_exit_releases_capture_mode() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);
        state.preview.active_mode = crate::preview::PreviewMode::Shell;

        update(&mut state, Intent::SetPreviewCapture(true));
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Capture {
                owner: SurfaceId::PreviewBody,
            }
        );

        update(
            &mut state,
            Intent::PreviewRuntimeEvent(PreviewRuntimeEvent::Exited {
                message: "preview exited".to_string(),
            }),
        );
        assert!(!state.preview.capture_active);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
    }

    #[test]
    fn preview_runtime_exit_removes_capture_even_when_modal_is_on_top() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);
        state.preview.active_mode = crate::preview::PreviewMode::Shell;

        update(&mut state, Intent::SetPreviewCapture(true));
        update(&mut state, Intent::OpenConfigRequested);
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::ConfigDialog,
            }
        );

        update(
            &mut state,
            Intent::PreviewRuntimeEvent(PreviewRuntimeEvent::Exited {
                message: "preview exited".to_string(),
            }),
        );
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::ConfigDialog,
            }
        );

        update(&mut state, Intent::CloseConfigRequested);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::PreviewPanel,
            ]
        );
    }

    #[test]
    fn preview_runtime_exit_releases_capture_under_shortcut_help_modal() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);
        state.preview.active_mode = crate::preview::PreviewMode::Shell;

        update(&mut state, Intent::SetPreviewCapture(true));
        update(&mut state, Intent::ToggleShortcutHelpRequested);
        assert_eq!(
            state.ui.interaction.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::ShortcutHelp,
            }
        );

        update(
            &mut state,
            Intent::PreviewRuntimeEvent(PreviewRuntimeEvent::Exited {
                message: "preview exited".to_string(),
            }),
        );

        update(&mut state, Intent::ToggleShortcutHelpRequested);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::PreviewPanel,
            ]
        );
    }

    #[test]
    fn select_token_closes_transient_surfaces_without_leaving_stale_modes() {
        let mut text_input_state = AppState::new().expect("state should build");
        text_input_state.set_active_panel(PanelId::Params);
        text_input_state.ui.interaction.focus_panel(PanelId::Params);
        open_text_input(
            &mut text_input_state,
            TextInputTarget::Control(ControlId::Param(ParamKey::BackgroundHue)),
        );

        update(&mut text_input_state, Intent::SelectToken(1));
        assert!(text_input_state.ui.text_input.is_none());
        assert_eq!(
            text_input_state.ui.interaction.current_mode(),
            InteractionMode::Normal
        );
        assert_eq!(
            effective_focus_path(&text_input_state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::ParamsPanel,
            ]
        );

        let mut help_state = AppState::new().expect("state should build");
        update(&mut help_state, Intent::ToggleShortcutHelpRequested);

        update(&mut help_state, Intent::SelectToken(1));
        assert!(!help_state.ui.shortcut_help_open);
        assert_eq!(
            help_state.ui.interaction.current_mode(),
            InteractionMode::Normal
        );
        assert_eq!(
            effective_focus_path(&help_state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::TokensPanel,
            ]
        );

        let mut picker_state = AppState::new().expect("state should build");
        picker_state.set_active_panel(PanelId::Tokens);
        picker_state.ui.interaction.focus_panel(PanelId::Tokens);
        open_source_picker(
            &mut picker_state,
            ControlId::Reference(
                TokenRole::Text,
                crate::app::controls::ReferenceField::AliasSource,
            ),
        );

        update(&mut picker_state, Intent::SelectToken(1));
        assert!(picker_state.ui.source_picker.is_none());
        assert_eq!(
            picker_state.ui.interaction.current_mode(),
            InteractionMode::Normal
        );
        assert_eq!(
            effective_focus_path(&picker_state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::TokensPanel,
            ]
        );
    }

    #[test]
    fn move_selection_closes_source_picker_without_leaving_stale_mode() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);
        open_source_picker(
            &mut state,
            ControlId::Reference(
                TokenRole::Text,
                crate::app::controls::ReferenceField::AliasSource,
            ),
        );

        move_selection(&mut state, 1);
        assert!(state.ui.source_picker.is_none());
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::TokensPanel,
            ]
        );
    }

    #[test]
    fn reset_requested_clears_transient_owners_and_capture() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);
        state.preview.active_mode = crate::preview::PreviewMode::Shell;

        update(&mut state, Intent::SetPreviewCapture(true));
        update(&mut state, Intent::OpenConfigRequested);
        assert!(state.preview.capture_active);

        update(&mut state, Intent::ResetRequested);
        assert_eq!(state.ui.interaction.current_mode(), InteractionMode::Normal);
        assert!(!state.preview.capture_active);
        assert!(state.ui.text_input.is_none());
        assert!(state.ui.source_picker.is_none());
        assert!(state.ui.config_modal.is_none());
        assert!(!state.ui.shortcut_help_open);
        assert_eq!(
            effective_focus_path(&state),
            vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::PreviewPanel,
            ]
        );
    }
}
