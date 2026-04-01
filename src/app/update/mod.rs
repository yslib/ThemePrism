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
        Intent::SetEditorAutoLoadProject(enabled) => {
            config::set_editor_auto_load_project(state, enabled)
        }
        Intent::SetEditorAutoSaveOnExport(enabled) => {
            config::set_editor_auto_save_on_export(state, enabled)
        }
        Intent::SetEditorStartupFocus(focus) => config::set_editor_startup_focus(state, focus),
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
        Intent::ExportThemeRequested => {
            let mut effects = Vec::new();
            if state.editor.auto_save_project_on_export {
                effects.push(project::save_project_effect(state));
            }
            effects.push(Effect::ExportTheme {
                profiles: state.project.export_profiles.clone(),
                theme: state.domain.resolved.clone(),
            });
            effects
        }
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

pub fn cycle_index(current: usize, len: usize, delta: i32) -> usize {
    let len = len as i32;
    ((current as i32 + delta).rem_euclid(len)) as usize
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod behavior_tests {
    use super::*;
    use crate::app::controls::ControlId;
    use crate::app::interaction::{InteractionMode, SurfaceId, effective_focus_path};
    use crate::app::state::TextInputTarget;
    use crate::app::workspace::{PanelId, WorkspaceTab};
    use crate::domain::params::ParamKey;
    use crate::domain::preview::PreviewRuntimeEvent;
    use crate::domain::tokens::TokenRole;
    use crate::preview::PreviewFrame;

    #[test]
    fn active_numeric_input_steps_and_syncs_buffer() {
        let mut state = AppState::new().expect("state should build");
        text_input::open_text_input(
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
    fn set_workspace_tab_switches_directly_and_restores_last_panel_focus() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::Inspector);
        state.ui.interaction.focus_panel(PanelId::Inspector);

        update(&mut state, Intent::SetWorkspaceTab(WorkspaceTab::Project));
        assert_eq!(state.ui.active_tab, WorkspaceTab::Project);
        assert_eq!(state.active_panel(), PanelId::ProjectConfig);

        update(&mut state, Intent::SetWorkspaceTab(WorkspaceTab::Theme));
        assert_eq!(state.ui.active_tab, WorkspaceTab::Theme);
        assert_eq!(state.active_panel(), PanelId::Inspector);
        assert_eq!(
            state.ui.interaction.focused_surface(),
            SurfaceId::InspectorPanel
        );
    }

    #[test]
    fn interaction_inspector_panel_scrolls_instead_of_reporting_no_list_selection() {
        let mut state = AppState::new().expect("state should build");
        state.set_active_panel(PanelId::InteractionInspector);
        state
            .ui
            .interaction
            .focus_panel(PanelId::InteractionInspector);

        let previous_status = state.ui.status.clone();
        update(&mut state, Intent::MoveSelection(1));

        assert!(state.ui.interaction_inspector_scroll > 0);
        assert_eq!(state.ui.status, previous_status);
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
    fn toggle_fullscreen_targets_the_focused_surface() {
        let mut state = AppState::new().expect("state should build");
        state.ui.interaction.focus_path = vec![
            SurfaceId::AppRoot,
            SurfaceId::MainWindow,
            SurfaceId::PreviewPanel,
        ];

        update(&mut state, Intent::ToggleFullscreenRequested);

        assert_eq!(state.ui.fullscreen_surface, Some(SurfaceId::PreviewPanel));
    }

    #[test]
    fn toggling_fullscreen_twice_restores_normal_layout() {
        let mut state = AppState::new().expect("state should build");
        state.ui.fullscreen_surface = Some(SurfaceId::PreviewPanel);

        update(&mut state, Intent::ToggleFullscreenRequested);

        assert_eq!(state.ui.fullscreen_surface, None);
    }

    #[test]
    fn cycling_workspace_tabs_clears_fullscreen_for_hidden_panels() {
        let mut state = AppState::new().expect("state should build");
        state.ui.fullscreen_surface = Some(SurfaceId::PreviewPanel);

        update(&mut state, Intent::CycleWorkspaceTab(1));

        assert_eq!(state.ui.active_tab, WorkspaceTab::Project);
        assert_eq!(state.ui.fullscreen_surface, None);
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

        text_input::open_text_input(
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
        text_input::open_text_input(
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
        text_input::open_source_picker(
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
        text_input::open_source_picker(
            &mut state,
            ControlId::Reference(
                TokenRole::Text,
                crate::app::controls::ReferenceField::AliasSource,
            ),
        );

        navigation::move_selection(&mut state, 1);
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
