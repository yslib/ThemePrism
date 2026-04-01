use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::actions::{BoundAction, matches_bound_action};
use crate::app::interaction::{
    InteractionMode, SurfaceId, UiAction, effective_focus_surface, route_ui_action,
};
use crate::app::{AppState, Intent};

#[derive(Debug, Default, Clone, Copy)]
pub struct TuiEventAdapter;

impl TuiEventAdapter {
    pub fn map_event(self, state: &AppState, event: Event) -> Vec<Intent> {
        let Event::Key(key) = event else {
            return Vec::new();
        };

        if key.kind != KeyEventKind::Press {
            return Vec::new();
        }

        let Some(action) = map_ui_action(state, &key) else {
            return Vec::new();
        };

        route_ui_action(state, action)
    }
}

fn map_ui_action(state: &AppState, key: &KeyEvent) -> Option<UiAction> {
    let preset = state.editor.keymap_preset;

    match effective_focus_surface(state) {
        SurfaceId::AppRoot => None,
        SurfaceId::ShortcutHelp => match_action(
            preset,
            key,
            &[
                BoundAction::OpenHelp,
                BoundAction::Cancel,
                BoundAction::MoveUp,
                BoundAction::MoveDown,
            ],
        )
        .map(bound_action_to_ui_action),
        SurfaceId::SourcePicker => match_action(
            preset,
            key,
            &[
                BoundAction::Cancel,
                BoundAction::Apply,
                BoundAction::MoveUp,
                BoundAction::MoveDown,
                BoundAction::Backspace,
                BoundAction::Clear,
            ],
        )
        .map(bound_action_to_ui_action)
        .or_else(|| free_text_action(key)),
        SurfaceId::NumericEditorSurface => match_action(
            preset,
            key,
            &[
                BoundAction::Cancel,
                BoundAction::Apply,
                BoundAction::MoveLeft,
                BoundAction::MoveRight,
                BoundAction::Backspace,
                BoundAction::Clear,
            ],
        )
        .map(bound_action_to_ui_action)
        .or_else(|| free_text_action(key)),
        SurfaceId::ConfigDialog => match_action(
            preset,
            key,
            &[
                BoundAction::OpenHelp,
                BoundAction::Cancel,
                BoundAction::Activate,
                BoundAction::Toggle,
                BoundAction::MoveUp,
                BoundAction::MoveDown,
            ],
        )
        .map(bound_action_to_ui_action),
        SurfaceId::CommandPalette => free_text_action(key).or_else(|| {
            match_action(
                preset,
                key,
                &[
                    BoundAction::Cancel,
                    BoundAction::Apply,
                    BoundAction::MoveUp,
                    BoundAction::MoveDown,
                    BoundAction::Backspace,
                    BoundAction::Clear,
                ],
            )
            .map(bound_action_to_ui_action)
        }),
        surface if surface.is_workspace_surface() => {
            if matches!(
                state.ui.interaction.current_mode(),
                InteractionMode::NavigateScope(_)
            ) {
                if let KeyCode::Char(ch) = key.code {
                    if ch.is_ascii_alphanumeric()
                        && !key.modifiers.intersects(
                            KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                        )
                    {
                        return Some(UiAction::NavigateTo(ch.to_ascii_lowercase()));
                    }
                }
            }
            if let KeyCode::Char(ch) = key.code {
                if ('1'..='9').contains(&ch)
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
                {
                    return Some(UiAction::NavigateTo(ch));
                }
            }

            match_action(
                preset,
                key,
                &[
                    BoundAction::OpenNavigation,
                    BoundAction::OpenCommandPalette,
                    BoundAction::Quit,
                    BoundAction::PreviousTab,
                    BoundAction::NextTab,
                    BoundAction::OpenConfig,
                    BoundAction::OpenHelp,
                    BoundAction::ToggleFullscreen,
                    BoundAction::SaveProject,
                    BoundAction::LoadProject,
                    BoundAction::ExportTheme,
                    BoundAction::Reset,
                    BoundAction::Activate,
                    BoundAction::Toggle,
                    BoundAction::MoveUp,
                    BoundAction::MoveDown,
                    BoundAction::MoveLeft,
                    BoundAction::MoveRight,
                    BoundAction::Cancel,
                ],
            )
            .map(bound_action_to_ui_action)
        }
        _ => None,
    }
}

fn free_text_action(key: &KeyEvent) -> Option<UiAction> {
    match key.code {
        KeyCode::Char(ch) if !ch.is_control() && !key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(UiAction::TypeChar(ch))
        }
        _ => None,
    }
}

