use super::{navigation, text_input, update};
use crate::app::controls::ControlId;
use crate::app::intent::Intent;
use crate::app::interaction::{
    effective_focus_path, effective_focus_surface, InteractionMode, SurfaceId,
};
use crate::app::state::{
    AppState, ConfigFieldId, ConfigModalState, TextInputState, TextInputTarget,
};
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::domain::params::ParamKey;
use crate::domain::preview::PreviewRuntimeEvent;
use crate::domain::rules::{Rule, RuleKind};
use crate::domain::tokens::TokenRole;
use crate::preview::{PreviewFrame, PreviewMode};

#[test]
fn navigation_intent_routes_through_public_update_entrypoint() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::FocusPanelByNumber(2));

    assert_eq!(state.active_panel(), PanelId::Params);
}

#[test]
fn preview_intent_updates_active_preview_mode() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::SetPreviewMode(PreviewMode::Shell));

    assert_eq!(state.preview.active_mode, PreviewMode::Shell);
}

#[test]
fn modal_intent_toggles_shortcut_help_state() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::ToggleShortcutHelpRequested);

    assert!(state.ui.shortcut_help_open);
}

#[test]
fn open_command_palette_initializes_query_and_selection() {
    let mut state = AppState::new().unwrap();

    update(&mut state, Intent::OpenCommandPaletteRequested);

    let palette = state.ui.command_palette.as_ref().unwrap();
    assert!(palette.query.is_empty());
    assert_eq!(palette.selected, 0);
}

#[test]
fn closing_command_palette_restores_prior_focus() {
    let mut state = AppState::new().unwrap();
    state.set_active_panel(PanelId::Preview);
    state.ui.interaction.focus_panel(PanelId::Preview);

    update(&mut state, Intent::OpenCommandPaletteRequested);
    update(&mut state, Intent::CloseCommandPaletteRequested);

    assert_eq!(effective_focus_surface(&state), SurfaceId::PreviewPanel);
}

#[test]
fn running_selected_palette_command_dispatches_existing_command_path() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);
    update(&mut state, Intent::SetCommandPaletteQuery("expo".into()));

    let effects = update(&mut state, Intent::RunSelectedCommandPaletteItem);

    assert!(effects
        .iter()
        .any(|effect| matches!(effect, crate::app::effect::Effect::ExportTheme { .. })));
}

#[test]
fn running_palette_with_no_matches_keeps_palette_open_and_query_intact() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);
    update(
        &mut state,
        Intent::SetCommandPaletteQuery("no-such-command".into()),
    );

    let effects = update(&mut state, Intent::RunSelectedCommandPaletteItem);

    assert!(effects.is_empty());
    let palette = state.ui.command_palette.as_ref().unwrap();
    assert_eq!(palette.query, "no-such-command");
    assert_eq!(palette.selected, 0);
}

#[test]
fn command_palette_query_edits_reset_selection_and_clamp_matches() {
    let mut state = AppState::new().unwrap();

    update(&mut state, Intent::OpenCommandPaletteRequested);
    update(&mut state, Intent::AppendCommandPaletteQuery('e'));
    update(&mut state, Intent::AppendCommandPaletteQuery('x'));
    update(&mut state, Intent::MoveCommandPaletteSelection(1));
    assert_eq!(state.ui.command_palette.as_ref().unwrap().selected, 1);

    update(&mut state, Intent::BackspaceCommandPaletteQuery);
    let palette = state.ui.command_palette.as_ref().unwrap();
    assert_eq!(palette.query, "e");
    assert_eq!(palette.selected, 0);

    update(&mut state, Intent::ClearCommandPaletteQuery);
    let palette = state.ui.command_palette.as_ref().unwrap();
    assert!(palette.query.is_empty());
    assert_eq!(palette.selected, 0);
}

#[test]
fn config_intents_open_modal_and_move_selection() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::OpenConfigRequested);
    update(&mut state, Intent::MoveConfigSelection(1));

    assert_eq!(
        state.ui.config_modal,
        Some(ConfigModalState { selected_field: 1 })
    );
}

#[test]
fn project_intent_updates_project_name() {
    let mut state = AppState::new().expect("state should build");

    update(
        &mut state,
        Intent::SetProjectName("Aurora Theme".to_string()),
    );

    assert_eq!(state.project.name, "Aurora Theme");
}

#[test]
fn inspector_intent_updates_rule_kind_for_selected_role() {
    let mut state = AppState::new().expect("state should build");

    update(
        &mut state,
        Intent::SetRuleKind(TokenRole::Background, RuleKind::Fixed),
    );

    assert!(matches!(
        state.domain.rules.get(TokenRole::Background),
        Some(Rule::Fixed { .. })
    ));
}

#[test]
fn text_input_intent_appends_to_seeded_buffer() {
    let mut state = AppState::new().expect("state should build");
    state.ui.text_input = Some(TextInputState {
        target: TextInputTarget::Config(ConfigFieldId::ProjectName),
        buffer: String::new(),
    });

    update(&mut state, Intent::AppendTextInput('a'));

    assert_eq!(
        state
            .ui
            .text_input
            .as_ref()
            .map(|input| input.buffer.as_str()),
        Some("a")
    );
}

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
