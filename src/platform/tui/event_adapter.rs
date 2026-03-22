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
        SurfaceId::TextInput => match_action(
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
        SurfaceId::MainWindow | SurfaceId::Panel(_) => {
            if state.ui.interaction.mode == InteractionMode::NavigateChildren(SurfaceId::MainWindow)
            {
                if let KeyCode::Char(ch) = key.code {
                    if ('1'..='9').contains(&ch) {
                        return Some(UiAction::SelectChild(ch as u8 - b'0'));
                    }
                }
            }

            match_action(
                preset,
                key,
                &[
                    BoundAction::Quit,
                    BoundAction::PreviousTab,
                    BoundAction::NextTab,
                    BoundAction::PreviousPanel,
                    BoundAction::NextPanel,
                    BoundAction::OpenConfig,
                    BoundAction::OpenHelp,
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
        BoundAction::PreviousTab => UiAction::PreviousTab,
        BoundAction::NextTab => UiAction::NextTab,
        BoundAction::PreviousPanel => UiAction::PreviousPanel,
        BoundAction::NextPanel => UiAction::NextPanel,
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
    use crate::app::{AppState, Intent};
    use crate::persistence::editor_config::EditorKeymapPreset;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })
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

        let previous = TuiEventAdapter.map_event(&state, key(KeyCode::Char('[')));
        let next = TuiEventAdapter.map_event(&state, key(KeyCode::Char(']')));

        assert!(matches!(
            previous.as_slice(),
            [Intent::CyclePreviewMode(-1)]
        ));
        assert!(matches!(next.as_slice(), [Intent::CyclePreviewMode(1)]));
    }

    #[test]
    fn preview_panel_enter_captures_preview() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);
        state.ui.interaction.focus_panel(PanelId::Preview);
        state.preview.active_mode = crate::preview::PreviewMode::Shell;

        let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));

        assert!(matches!(
            intents.as_slice(),
            [Intent::SetPreviewCapture(true)]
        ));
    }

    #[test]
    fn main_window_navigation_uses_digit_selection_after_activate() {
        let mut state = AppState::new().unwrap();
        state.ui.interaction.focus_root();

        let activate = TuiEventAdapter.map_event(&state, key(KeyCode::Enter));
        assert!(matches!(
            activate.as_slice(),
            [Intent::FocusSurface(_), Intent::SetInteractionMode(_)]
        ));

        state.ui.interaction.mode = crate::app::interaction::InteractionMode::NavigateChildren(
            crate::app::interaction::SurfaceId::MainWindow,
        );
        let select = TuiEventAdapter.map_event(&state, key(KeyCode::Char('2')));
        assert!(matches!(
            select.as_slice(),
            [Intent::FocusPanelByNumber(2), Intent::SetInteractionMode(_)]
        ));
    }
}
