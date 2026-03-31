use crate::app::actions::menu_bar_actions;
use crate::app::hint_nav::workspace_tab_hint_label;
use crate::app::interaction::focus_breadcrumb;
use crate::app::interaction::{InteractionMode, SurfaceId};
use crate::app::state::AppState;
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::domain::tokens::TokenRole;
use crate::i18n::{self, UiText};

use super::helpers::export_status_summary;
use super::interaction_panel::build_interaction_panel;
use super::layout::{WorkspaceLayout, compose_layout, panel_order, workspace_layout_for_tab};
use super::overlays::{
    build_config_overlay, build_help_overlay, build_numeric_editor_overlay, build_picker_overlay,
};
use super::project_tab::{
    build_editor_preferences_panel, build_export_targets_panel, build_project_config_panel,
};
use super::theme_tab::{
    build_inspector_panel, build_palette_panel, build_params_panel, build_preview_panel,
    build_token_panel, build_token_swatch_panel,
};
use super::{
    MainWindowView, MenuBarView, OverlayView, PanelView, StatusBarView, TabBarView, TabItemView,
    ViewTheme, ViewTree,
};

pub fn build_view(state: &AppState) -> ViewTree {
    let workspace_layout = workspace_layout_for_tab(state.ui.active_tab);
    build_view_with_layout(state, &workspace_layout)
}

pub fn build_view_with_layout(state: &AppState, workspace_layout: &WorkspaceLayout) -> ViewTree {
    let hint_navigation_active = matches!(
        state.ui.interaction.current_mode(),
        InteractionMode::NavigateScope(SurfaceId::MainWindow)
    );
    let theme = ViewTheme {
        background: state.theme_color(TokenRole::Background),
        surface: state.theme_color(TokenRole::Surface),
        border: state.theme_color(TokenRole::Border),
        selection: state.theme_color(TokenRole::Selection),
        text: state.theme_color(TokenRole::Text),
        text_muted: state.theme_color(TokenRole::TextMuted),
    };

    let visible_panels = panel_order(workspace_layout);
    let mut panel_for_slot = |slot| build_panel_for_slot(state, slot, &visible_panels);
    let mut status_bar_view = || build_status_bar_view(state);
    let workspace = compose_layout(workspace_layout, &mut panel_for_slot, &mut status_bar_view);
    let main_window = MainWindowView {
        hint_navigation_active,
        menu_bar: build_menu_bar_view(state),
        tab_bar: build_tab_bar_view(state),
        fullscreen_panel: state.ui.fullscreen_surface.and_then(SurfaceId::panel_id),
        workspace,
        status_bar: build_status_bar_view(state),
    };

    let mut overlays = Vec::new();
    if let Some(picker) = build_picker_overlay(state) {
        overlays.push(OverlayView::Picker(picker));
    }
    if let Some(config) = build_config_overlay(state) {
        overlays.push(config);
    }
    if let Some(editor) = build_numeric_editor_overlay(state) {
        overlays.push(editor);
    }
    if let Some(help) = build_help_overlay(state) {
        overlays.push(help);
    }

    ViewTree {
        theme,
        main_window,
        overlays,
    }
}

fn build_panel_for_slot(state: &AppState, slot: PanelId, visible_panels: &[PanelId]) -> PanelView {
    let mut panel = match slot {
        PanelId::Tokens => build_token_panel(state),
        PanelId::Params => build_params_panel(state),
        PanelId::Preview => build_preview_panel(state),
        PanelId::Palette => build_palette_panel(state),
        PanelId::ResolvedPrimary => {
            build_token_swatch_panel(state, "Resolved Tokens", &TokenRole::ALL[..10])
        }
        PanelId::ResolvedSecondary => {
            build_token_swatch_panel(state, "Resolved Tokens II", &TokenRole::ALL[10..])
        }
        PanelId::Inspector => build_inspector_panel(state),
        PanelId::InteractionInspector => build_interaction_panel(state),
        PanelId::ProjectConfig => build_project_config_panel(state),
        PanelId::ExportTargets => build_export_targets_panel(state),
        PanelId::EditorPreferences => build_editor_preferences_panel(state),
    };
    panel.id = slot;
    panel.active = state.active_panel() == slot;
    panel.hint_navigation_active = matches!(
        state.ui.interaction.current_mode(),
        InteractionMode::NavigateScope(SurfaceId::MainWindow)
    );
    panel.shortcut = visible_panels
        .iter()
        .position(|panel_id| *panel_id == slot)
        .and_then(|index| u8::try_from(index + 1).ok());
    panel
}

fn build_menu_bar_view(state: &AppState) -> MenuBarView {
    MenuBarView {
        title: i18n::text(state.locale(), UiText::MenuTitle),
        actions: menu_bar_actions(state.locale(), state.editor.keymap_preset),
    }
}

fn build_tab_bar_view(state: &AppState) -> TabBarView {
    let show_navigation_shortcuts = matches!(
        state.ui.interaction.current_mode(),
        InteractionMode::NavigateScope(SurfaceId::MainWindow)
    );

    TabBarView {
        tabs: WorkspaceTab::ALL
            .iter()
            .map(|tab| TabItemView {
                shortcut: show_navigation_shortcuts
                    .then(|| workspace_tab_hint_label(state, *tab))
                    .flatten(),
                label: i18n::workspace_tab_label(state.locale(), *tab),
                selected: *tab == state.ui.active_tab,
            })
            .collect(),
    }
}

fn build_status_bar_view(state: &AppState) -> StatusBarView {
    StatusBarView {
        focus_label: focus_breadcrumb(state),
        status_text: i18n::format2(
            state.locale(),
            UiText::StatusBarExports,
            "status",
            &state.ui.status,
            "summary",
            export_status_summary(state),
        ),
    }
}
