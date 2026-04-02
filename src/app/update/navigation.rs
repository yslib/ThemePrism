use crate::app::interaction::{InteractionMode, SurfaceId, build_interaction_tree, surface_label};
use crate::app::state::{AppState, TextInputTarget};
use crate::app::view::{panel_order, workspace_layout_for_tab};
use crate::app::workspace::PanelId;
use crate::domain::params::ParamKey;
use crate::domain::tokens::TokenRole;
use crate::i18n::{self, UiText};

use super::{cycle_index, modals, text_input, tr, tr1, tr2};

pub(super) fn cycle_workspace_tab(state: &mut AppState, delta: i32) {
    let target = if delta >= 0 {
        state.ui.active_tab.next()
    } else {
        state.ui.active_tab.previous()
    };
    set_workspace_tab(state, target);
}

pub(super) fn set_workspace_tab(state: &mut AppState, tab: crate::app::workspace::WorkspaceTab) {
    state.ui.active_tab = tab;
    let visible_panels = panel_order(&workspace_layout_for_tab(state.ui.active_tab));
    if state
        .ui
        .fullscreen_surface
        .and_then(SurfaceId::panel_id)
        .is_some_and(|panel| !visible_panels.contains(&panel))
    {
        state.ui.fullscreen_surface = None;
    }
    if state
        .ui
        .interaction
        .focused_workspace_surface()
        .is_workspace_panel()
    {
        state.ui.interaction.focus_panel(state.active_panel());
    }
    let panel = state.active_panel();
    state.ui.status = tr2(
        state,
        UiText::StatusSwitchedTab,
        "tab",
        i18n::workspace_tab_label(state.locale(), state.ui.active_tab),
        "panel",
        i18n::panel_label(state.locale(), panel),
    );
}

pub(super) fn focus_panel_by_number(state: &mut AppState, number: u8) {
    let layout = workspace_layout_for_tab(state.ui.active_tab);
    let panels = panel_order(&layout);
    let index = number.saturating_sub(1) as usize;

    if let Some(panel) = panels.get(index).copied() {
        state.set_active_panel(panel);
        state.ui.interaction.focus_panel(panel);
        state.ui.status = tr1(
            state,
            UiText::StatusFocusedPanel,
            "panel",
            i18n::panel_label(state.locale(), panel),
        );
    } else {
        state.ui.status = tr2(
            state,
            UiText::StatusTabOnlyHasPanels,
            "tab",
            i18n::workspace_tab_label(state.locale(), state.ui.active_tab),
            "count",
            panels.len(),
        );
    }
}

pub(super) fn move_selection(state: &mut AppState, delta: i32) {
    match state.active_panel() {
        PanelId::Tokens => {
            state.ui.selected_token =
                cycle_index(state.ui.selected_token, TokenRole::ALL.len(), delta);
            state.ui.inspector_field = state
                .ui
                .inspector_field
                .min(state.inspector_field_count().saturating_sub(1));
            modals::close_source_picker_surface(state);
            state.ui.status = tr1(
                state,
                UiText::StatusSelectedToken,
                "token",
                state.selected_role().label(),
            );
        }
        PanelId::Params => {
            state.ui.selected_param =
                cycle_index(state.ui.selected_param, ParamKey::ALL.len(), delta);
            modals::close_source_picker_surface(state);
            state.ui.status = tr1(
                state,
                UiText::StatusSelectedParam,
                "param",
                state.selected_param_key().label(),
            );
        }
        PanelId::Inspector => {
            state.ui.inspector_field = cycle_index(
                state.ui.inspector_field,
                state.inspector_field_count(),
                delta,
            );
            modals::close_source_picker_surface(state);
        }
        PanelId::ProjectConfig => {
            move_project_field_selection(state, PanelId::ProjectConfig, delta)
        }
        PanelId::ExportTargets => {
            move_project_field_selection(state, PanelId::ExportTargets, delta)
        }
        PanelId::EditorPreferences => {
            move_project_field_selection(state, PanelId::EditorPreferences, delta)
        }
        PanelId::InteractionInspector => {
            state.ui.interaction_inspector_scroll = if delta < 0 {
                state
                    .ui
                    .interaction_inspector_scroll
                    .saturating_sub(delta.unsigned_abs() as u16)
            } else {
                state
                    .ui
                    .interaction_inspector_scroll
                    .saturating_add(delta as u16)
            };
        }
        PanelId::Preview
        | PanelId::Palette
        | PanelId::ResolvedPrimary
        | PanelId::ResolvedSecondary => {
            state.ui.status = tr1(
                state,
                UiText::StatusPanelNoListSelection,
                "panel",
                i18n::panel_label(state.locale(), state.active_panel()),
            );
        }
    }
}

