mod command_palette;
mod config;
mod inspector;
mod modals;
mod navigation;
mod preview;
mod project;
mod text_input;

use crate::app::effect::Effect;
use crate::app::intent::Intent;
use crate::app::state::AppState;
use crate::i18n::{self, UiText};

pub use config::config_fields;
pub use inspector::current_source_for_control;
pub use text_input::{default_input_buffer, filtered_source_options};

pub fn update(state: &mut AppState, intent: Intent) -> Vec<Effect> {
    match intent {
        Intent::QuitRequested => {
            state.ui.should_quit = true;
            Vec::new()
        }
        Intent::CycleWorkspaceTab(delta) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            navigation::cycle_workspace_tab(state, delta);
            Vec::new()
        }
        Intent::SetWorkspaceTab(tab) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            navigation::set_workspace_tab(state, tab);
            Vec::new()
        }
        Intent::FocusPanelByNumber(number) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            navigation::focus_panel_by_number(state, number);
            Vec::new()
        }
        Intent::FocusSurface(surface) => {
            navigation::focus_surface(state, surface);
            Vec::new()
        }
        Intent::SetInteractionMode(mode) => {
            navigation::set_interaction_mode(state, mode);
            Vec::new()
        }
        Intent::MoveSelection(delta) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            navigation::move_selection(state, delta);
            Vec::new()
        }
        Intent::SelectToken(index) => {
            navigation::select_token(state, index);
            Vec::new()
        }
        Intent::AdjustControlByStep(control, delta) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            inspector::adjust_control_by_step(state, control, delta);
            Vec::new()
        }
        Intent::ActivateControl(control) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            inspector::activate_control(state, control);
            Vec::new()
        }
        Intent::AdjustActiveNumericInputByStep(delta) => {
            text_input::adjust_active_numeric_input(state, delta);
            Vec::new()
        }
        Intent::CyclePreviewMode(delta) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            preview::cycle_preview_mode(state, delta);
            Vec::new()
        }
        Intent::SetPreviewMode(mode) => {
            if transient_surface_open(state) {
                return Vec::new();
            }
            preview::set_preview_mode(state, mode);
            Vec::new()
        }
        Intent::SetPreviewCapture(active) => {
            preview::set_preview_capture(state, active);
            Vec::new()
        }
        Intent::ToggleFullscreenRequested => {
            navigation::toggle_fullscreen(state);
            Vec::new()
        }
        Intent::SetParamValue(key, value) => {
            inspector::set_param_value(state, key, value);
            Vec::new()
        }
        Intent::SetRuleKind(role, kind) => {
            inspector::set_rule_kind_for_role(state, role, kind);
            Vec::new()
        }
        Intent::SetReferenceSource(control, source) => {
            inspector::set_reference_source(state, control, source);
            Vec::new()
        }
        Intent::SetMixRatio(role, ratio) => {
            inspector::set_mix_ratio(state, role, ratio);
            Vec::new()
        }
        Intent::SetAdjustOp(role, op) => {
            inspector::set_adjust_op(state, role, op);
            Vec::new()
        }
        Intent::SetAdjustAmount(role, amount) => {
            inspector::set_adjust_amount(state, role, amount);
            Vec::new()
        }
        Intent::SetFixedColor(role, color) => {
            inspector::set_fixed_color(state, role, color);
            Vec::new()
        }
        Intent::SetProjectName(name) => project::set_project_name(state, name),
        Intent::SetExportEnabled(index, enabled) => {
            project::set_export_enabled(state, index, enabled)
        }
        Intent::SetExportOutputPath(index, path) => {
            project::set_export_output_path(state, index, path)
        }
        Intent::SetExportTemplatePath(index, path) => {
            project::set_export_template_path(state, index, path)
        }
        Intent::SetEditorProjectPath(path) => config::set_editor_project_path(state, path),
        Intent::SetEditorKeymapPreset(preset) => config::set_editor_keymap_preset(state, preset),
        Intent::SetEditorLocale(locale) => config::set_editor_locale(state, locale),
        Intent::AppendTextInput(ch) => {
            text_input::append_text_input(state, ch);
            Vec::new()
        }
        Intent::BackspaceTextInput => {
            text_input::backspace_text_input(state);
            Vec::new()
        }
        Intent::ClearTextInput => {
            text_input::clear_text_input(state);
            Vec::new()
        }
        Intent::CommitTextInput => text_input::commit_text_input(state),
        Intent::CancelTextInput => {
            modals::close_text_input_surface(state);
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
            text_input::move_source_picker_selection(state, delta);
            Vec::new()
        }
        Intent::ApplySourcePickerSelection => {
            text_input::apply_source_picker_selection(state);
            Vec::new()
        }
        Intent::CloseSourcePicker => {
            modals::close_source_picker_surface(state);
            state.ui.status = tr(state, UiText::StatusSourcePickerClosed);
            Vec::new()
        }
        Intent::OpenConfigRequested => {
            modals::open_config_modal(state);
            Vec::new()
        }
        Intent::CloseConfigRequested => {
            modals::close_config_modal(state);
            Vec::new()
        }
        Intent::OpenCommandPaletteRequested => {
            command_palette::open_command_palette(state);
            Vec::new()
        }
        Intent::CloseCommandPaletteRequested => {
            command_palette::close_command_palette(state);
            Vec::new()
        }
        Intent::SetCommandPaletteQuery(query) => {
            command_palette::set_query(state, query);
            Vec::new()
        }
        Intent::AppendCommandPaletteQuery(ch) => {
            command_palette::append_query(state, ch);
            Vec::new()
        }
        Intent::BackspaceCommandPaletteQuery => {
            command_palette::backspace_query(state);
            Vec::new()
        }
        Intent::ClearCommandPaletteQuery => {
            command_palette::clear_query(state);
            Vec::new()
        }
        Intent::MoveCommandPaletteSelection(delta) => {
            command_palette::move_selection(state, delta);
            Vec::new()
        }
        Intent::RunSelectedCommandPaletteItem => command_palette::run_selected(state),
        Intent::ToggleShortcutHelpRequested => {
            modals::toggle_shortcut_help(state);
            Vec::new()
        }
        Intent::ScrollShortcutHelp(delta) => {
            modals::scroll_shortcut_help(state, delta);
            Vec::new()
        }
        Intent::MoveConfigSelection(delta) => {
            config::move_config_selection(state, delta);
            Vec::new()
        }
        Intent::ActivateConfigField => config::activate_config_field(state),
        Intent::SaveProjectRequested => vec![project::save_project_effect(state)],
        Intent::LoadProjectRequested => vec![Effect::LoadProject {
            path: state.editor.project_path.clone(),
        }],
        Intent::ExportThemeRequested => vec![Effect::ExportTheme {
            project_name: state.project.name.clone(),
            params: state.domain.params.clone(),
            profiles: state.project.export_profiles.clone(),
            theme: state.domain.resolved.clone(),
        }],
        Intent::ResetRequested => {
            project::reset_state(state);
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
            preview::apply_preview_runtime_event(state, event);
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

fn transient_surface_open(state: &AppState) -> bool {
    state.ui.source_picker.is_some()
        || state.ui.text_input.is_some()
        || state.ui.config_modal.is_some()
        || state.ui.command_palette.is_some()
        || state.ui.shortcut_help_open
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

fn cycle_index(current: usize, len: usize, delta: i32) -> usize {
    let len = len as i32;
    ((current as i32 + delta).rem_euclid(len)) as usize
}

#[cfg(test)]
mod tests;
