mod state;
mod tree;

#[cfg(test)]
mod tests;

pub use state::{InteractionMode, InteractionState};
#[allow(unused_imports)]
pub use tree::{
    BubblePolicy, ChildNavigation, DefaultAction, InteractionTree, SurfaceId, SurfaceNode,
    TabScope, build_interaction_tree,
};

use crate::app::Intent;
use crate::app::state::AppState;
use crate::app::workspace::PanelId;
use crate::i18n::{self, UiText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    PreviousTab,
    NextTab,
    PreviousPanel,
    NextPanel,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Activate,
    Toggle,
    Apply,
    Cancel,
    Backspace,
    Clear,
    OpenConfig,
    OpenHelp,
    SaveProject,
    LoadProject,
    ExportTheme,
    Reset,
    Quit,
    SelectChild(u8),
    TypeChar(char),
}

pub fn effective_focus_path(state: &AppState) -> Vec<SurfaceId> {
    let mut path = if state.ui.interaction.focus_path.is_empty() {
        vec![SurfaceId::AppRoot, SurfaceId::MainWindow]
    } else {
        state.ui.interaction.focus_path.clone()
    };

    if state.ui.text_input.is_some() {
        path.push(SurfaceId::NumericEditorSurface);
    } else if state.ui.source_picker.is_some() {
        path.push(SurfaceId::SourcePicker);
    } else if state.ui.config_modal.is_some() {
        path.push(SurfaceId::ConfigDialog);
    } else if state.ui.shortcut_help_open {
        path.push(SurfaceId::ShortcutHelp);
    }

    path
}

pub fn effective_focus_surface(state: &AppState) -> SurfaceId {
    effective_focus_path(state)
        .last()
        .copied()
        .unwrap_or(SurfaceId::MainWindow)
}

pub fn focus_breadcrumb(state: &AppState) -> String {
    let mut parts = vec![i18n::workspace_tab_label(
        state.locale(),
        state.ui.active_tab,
    )];
    let focus_path = effective_focus_path(state);

    for (index, surface) in focus_path.iter().enumerate() {
        if index == 0 && matches!(surface, SurfaceId::AppRoot) {
            continue;
        }
        if index == 1 && matches!(surface, SurfaceId::MainWindow) && focus_path.len() > 2 {
            continue;
        }
        parts.push(surface_label(state, *surface));
    }

    parts.join(" / ")
}

pub fn route_ui_action(state: &AppState, action: UiAction) -> Vec<Intent> {
    let focus_path = effective_focus_path(state);
    let mut modal_boundary = false;

    for surface in focus_path.iter().rev().copied() {
        if let Some(intents) = route_on_surface(state, surface, action) {
            return intents;
        }

        if surface.is_modal_surface() {
            modal_boundary = true;
            break;
        }
    }

    if modal_boundary {
        Vec::new()
    } else {
        Vec::new()
    }
}

fn route_on_surface(state: &AppState, surface: SurfaceId, action: UiAction) -> Option<Vec<Intent>> {
    if let Some(panel) = surface.panel_id() {
        return route_panel_action(state, panel, action);
    }

    match surface {
        SurfaceId::AppRoot => None,
        SurfaceId::MainWindow => route_main_window_action(state, action),
        SurfaceId::NumericEditorSurface => route_text_input_action(state, action),
        SurfaceId::SourcePicker => route_source_picker_action(action),
        SurfaceId::ConfigDialog => route_config_dialog_action(action),
        SurfaceId::ShortcutHelp => route_shortcut_help_action(action),
        SurfaceId::PreviewTabs
        | SurfaceId::PreviewBody
        | SurfaceId::TokensPanel
        | SurfaceId::ParamsPanel
        | SurfaceId::PreviewPanel
        | SurfaceId::ResolvedPrimaryPanel
        | SurfaceId::ResolvedSecondaryPanel
        | SurfaceId::PalettePanel
        | SurfaceId::InspectorPanel
        | SurfaceId::ProjectConfigPanel
        | SurfaceId::ExportTargetsPanel
        | SurfaceId::EditorPreferencesPanel => None,
    }
}

