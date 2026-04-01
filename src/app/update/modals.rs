use crate::app::interaction::{InteractionMode, SurfaceId};
use crate::app::state::{AppState, CommandPaletteState, ConfigModalState};
use crate::i18n::UiText;

use super::{navigation, tr};

pub(super) fn push_modal_owner(state: &mut AppState, owner: SurfaceId) {
    let mode = InteractionMode::Modal { owner };
    if state.ui.interaction.current_mode() != mode {
        state.ui.interaction.push_mode(mode);
    }
}

pub(super) fn pop_modal_owner(state: &mut AppState, owner: SurfaceId) {
    state
        .ui
        .interaction
        .remove_mode(InteractionMode::Modal { owner });
}

pub(super) fn push_capture_owner(state: &mut AppState, owner: SurfaceId) {
    let mode = InteractionMode::Capture { owner };
    if state.ui.interaction.current_mode() != mode {
        state.ui.interaction.push_mode(mode);
    }
}

pub(super) fn pop_capture_owner(state: &mut AppState, owner: SurfaceId) {
    state
        .ui
        .interaction
        .remove_mode(InteractionMode::Capture { owner });
}

pub(super) fn close_text_input_surface(state: &mut AppState) {
    if state.ui.text_input.take().is_some() {
        pop_modal_owner(state, SurfaceId::NumericEditorSurface);
    }
}

pub(super) fn close_source_picker_surface(state: &mut AppState) {
    if state.ui.source_picker.take().is_some() {
        pop_modal_owner(state, SurfaceId::SourcePicker);
    }
}

pub(super) fn close_config_surface(state: &mut AppState) {
    if state.ui.config_modal.take().is_some() {
        pop_modal_owner(state, SurfaceId::ConfigDialog);
    }
}

pub(super) fn close_command_palette_surface(state: &mut AppState) {
    if state.ui.command_palette.take().is_some() {
        pop_modal_owner(state, SurfaceId::CommandPaletteDialog);
    }
}

pub(super) fn close_shortcut_help_surface(state: &mut AppState) {
    if state.ui.shortcut_help_open {
        state.ui.shortcut_help_open = false;
        state.ui.shortcut_help_scroll = 0;
        pop_modal_owner(state, SurfaceId::ShortcutHelp);
    }
}

pub(super) fn open_config_modal(state: &mut AppState) {
    close_command_palette_surface(state);
    close_source_picker_surface(state);
    close_text_input_surface(state);
    close_shortcut_help_surface(state);
    state.ui.config_modal = Some(ConfigModalState { selected_field: 0 });
    push_modal_owner(state, SurfaceId::ConfigDialog);
    navigation::focus_surface(state, SurfaceId::ConfigDialog);
    state.ui.status = tr(state, UiText::StatusConfigOpened);
}

pub(super) fn close_config_modal(state: &mut AppState) {
    let was_open = state.ui.config_modal.is_some();
    close_text_input_surface(state);
    close_config_surface(state);
    if was_open {
        state.ui.status = tr(state, UiText::StatusConfigClosed);
    }
}

pub(super) fn open_command_palette_modal(state: &mut AppState) {
    close_source_picker_surface(state);
    close_text_input_surface(state);
    close_config_surface(state);
    close_shortcut_help_surface(state);
    state.ui.command_palette = Some(CommandPaletteState {
        query: String::new(),
        selected: 0,
    });
    push_modal_owner(state, SurfaceId::CommandPaletteDialog);
    navigation::focus_surface(state, SurfaceId::CommandPaletteDialog);
}

pub(super) fn close_command_palette_modal(state: &mut AppState) {
    close_command_palette_surface(state);
}

pub(super) fn toggle_shortcut_help(state: &mut AppState) {
    let next = !state.ui.shortcut_help_open;
    if next {
        close_command_palette_surface(state);
        close_source_picker_surface(state);
        close_text_input_surface(state);
        close_config_surface(state);
        state.ui.shortcut_help_open = true;
        state.ui.shortcut_help_scroll = 0;
        push_modal_owner(state, SurfaceId::ShortcutHelp);
        navigation::focus_surface(state, SurfaceId::ShortcutHelp);
        state.ui.status = tr(state, UiText::StatusHelpOpened);
    } else {
        close_shortcut_help_surface(state);
        state.ui.status = tr(state, UiText::StatusHelpClosed);
    }
}

pub(super) fn scroll_shortcut_help(state: &mut AppState, delta: i32) {
    if !state.ui.shortcut_help_open {
        return;
    }

    state.ui.shortcut_help_scroll = if delta < 0 {
        state
            .ui
            .shortcut_help_scroll
            .saturating_sub(delta.unsigned_abs() as u16)
    } else {
        state.ui.shortcut_help_scroll.saturating_add(delta as u16)
    };
}
