mod routing;
mod state;
mod tree;

#[cfg(test)]
mod tests;

pub use routing::route_ui_action;
pub use state::{InteractionMode, InteractionState};
#[allow(unused_imports)]
pub use tree::{
    build_interaction_tree, BubblePolicy, ChildNavigation, DefaultAction, InteractionTree,
    SurfaceId, SurfaceNode, TabScope,
};

use crate::app::state::AppState;
use crate::app::workspace::PanelId;
use crate::i18n::{self, UiText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    OpenNavigation,
    NavigateTo(char),
    PreviousTab,
    NextTab,
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
    ToggleFullscreenRequested,
    SaveProject,
    LoadProject,
    ExportTheme,
    Reset,
    Quit,
    TypeChar(char),
}

pub fn effective_focus_path(state: &AppState) -> Vec<SurfaceId> {
    let mut path = if state.ui.interaction.focus_path.is_empty() {
        vec![SurfaceId::AppRoot, SurfaceId::MainWindow]
    } else {
        state.ui.interaction.focus_path.clone()
    };

    if let InteractionMode::Capture { owner } = state.ui.interaction.current_mode() {
        if path.last().copied() != Some(owner) {
            path.push(owner);
        }
    }

    path
}

pub fn effective_focus_surface(state: &AppState) -> SurfaceId {
    effective_focus_path(state)
        .last()
        .copied()
        .unwrap_or(SurfaceId::MainWindow)
}

pub fn has_active_capture(state: &AppState, owner: SurfaceId) -> bool {
    matches!(
        state.ui.interaction.current_mode(),
        InteractionMode::Capture { owner: active_owner } if active_owner == owner
    ) && effective_focus_surface(state) == owner
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
        SurfaceId::InteractionInspectorPanel => {
            i18n::panel_label(state.locale(), PanelId::InteractionInspector)
        }
        SurfaceId::ProjectConfigPanel => i18n::panel_label(state.locale(), PanelId::ProjectConfig),
        SurfaceId::ExportTargetsPanel => i18n::panel_label(state.locale(), PanelId::ExportTargets),
        SurfaceId::EditorPreferencesPanel => {
            i18n::panel_label(state.locale(), PanelId::EditorPreferences)
        }
        SurfaceId::NumericEditorSurface => i18n::text(state.locale(), UiText::SurfaceInputEditor),
        SurfaceId::SourcePicker => i18n::text(state.locale(), UiText::SurfaceSourcePicker),
        SurfaceId::ConfigDialog => i18n::text(state.locale(), UiText::SurfaceConfigDialog),
        SurfaceId::CommandPaletteDialog => "Command Palette".to_string(),
        SurfaceId::ShortcutHelp => i18n::text(state.locale(), UiText::SurfaceShortcutHelp),
    }
}
