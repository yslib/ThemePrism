use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use crate::app::actions::{BoundAction, matches_bound_action};
use crate::app::state::TextInputTarget;
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

        let preset = state.editor.keymap_preset;

        if state.ui.shortcut_help_open {
            return match_action(
                preset,
                &key,
                &[
                    BoundAction::OpenHelp,
                    BoundAction::Cancel,
                    BoundAction::MoveUp,
                    BoundAction::MoveDown,
                ],
            )
            .map(|action| match action {
                BoundAction::OpenHelp | BoundAction::Cancel => {
                    vec![Intent::ToggleShortcutHelpRequested]
                }
                BoundAction::MoveUp => vec![Intent::ScrollShortcutHelp(-1)],
                BoundAction::MoveDown => vec![Intent::ScrollShortcutHelp(1)],
                _ => Vec::new(),
            })
            .unwrap_or_default();
        }

        if state.ui.source_picker.is_some() {
            return match_action(
                preset,
                &key,
                &[
                    BoundAction::Cancel,
                    BoundAction::Apply,
                    BoundAction::MoveUp,
                    BoundAction::MoveDown,
                    BoundAction::Backspace,
                    BoundAction::Clear,
                ],
            )
            .map(|action| match action {
                BoundAction::Cancel => vec![Intent::CloseSourcePicker],
                BoundAction::Apply => vec![Intent::ApplySourcePickerSelection],
                BoundAction::MoveUp => vec![Intent::MoveSourcePickerSelection(-1)],
                BoundAction::MoveDown => vec![Intent::MoveSourcePickerSelection(1)],
                BoundAction::Backspace => vec![Intent::BackspaceSourcePickerFilter],
                BoundAction::Clear => vec![Intent::ClearSourcePickerFilter],
                _ => Vec::new(),
            })
            .unwrap_or_else(|| match key.code {
                KeyCode::Char(ch) if !ch.is_control() => vec![Intent::AppendSourcePickerFilter(ch)],
                _ => Vec::new(),
            });
        }

        if state.ui.text_input.is_some() {
            let active_numeric = match state.ui.text_input.as_ref().map(|input| input.target) {
                Some(TextInputTarget::Control(control)) if control.supports_numeric_editor() => {
                    Some(control)
                }
                _ => None,
            };
            return match_action(
                preset,
                &key,
                &[
                    BoundAction::Cancel,
                    BoundAction::Apply,
                    BoundAction::MoveLeft,
                    BoundAction::MoveRight,
                    BoundAction::Backspace,
                    BoundAction::Clear,
                ],
            )
            .map(|action| match action {
                BoundAction::Cancel => vec![Intent::CancelTextInput],
                BoundAction::Apply => vec![Intent::CommitTextInput],
                BoundAction::MoveLeft if active_numeric.is_some() => {
                    vec![Intent::AdjustActiveNumericInputByStep(-1)]
                }
                BoundAction::MoveRight if active_numeric.is_some() => {
                    vec![Intent::AdjustActiveNumericInputByStep(1)]
                }
                BoundAction::Backspace => vec![Intent::BackspaceTextInput],
                BoundAction::Clear => vec![Intent::ClearTextInput],
                _ => Vec::new(),
            })
            .unwrap_or_else(|| match key.code {
                KeyCode::Char(ch) if !ch.is_control() => vec![Intent::AppendTextInput(ch)],
                _ => Vec::new(),
            });
        }

        if state.ui.config_modal.is_some() {
            return match_action(
                preset,
                &key,
                &[
                    BoundAction::OpenHelp,
                    BoundAction::Cancel,
                    BoundAction::Activate,
                    BoundAction::Toggle,
                    BoundAction::MoveUp,
                    BoundAction::MoveDown,
                ],
            )
            .map(|action| match action {
                BoundAction::OpenHelp => vec![Intent::ToggleShortcutHelpRequested],
                BoundAction::Cancel => vec![Intent::CloseConfigRequested],
                BoundAction::Activate | BoundAction::Toggle => vec![Intent::ActivateConfigField],
                BoundAction::MoveUp => vec![Intent::MoveConfigSelection(-1)],
                BoundAction::MoveDown => vec![Intent::MoveConfigSelection(1)],
                _ => Vec::new(),
            })
            .unwrap_or_default();
        }

        if matches_bound_action(preset, BoundAction::OpenHelp, &key) {
            return vec![Intent::ToggleShortcutHelpRequested];
        }

        if let KeyCode::Char(ch) = key.code {
            if ('1'..='9').contains(&ch) {
                return vec![Intent::FocusPanelByNumber(ch as u8 - b'0')];
            }
        }

        match_action(
            preset,
            &key,
            &[
                BoundAction::Quit,
                BoundAction::PreviousTab,
                BoundAction::NextTab,
                BoundAction::PreviousPanel,
                BoundAction::NextPanel,
                BoundAction::OpenConfig,
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
            ],
        )
        .map(|action| match action {
            BoundAction::Quit => vec![Intent::QuitRequested],
            BoundAction::PreviousTab => vec![Intent::CycleWorkspaceTab(-1)],
            BoundAction::NextTab => vec![Intent::CycleWorkspaceTab(1)],
            BoundAction::PreviousPanel => vec![Intent::MoveFocus(-1)],
            BoundAction::NextPanel => vec![Intent::MoveFocus(1)],
            BoundAction::OpenConfig => vec![Intent::OpenConfigRequested],
            BoundAction::SaveProject => vec![Intent::SaveProjectRequested],
            BoundAction::LoadProject => vec![Intent::LoadProjectRequested],
            BoundAction::ExportTheme => vec![Intent::ExportThemeRequested],
            BoundAction::Reset => vec![Intent::ResetRequested],
            BoundAction::Activate if state.active_control().is_some() => {
                vec![Intent::ActivateControl(
                    state.active_control().expect("checked above"),
                )]
            }
            BoundAction::Activate | BoundAction::Toggle
                if state.active_config_field().is_some() =>
            {
                vec![Intent::ActivateConfigField]
            }
            BoundAction::MoveUp => vec![Intent::MoveSelection(-1)],
            BoundAction::MoveDown => vec![Intent::MoveSelection(1)],
            BoundAction::MoveLeft if state.active_control().is_some() => {
                vec![Intent::AdjustControlByStep(
                    state.active_control().expect("checked above"),
                    -1,
                )]
            }
            BoundAction::MoveRight if state.active_control().is_some() => {
                vec![Intent::AdjustControlByStep(
                    state.active_control().expect("checked above"),
                    1,
                )]
            }
            _ => Vec::new(),
        })
        .unwrap_or_default()
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
}
