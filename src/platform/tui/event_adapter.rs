use crossterm::event::{Event, KeyCode, KeyEventKind};

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

        if state.ui.source_picker.is_some() {
            return match key.code {
                KeyCode::Esc => vec![Intent::CloseSourcePicker],
                KeyCode::Enter => vec![Intent::ApplySourcePickerSelection],
                KeyCode::Up | KeyCode::Char('k') => vec![Intent::MoveSourcePickerSelection(-1)],
                KeyCode::Down | KeyCode::Char('j') => vec![Intent::MoveSourcePickerSelection(1)],
                KeyCode::Backspace => vec![Intent::BackspaceSourcePickerFilter],
                KeyCode::Delete => vec![Intent::ClearSourcePickerFilter],
                KeyCode::Char(ch) if !ch.is_control() => vec![Intent::AppendSourcePickerFilter(ch)],
                _ => Vec::new(),
            };
        }

        if state.ui.text_input.is_some() {
            return match key.code {
                KeyCode::Esc => vec![Intent::CancelTextInput],
                KeyCode::Enter => vec![Intent::CommitTextInput],
                KeyCode::Backspace => vec![Intent::BackspaceTextInput],
                KeyCode::Delete => vec![Intent::ClearTextInput],
                KeyCode::Char(ch) if !ch.is_control() => vec![Intent::AppendTextInput(ch)],
                _ => Vec::new(),
            };
        }

        let Some(control) = state.active_control() else {
            return match key.code {
                KeyCode::Char('q') => vec![Intent::QuitRequested],
                KeyCode::Tab => vec![Intent::MoveFocus(1)],
                KeyCode::BackTab => vec![Intent::MoveFocus(-1)],
                KeyCode::Char('s') => vec![Intent::SaveProjectRequested],
                KeyCode::Char('o') => vec![Intent::LoadProjectRequested],
                KeyCode::Char('e') => vec![Intent::ExportThemeRequested],
                KeyCode::Char('r') => vec![Intent::ResetRequested],
                KeyCode::Up | KeyCode::Char('k') => vec![Intent::MoveSelection(-1)],
                KeyCode::Down | KeyCode::Char('j') => vec![Intent::MoveSelection(1)],
                _ => Vec::new(),
            };
        };

        match key.code {
            KeyCode::Char('q') => vec![Intent::QuitRequested],
            KeyCode::Tab => vec![Intent::MoveFocus(1)],
            KeyCode::BackTab => vec![Intent::MoveFocus(-1)],
            KeyCode::Char('s') => vec![Intent::SaveProjectRequested],
            KeyCode::Char('o') => vec![Intent::LoadProjectRequested],
            KeyCode::Char('e') => vec![Intent::ExportThemeRequested],
            KeyCode::Char('r') => vec![Intent::ResetRequested],
            KeyCode::Enter | KeyCode::Char('i') => vec![Intent::ActivateControl(control)],
            KeyCode::Up | KeyCode::Char('k') => vec![Intent::MoveSelection(-1)],
            KeyCode::Down | KeyCode::Char('j') => vec![Intent::MoveSelection(1)],
            KeyCode::Left | KeyCode::Char('h') => vec![Intent::AdjustControlByStep(control, -1)],
            KeyCode::Right | KeyCode::Char('l') => vec![Intent::AdjustControlByStep(control, 1)],
            _ => Vec::new(),
        }
    }
}