pub(super) fn focus_surface(state: &mut AppState, surface: SurfaceId) {
    match surface {
        SurfaceId::AppRoot => {}
        SurfaceId::MainWindow => {
            state.ui.interaction.set_focus_root_path();
            state.ui.status = tr1(
                state,
                UiText::StatusFocusedSurface,
                "surface",
                tr(state, UiText::SurfaceMainWindow),
            );
        }
        SurfaceId::TokensPanel
        | SurfaceId::ParamsPanel
        | SurfaceId::PreviewPanel
        | SurfaceId::PreviewTabs
        | SurfaceId::PreviewBody
        | SurfaceId::PalettePanel
        | SurfaceId::ResolvedPrimaryPanel
        | SurfaceId::ResolvedSecondaryPanel
        | SurfaceId::InspectorPanel
        | SurfaceId::InteractionInspectorPanel
        | SurfaceId::ProjectConfigPanel
        | SurfaceId::ExportTargetsPanel
        | SurfaceId::EditorPreferencesPanel => {
            let panel = surface.panel_id().expect("workspace surface");
            state.set_active_panel(panel);
            state.ui.interaction.focus_path = focus_path_for_surface(state, surface)
                .unwrap_or_else(|| {
                    vec![
                        SurfaceId::AppRoot,
                        SurfaceId::MainWindow,
                        SurfaceId::workspace_surface(panel),
                    ]
                });
            state.ui.status = tr1(
                state,
                UiText::StatusFocusedSurface,
                "surface",
                i18n::panel_label(state.locale(), panel),
            );
        }
        SurfaceId::NumericEditorSurface
        | SurfaceId::SourcePicker
        | SurfaceId::ConfigDialog
        | SurfaceId::CommandPalette
        | SurfaceId::ShortcutHelp => {
            if let Some(path) = focus_path_for_surface(state, surface) {
                state.ui.interaction.focus_path = path;
            }
        }
    }
}

pub(super) fn set_interaction_mode(state: &mut AppState, mode: InteractionMode) {
    state.ui.interaction.set_mode(mode);
    if let InteractionMode::NavigateChildren(surface) | InteractionMode::NavigateScope(surface) =
        mode
    {
        state.ui.status = tr1(
            state,
            UiText::StatusSurfaceNavigationActive,
            "surface",
            surface_label(state, surface),
        );
    }
}

pub(super) fn toggle_fullscreen(state: &mut AppState) {
    if state.ui.fullscreen_surface.is_some() {
        state.ui.fullscreen_surface = None;
        state.ui.status = tr(state, UiText::StatusFullscreenDisabled);
        return;
    }

    let focused = state.ui.interaction.focused_surface();
    if focused.panel_id().is_some() {
        state.ui.fullscreen_surface = Some(focused);
        state.ui.status = tr1(
            state,
            UiText::StatusFullscreenEnabled,
            "surface",
            surface_label(state, focused),
        );
    }
}

pub(super) fn select_token(state: &mut AppState, index: usize) {
    state.ui.selected_token = index.min(TokenRole::ALL.len().saturating_sub(1));
    state.ui.inspector_field = state
        .ui
        .inspector_field
        .min(state.inspector_field_count().saturating_sub(1));
    modals::close_source_picker_surface(state);
    modals::close_text_input_surface(state);
    modals::close_shortcut_help_surface(state);
    state.ui.status = tr1(
        state,
        UiText::StatusSelectedToken,
        "token",
        state.selected_role().label(),
    );
}

fn focus_path_for_surface(state: &AppState, surface: SurfaceId) -> Option<Vec<SurfaceId>> {
    let tree = build_interaction_tree(state);
    if tree.node(surface).is_none() {
        return None;
    }

    let mut path = vec![surface];
    let mut current = surface;
    while let Some(parent) = tree.parent_of(current) {
        path.push(parent);
        current = parent;
    }
    path.reverse();
    Some(path)
}

fn move_project_field_selection(state: &mut AppState, panel: PanelId, delta: i32) {
    let len = match panel {
        PanelId::ProjectConfig => state.project_fields().len(),
        PanelId::ExportTargets => state.export_fields().len(),
        PanelId::EditorPreferences => state.editor_fields().len(),
        _ => 0,
    };
    if len == 0 {
        state.ui.status = tr1(
            state,
            UiText::StatusPanelNoEditableFields,
            "panel",
            i18n::panel_label(state.locale(), panel),
        );
        return;
    }

    match panel {
        PanelId::ProjectConfig => {
            state.ui.project_field = cycle_index(state.ui.project_field.min(len - 1), len, delta)
        }
        PanelId::ExportTargets => {
            state.ui.export_field = cycle_index(state.ui.export_field.min(len - 1), len, delta)
        }
        PanelId::EditorPreferences => {
            state.ui.editor_field = cycle_index(state.ui.editor_field.min(len - 1), len, delta)
        }
        _ => {}
    }
    if let Some(field) = state.active_config_field() {
        state.ui.status = tr1(
            state,
            UiText::StatusSelectedField,
            "field",
            text_input::input_target_label(state, TextInputTarget::Config(field)),
        );
    }
}