fn bound_action_to_ui_action(action: BoundAction) -> UiAction {
    match action {
        BoundAction::OpenCommandPalette => UiAction::OpenCommandPalette,
        BoundAction::OpenNavigation => UiAction::OpenNavigation,
        BoundAction::PreviousTab => UiAction::PreviousTab,
        BoundAction::NextTab => UiAction::NextTab,
        BoundAction::MoveUp => UiAction::MoveUp,
        BoundAction::MoveDown => UiAction::MoveDown,
        BoundAction::MoveLeft => UiAction::MoveLeft,
        BoundAction::MoveRight => UiAction::MoveRight,
        BoundAction::Activate => UiAction::Activate,
        BoundAction::Toggle => UiAction::Toggle,
        BoundAction::Apply => UiAction::Apply,
        BoundAction::Cancel => UiAction::Cancel,
        BoundAction::Clear => UiAction::Clear,
        BoundAction::Backspace => UiAction::Backspace,
        BoundAction::OpenConfig => UiAction::OpenConfig,
        BoundAction::OpenHelp => UiAction::OpenHelp,
        BoundAction::ToggleFullscreen => UiAction::ToggleFullscreenRequested,
        BoundAction::SaveProject => UiAction::SaveProject,
        BoundAction::LoadProject => UiAction::LoadProject,
        BoundAction::ExportTheme => UiAction::ExportTheme,
        BoundAction::Reset => UiAction::Reset,
        BoundAction::Quit => UiAction::Quit,
        BoundAction::ReleasePreviewCapture => UiAction::Cancel,
    }
}