fn route_main_window_action(state: &AppState, action: UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::PreviousTab => Some(vec![Intent::CycleWorkspaceTab(-1)]),
        UiAction::NextTab => Some(vec![Intent::CycleWorkspaceTab(1)]),
        UiAction::PreviousPanel => Some(vec![Intent::MoveFocus(-1)]),
        UiAction::NextPanel => Some(vec![Intent::MoveFocus(1)]),
        UiAction::Activate => Some(vec![
            Intent::FocusSurface(SurfaceId::MainWindow),
            Intent::SetInteractionMode(InteractionMode::NavigateChildren(SurfaceId::MainWindow)),
        ]),
        UiAction::SelectChild(number)
            if state.ui.interaction.current_mode()
                == InteractionMode::NavigateChildren(SurfaceId::MainWindow) =>
        {
            Some(vec![
                Intent::FocusPanelByNumber(number),
                Intent::SetInteractionMode(InteractionMode::Normal),
            ])
        }
        UiAction::Cancel => Some(vec![
            Intent::FocusSurface(SurfaceId::MainWindow),
            Intent::SetInteractionMode(InteractionMode::Normal),
        ]),
        UiAction::OpenConfig => Some(vec![Intent::OpenConfigRequested]),
        UiAction::OpenHelp => Some(vec![Intent::ToggleShortcutHelpRequested]),
        UiAction::SaveProject => Some(vec![Intent::SaveProjectRequested]),
        UiAction::LoadProject => Some(vec![Intent::LoadProjectRequested]),
        UiAction::ExportTheme => Some(vec![Intent::ExportThemeRequested]),
        UiAction::Reset => Some(vec![Intent::ResetRequested]),
        UiAction::Quit => Some(vec![Intent::QuitRequested]),
        _ => None,
    }
}

fn route_panel_action(state: &AppState, panel: PanelId, action: UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::PreviousTab if panel == PanelId::Preview => {
            Some(vec![Intent::CyclePreviewMode(-1)])
        }
        UiAction::NextTab if panel == PanelId::Preview => Some(vec![Intent::CyclePreviewMode(1)]),
        UiAction::Activate if panel == PanelId::Preview => {
            Some(vec![Intent::SetPreviewCapture(true)])
        }
        UiAction::Cancel => Some(vec![
            Intent::FocusSurface(SurfaceId::MainWindow),
            Intent::SetInteractionMode(InteractionMode::Normal),
        ]),
        UiAction::PreviousPanel => Some(vec![Intent::MoveFocus(-1)]),
        UiAction::NextPanel => Some(vec![Intent::MoveFocus(1)]),
        UiAction::MoveUp => Some(vec![Intent::MoveSelection(-1)]),
        UiAction::MoveDown => Some(vec![Intent::MoveSelection(1)]),
        UiAction::MoveLeft if state.active_control().is_some() => {
            Some(vec![Intent::AdjustControlByStep(
                state.active_control().expect("checked above"),
                -1,
            )])
        }
        UiAction::MoveRight if state.active_control().is_some() => {
            Some(vec![Intent::AdjustControlByStep(
                state.active_control().expect("checked above"),
                1,
            )])
        }
        UiAction::Activate if state.active_control().is_some() => {
            Some(vec![Intent::ActivateControl(
                state.active_control().expect("checked above"),
            )])
        }
        UiAction::Activate | UiAction::Toggle if state.active_config_field().is_some() => {
            Some(vec![Intent::ActivateConfigField])
        }
        _ => None,
    }
}

fn route_text_input_action(state: &AppState, action: UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::Apply => Some(vec![Intent::CommitTextInput]),
        UiAction::Cancel => Some(vec![Intent::CancelTextInput]),
        UiAction::Backspace => Some(vec![Intent::BackspaceTextInput]),
        UiAction::Clear => Some(vec![Intent::ClearTextInput]),
        UiAction::MoveLeft if state.ui.text_input.as_ref().is_some_and(|input| {
            matches!(
                input.target,
                crate::app::state::TextInputTarget::Control(control) if control.supports_numeric_editor()
            )
        }) => Some(vec![Intent::AdjustActiveNumericInputByStep(-1)]),
        UiAction::MoveRight if state.ui.text_input.as_ref().is_some_and(|input| {
            matches!(
                input.target,
                crate::app::state::TextInputTarget::Control(control) if control.supports_numeric_editor()
            )
        }) => Some(vec![Intent::AdjustActiveNumericInputByStep(1)]),
        UiAction::TypeChar(ch) => Some(vec![Intent::AppendTextInput(ch)]),
        _ => None,
    }
}

