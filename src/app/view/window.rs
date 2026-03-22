use crate::app::state::{AppState, TextInputTarget};
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::domain::tokens::TokenRole;

use super::helpers::export_status_summary;
use super::layout::{WorkspaceLayout, compose_layout, panel_order, workspace_layout_for_tab};
use super::overlays::{build_config_overlay, build_numeric_editor_overlay, build_picker_overlay};
use super::project_tab::{
    build_editor_preferences_panel, build_export_targets_panel, build_project_config_panel,
};
use super::theme_tab::{
    build_code_panel, build_inspector_panel, build_palette_panel, build_params_panel,
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
        menu_bar: build_menu_bar_view(),
        tab_bar: build_tab_bar_view(state),
        workspace,
        status_bar: build_status_bar_view(state),
    };

    let mut overlays = Vec::new();
    if let Some(picker) = build_picker_overlay(state) {
        overlays.push(OverlayView::Picker(picker));
    }
    if let Some(config) = build_config_overlay(state) {
        overlays.push(OverlayView::Config(config));
    }
    if let Some(editor) = build_numeric_editor_overlay(state) {
        overlays.push(OverlayView::NumericEditor(editor));
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
        PanelId::Preview => build_code_panel(state),
        PanelId::Palette => build_palette_panel(state),
        PanelId::ResolvedPrimary => {
            build_token_swatch_panel(state, "Resolved Tokens", &TokenRole::ALL[..10])
        }
        PanelId::ResolvedSecondary => {
            build_token_swatch_panel(state, "Resolved Tokens II", &TokenRole::ALL[10..])
        }
        PanelId::Inspector => build_inspector_panel(state),
        PanelId::ProjectConfig => build_project_config_panel(state),
        PanelId::ExportTargets => build_export_targets_panel(state),
        PanelId::EditorPreferences => build_editor_preferences_panel(state),
    };
    panel.id = slot;
    panel.active = state.active_panel() == slot;
    panel.shortcut = visible_panels
        .iter()
        .position(|panel_id| *panel_id == slot)
        .and_then(|index| u8::try_from(index + 1).ok());
    panel.title = match panel.shortcut {
        Some(shortcut) => format!("[{shortcut}] {}", panel.title),
        None => panel.title,
    };
    panel
}

fn build_menu_bar_view() -> MenuBarView {
    MenuBarView {
        title: "Theme Generator".to_string(),
        actions: vec![
            "[/] Tabs".to_string(),
            "1-9 Panels".to_string(),
            "Enter Edit".to_string(),
            "s Save".to_string(),
            "o Load".to_string(),
            "e Export".to_string(),
        ],
    }
}

fn build_tab_bar_view(state: &AppState) -> TabBarView {
    TabBarView {
        tabs: WorkspaceTab::ALL
            .iter()
            .map(|tab| TabItemView {
                label: tab.label().to_string(),
                selected: *tab == state.ui.active_tab,
            })
            .collect(),
    }
}

fn build_status_bar_view(state: &AppState) -> StatusBarView {
    StatusBarView {
        focus_label: format!(
            "{} / {}",
            state.ui.active_tab.label(),
            state.active_panel().label()
        ),
        help_text: status_help_text(state).to_string(),
        status_text: format!(
            "{}  |  Exports: {}",
            state.ui.status,
            export_status_summary(state)
        ),
    }
}

fn status_help_text(state: &AppState) -> &'static str {
    if state.ui.source_picker.is_some() {
        "↑↓ select  |  type to filter  |  Enter apply  |  Esc close"
    } else if let Some(input) = &state.ui.text_input {
        match input.target {
            TextInputTarget::Control(control) if control.supports_numeric_editor() => {
                "←→ nudge live  |  type exact value  |  Enter apply  |  Esc close  |  Del clear"
            }
            _ => "Enter apply  |  Esc cancel  |  Backspace delete",
        }
    } else if state.ui.config_modal.is_some() {
        "↑↓ select  |  Enter edit/toggle  |  Space toggle  |  Esc close"
    } else if state.active_config_field().is_some() {
        "[ ] tabs  |  1-9 panels  |  ↑↓ fields  |  Enter edit/toggle  |  c modal  |  q quit"
    } else {
        "[ ] tabs  |  1-9 panels  |  Tab cycle  |  ↑↓ select  |  ←→ adjust  |  Enter edit  |  q quit"
    }
}