fn match_action(
    preset: crate::persistence::editor_config::EditorKeymapPreset,
    key: &KeyEvent,
    actions: &[BoundAction],
) -> Option<BoundAction> {
    actions
        .iter()
        .copied()
        .find(|action| matches_bound_action(preset, *action, key))
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use super::TuiEventAdapter;
    use crate::app::workspace::PanelId;
    use crate::app::{AppState, Intent, update};
    use crate::persistence::editor_config::EditorKeymapPreset;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })
    }

    fn ctrl_key(ch: char) -> Event {
        Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })
    }

    fn apply_intents(state: &mut AppState, intents: Vec<Intent>) {
        for intent in intents {
            update(state, intent);
        }
    }

    #[test]
    fn question_mark_opens_shortcut_help() {
        let state = AppState::new().unwrap();
        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('?')));
        assert!(matches!(
            intents.as_slice(),
            [Intent::ToggleShortcutHelpRequested]
        ));
    }

    #[test]
    fn ctrl_p_opens_command_palette() {
        let state = AppState::new().unwrap();
        let intents = TuiEventAdapter.map_event(&state, ctrl_key('p'));

        assert!(matches!(
            intents.as_slice(),
            [Intent::OpenCommandPaletteRequested]
        ));
    }

    #[test]
    fn palette_text_input_consumes_printable_keys_without_leaking_workspace_shortcuts() {
        let mut state = AppState::new().unwrap();
        update(&mut state, Intent::OpenCommandPaletteRequested);

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('e')));

        assert!(matches!(
            intents.as_slice(),
            [Intent::AppendCommandPaletteQuery('e')]
        ));
    }

    #[test]
    fn palette_text_input_uses_printable_chars_under_vim_preset() {
        let mut state = AppState::new().unwrap();
        state.editor.keymap_preset = EditorKeymapPreset::Vim;
        update(&mut state, Intent::OpenCommandPaletteRequested);

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('j')));

        assert!(matches!(
            intents.as_slice(),
            [Intent::AppendCommandPaletteQuery('j')]
        ));
    }

    #[test]
    fn command_palette_maps_enter_and_escape_to_run_and_close() {
        let mut state = AppState::new().unwrap();
        update(&mut state, Intent::OpenCommandPaletteRequested);

        let run = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        let close = TuiEventAdapter.map_event(&state, key(KeyCode::Esc));

        assert!(matches!(
            run.as_slice(),
            [Intent::RunSelectedCommandPaletteItem]
        ));
        assert!(matches!(
            close.as_slice(),
            [Intent::CloseCommandPaletteRequested]
        ));
    }

    #[test]
    fn vim_preset_maps_j_to_move_selection() {
        let mut state = AppState::new().unwrap();
        state.editor.keymap_preset = EditorKeymapPreset::Vim;
        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('j')));
        assert!(matches!(intents.as_slice(), [Intent::MoveSelection(1)]));
    }

    #[test]
    fn standard_preset_does_not_map_j_to_move_selection() {
        let state = AppState::new().unwrap();
        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('j')));
        assert!(intents.is_empty());
    }

    #[test]
    fn preview_panel_bracket_shortcuts_cycle_modes() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        let activate = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        apply_intents(&mut state, activate);

        let previous = TuiEventAdapter.map_event(&state, key(KeyCode::Char('[')));
        let next = TuiEventAdapter.map_event(&state, key(KeyCode::Char(']')));

        assert!(matches!(
            previous.as_slice(),
            [Intent::CyclePreviewMode(-1)]
        ));
        assert!(matches!(next.as_slice(), [Intent::CyclePreviewMode(1)]));
    }

    #[test]
    fn preview_panel_enter_focuses_preview_tabs() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));

        assert!(matches!(
            intents.as_slice(),
            [
                Intent::FocusSurface(crate::app::interaction::SurfaceId::PreviewTabs),
                Intent::SetInteractionMode(
                    crate::app::interaction::InteractionMode::NavigateChildren(
                        crate::app::interaction::SurfaceId::PreviewPanel
                    )
                )
            ]
        ));
    }

    #[test]
    fn preview_child_navigation_can_reach_body_and_capture_preview() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);
        state.preview.active_mode = crate::preview::PreviewMode::Shell;

        let activate = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        apply_intents(&mut state, activate);
        assert_eq!(
            state.ui.interaction.focus_path,
            vec![
                crate::app::interaction::SurfaceId::AppRoot,
                crate::app::interaction::SurfaceId::MainWindow,
                crate::app::interaction::SurfaceId::PreviewPanel,
                crate::app::interaction::SurfaceId::PreviewTabs,
            ]
        );
        assert_eq!(
            state.ui.interaction.current_mode(),
            crate::app::interaction::InteractionMode::NavigateChildren(
                crate::app::interaction::SurfaceId::PreviewPanel,
            )
        );
        let move_to_body = TuiEventAdapter.map_event(&state, key(KeyCode::Right));
        apply_intents(&mut state, move_to_body);
        assert_eq!(
            state.ui.interaction.focus_path,
            vec![
                crate::app::interaction::SurfaceId::AppRoot,
                crate::app::interaction::SurfaceId::MainWindow,
                crate::app::interaction::SurfaceId::PreviewPanel,
                crate::app::interaction::SurfaceId::PreviewBody,
            ]
        );

        let capture = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));

        assert!(matches!(
            capture.as_slice(),
            [Intent::SetPreviewCapture(true)]
        ));
    }

    #[test]
    fn preview_body_bracket_shortcut_cycles_preview_mode() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.preview.active_mode = crate::preview::PreviewMode::Shell;
        state.ui.interaction.focus_path = vec![
            crate::app::interaction::SurfaceId::AppRoot,
            crate::app::interaction::SurfaceId::MainWindow,
            crate::app::interaction::SurfaceId::PreviewPanel,
            crate::app::interaction::SurfaceId::PreviewBody,
        ];

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char(']')));

        assert!(matches!(intents.as_slice(), [Intent::CyclePreviewMode(1)]));
    }

    #[test]
    fn preview_child_navigation_uses_down_to_reach_body() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        let activate = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        apply_intents(&mut state, activate);

        let move_to_body = TuiEventAdapter.map_event(&state, key(KeyCode::Down));
        assert!(matches!(
            move_to_body.as_slice(),
            [Intent::FocusSurface(
                crate::app::interaction::SurfaceId::PreviewBody
            )]
        ));
    }

    #[test]
    fn preview_child_navigation_edges_do_not_fall_back_to_panel_selection() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        let activate = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        apply_intents(&mut state, activate);

        let up = TuiEventAdapter.map_event(&state, key(KeyCode::Up));
        assert!(up.is_empty());
    }

    #[test]
    fn preview_child_navigation_uses_vertical_keys_without_falling_back_to_move_selection() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        let activate = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        apply_intents(&mut state, activate);

        let down = TuiEventAdapter.map_event(&state, key(KeyCode::Down));
        assert!(matches!(
            down.as_slice(),
            [Intent::FocusSurface(
                crate::app::interaction::SurfaceId::PreviewBody
            )]
        ));
        apply_intents(&mut state, down);

        let up = TuiEventAdapter.map_event(&state, key(KeyCode::Up));
        assert!(matches!(
            up.as_slice(),
            [Intent::FocusSurface(
                crate::app::interaction::SurfaceId::PreviewTabs
            )]
        ));
    }

    #[test]
    fn vim_keys_move_between_preview_children_during_navigation() {
        let mut state = AppState::new().unwrap();
        state.editor.keymap_preset = EditorKeymapPreset::Vim;
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);

        let activate = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        apply_intents(&mut state, activate);

        let down = TuiEventAdapter.map_event(&state, key(KeyCode::Char('j')));
        assert!(matches!(
            down.as_slice(),
            [Intent::FocusSurface(
                crate::app::interaction::SurfaceId::PreviewBody
            )]
        ));
        apply_intents(&mut state, down);

        let up = TuiEventAdapter.map_event(&state, key(KeyCode::Char('k')));
        assert!(matches!(
            up.as_slice(),
            [Intent::FocusSurface(
                crate::app::interaction::SurfaceId::PreviewTabs
            )]
        ));
    }

    #[test]
    fn tab_on_regular_panel_bubbles_to_workspace_tab_switch() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char(']')));

        assert!(matches!(intents.as_slice(), [Intent::CycleWorkspaceTab(1)]));
    }

    #[test]
    fn tab_key_no_longer_cycles_panels() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Tab));

        assert!(intents.is_empty());
    }

    #[test]
    fn g_opens_main_window_navigation_mode() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('g')));

        assert!(matches!(
            intents.as_slice(),
            [Intent::SetInteractionMode(
                crate::app::interaction::InteractionMode::NavigateScope(
                    crate::app::interaction::SurfaceId::MainWindow
                )
            )]
        ));
    }

    #[test]
    fn digits_switch_panels_without_opening_navigation_mode() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);

        let jump = TuiEventAdapter.map_event(&state, key(KeyCode::Char('2')));
        apply_intents(&mut state, jump);

        assert_eq!(state.active_panel(), PanelId::Params);
        assert_eq!(
            state.ui.interaction.current_mode(),
            crate::app::interaction::InteractionMode::Normal
        );
    }

    #[test]
    fn main_window_navigation_letter_switches_workspace_tab_immediately() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);

        let open_navigation = TuiEventAdapter.map_event(&state, key(KeyCode::Char('g')));
        apply_intents(&mut state, open_navigation);
        let jump = TuiEventAdapter.map_event(&state, key(KeyCode::Char('b')));
        apply_intents(&mut state, jump);

        assert_eq!(
            state.ui.active_tab,
            crate::app::workspace::WorkspaceTab::Project
        );
        assert_eq!(state.active_panel(), PanelId::ProjectConfig);
        assert_eq!(
            state.ui.interaction.current_mode(),
            crate::app::interaction::InteractionMode::Normal
        );
    }

    #[test]
    fn hint_navigation_can_jump_directly_to_preview_tabs() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);

        let open_navigation = TuiEventAdapter.map_event(&state, key(KeyCode::Char('g')));
        apply_intents(&mut state, open_navigation);
        let jump = TuiEventAdapter.map_event(&state, key(KeyCode::Char('d')));
        apply_intents(&mut state, jump);

        assert_eq!(state.active_panel(), PanelId::Preview);
        assert_eq!(
            state.preview.active_mode,
            crate::preview::PreviewMode::Shell
        );
        assert_eq!(
            state.ui.interaction.focused_surface(),
            crate::app::interaction::SurfaceId::PreviewTabs
        );
        assert_eq!(
            state.ui.interaction.current_mode(),
            crate::app::interaction::InteractionMode::Normal
        );
    }

    #[test]
    fn escape_cancels_navigation_mode_without_moving_focus() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Inspector);
        state.ui.interaction.focus_panel(PanelId::Inspector);

        let open_navigation = TuiEventAdapter.map_event(&state, key(KeyCode::Char('g')));
        apply_intents(&mut state, open_navigation);
        let cancel = TuiEventAdapter.map_event(&state, key(KeyCode::Esc));
        apply_intents(&mut state, cancel);

        assert_eq!(state.active_panel(), PanelId::Inspector);
        assert_eq!(
            state.ui.interaction.focused_surface(),
            crate::app::interaction::SurfaceId::InspectorPanel
        );
        assert_eq!(
            state.ui.interaction.current_mode(),
            crate::app::interaction::InteractionMode::Normal
        );
    }

    #[test]
    fn letters_do_not_switch_tabs_without_navigation_mode() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Tokens);
        state.ui.interaction.focus_panel(PanelId::Tokens);

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('b')));

        assert!(intents.is_empty());
        assert_eq!(
            state.ui.active_tab,
            crate::app::workspace::WorkspaceTab::Theme
        );
    }
}
