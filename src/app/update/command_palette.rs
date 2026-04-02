use crate::app::command_palette::{CommandId, filter_commands};
use crate::app::effect::Effect;
use crate::app::intent::Intent;
use crate::app::state::AppState;

use super::{cycle_index, modals, update};

pub(super) fn open_command_palette(state: &mut AppState) {
    modals::open_command_palette_modal(state);
}

pub(super) fn close_command_palette(state: &mut AppState) {
    modals::close_command_palette_modal(state);
}

pub(super) fn set_query(state: &mut AppState, query: String) {
    if let Some(palette) = &mut state.ui.command_palette {
        palette.query = query;
        palette.selected = 0;
        clamp_selection(state);
    }
}

pub(super) fn append_query(state: &mut AppState, ch: char) {
    if let Some(palette) = &mut state.ui.command_palette {
        palette.query.push(ch);
        palette.selected = 0;
        clamp_selection(state);
    }
}

pub(super) fn backspace_query(state: &mut AppState) {
    if let Some(palette) = &mut state.ui.command_palette {
        palette.query.pop();
        palette.selected = 0;
        clamp_selection(state);
    }
}

pub(super) fn clear_query(state: &mut AppState) {
    if let Some(palette) = &mut state.ui.command_palette {
        palette.query.clear();
        palette.selected = 0;
    }
}

pub(super) fn move_selection(state: &mut AppState, delta: i32) {
    let Some(current) = state.ui.command_palette.as_ref() else {
        return;
    };
    let len = filter_commands(state, &current.query).len();
    let Some(palette) = &mut state.ui.command_palette else {
        return;
    };

    palette.selected = if len == 0 {
        0
    } else {
        cycle_index(palette.selected.min(len.saturating_sub(1)), len, delta)
    };
}

pub(super) fn run_selected(state: &mut AppState) -> Vec<Effect> {
    let Some(palette) = state.ui.command_palette.as_ref() else {
        return Vec::new();
    };
    let selected = filter_commands(state, &palette.query)
        .get(palette.selected)
        .map(|item| item.id);

    match selected {
        Some(command) => {
            modals::close_command_palette_modal(state);
            update(state, command_intent(command))
        }
        None => Vec::new(),
    }
}

fn clamp_selection(state: &mut AppState) {
    let Some(query) = state
        .ui
        .command_palette
        .as_ref()
        .map(|palette| palette.query.clone())
    else {
        return;
    };
    let len = filter_commands(state, &query).len();
    let Some(palette) = &mut state.ui.command_palette else {
        return;
    };

    if len == 0 {
        palette.selected = 0;
    } else {
        palette.selected = palette.selected.min(len - 1);
    }
}

fn command_intent(command: CommandId) -> Intent {
    match command {
        CommandId::SaveProject => Intent::SaveProjectRequested,
        CommandId::LoadProject => Intent::LoadProjectRequested,
        CommandId::ExportTheme => Intent::ExportThemeRequested,
        CommandId::Reset => Intent::ResetRequested,
        CommandId::OpenConfig => Intent::OpenConfigRequested,
        CommandId::OpenHelp => Intent::ToggleShortcutHelpRequested,
        CommandId::ToggleFullscreen => Intent::ToggleFullscreenRequested,
        CommandId::Quit => Intent::QuitRequested,
    }
}