fn route_source_picker_action(action: UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::Apply => Some(vec![Intent::ApplySourcePickerSelection]),
        UiAction::Cancel => Some(vec![Intent::CloseSourcePicker]),
        UiAction::MoveUp => Some(vec![Intent::MoveSourcePickerSelection(-1)]),
        UiAction::MoveDown => Some(vec![Intent::MoveSourcePickerSelection(1)]),
        UiAction::Backspace => Some(vec![Intent::BackspaceSourcePickerFilter]),
        UiAction::Clear => Some(vec![Intent::ClearSourcePickerFilter]),
        UiAction::TypeChar(ch) => Some(vec![Intent::AppendSourcePickerFilter(ch)]),
        _ => None,
    }
}

fn route_config_dialog_action(action: UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::OpenHelp => Some(vec![Intent::ToggleShortcutHelpRequested]),
        UiAction::Cancel => Some(vec![Intent::CloseConfigRequested]),
        UiAction::Activate | UiAction::Toggle => Some(vec![Intent::ActivateConfigField]),
        UiAction::MoveUp => Some(vec![Intent::MoveConfigSelection(-1)]),
        UiAction::MoveDown => Some(vec![Intent::MoveConfigSelection(1)]),
        _ => None,
    }
}

fn route_shortcut_help_action(action: UiAction) -> Option<Vec<Intent>> {
    match action {
        UiAction::OpenHelp | UiAction::Cancel => Some(vec![Intent::ToggleShortcutHelpRequested]),
        UiAction::MoveUp => Some(vec![Intent::ScrollShortcutHelp(-1)]),
        UiAction::MoveDown => Some(vec![Intent::ScrollShortcutHelp(1)]),
        _ => None,
    }
}

pub fn surface_label(state: &AppState, surface: SurfaceId) -> String {
    match surface {
        SurfaceId::AppRoot => i18n::text(state.locale(), UiText::SurfaceMainWindow),
        SurfaceId::MainWindow => i18n::text(state.locale(), UiText::SurfaceMainWindow),
        SurfaceId::TokensPanel => i18n::panel_label(state.locale(), PanelId::Tokens),
        SurfaceId::ParamsPanel => i18n::panel_label(state.locale(), PanelId::Params),
        SurfaceId::PreviewPanel | SurfaceId::PreviewTabs | SurfaceId::PreviewBody => {
            i18n::panel_label(state.locale(), PanelId::Preview)
        }
        SurfaceId::PalettePanel => i18n::panel_label(state.locale(), PanelId::Palette),
        SurfaceId::ResolvedPrimaryPanel => {
            i18n::panel_label(state.locale(), PanelId::ResolvedPrimary)
        }
        SurfaceId::ResolvedSecondaryPanel => {
            i18n::panel_label(state.locale(), PanelId::ResolvedSecondary)
        }
        SurfaceId::InspectorPanel => i18n::panel_label(state.locale(), PanelId::Inspector),
        SurfaceId::ProjectConfigPanel => i18n::panel_label(state.locale(), PanelId::ProjectConfig),
        SurfaceId::ExportTargetsPanel => i18n::panel_label(state.locale(), PanelId::ExportTargets),
        SurfaceId::EditorPreferencesPanel => {
            i18n::panel_label(state.locale(), PanelId::EditorPreferences)
        }
        SurfaceId::NumericEditorSurface => i18n::text(state.locale(), UiText::SurfaceInputEditor),
        SurfaceId::SourcePicker => i18n::text(state.locale(), UiText::SurfaceSourcePicker),
        SurfaceId::ConfigDialog => i18n::text(state.locale(), UiText::SurfaceConfigDialog),
        SurfaceId::ShortcutHelp => i18n::text(state.locale(), UiText::SurfaceShortcutHelp),
    }
}
