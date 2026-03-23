use crate::app::interaction::{build_interaction_tree, SurfaceId};
use crate::app::controls::{ControlId, ReferenceField};
use crate::app::state::AppState;
use crate::domain::params::ParamKey;
use crate::domain::tokens::TokenRole;
use crate::app::workspace::{PanelId, WorkspaceTab};

#[test]
fn interaction_tree_contains_preview_tabs_under_preview_panel() {
    let state = AppState::new().expect("state");
    let tree = build_interaction_tree(&state);

    assert_eq!(
        tree.parent_of(SurfaceId::PreviewTabs),
        Some(SurfaceId::PreviewPanel)
    );
    assert_eq!(
        tree.parent_of(SurfaceId::PreviewBody),
        Some(SurfaceId::PreviewPanel)
    );
}

#[test]
fn interaction_tree_uses_visible_panels_for_active_workspace_tab() {
    let mut state = AppState::new().expect("state");
    state.ui.active_tab = WorkspaceTab::Project;

    let tree = build_interaction_tree(&state);

    assert!(tree.is_visible(SurfaceId::ProjectConfigPanel));
    assert!(!tree.is_visible(SurfaceId::TokensPanel));
}

#[test]
fn interaction_tree_uses_visible_theme_tab_children() {
    let state = AppState::new().expect("state");
    let tree = build_interaction_tree(&state);

    assert_eq!(
        tree.node(SurfaceId::MainWindow).expect("main window").children,
        vec![
            SurfaceId::TokensPanel,
            SurfaceId::ParamsPanel,
            SurfaceId::PalettePanel,
            SurfaceId::ResolvedPrimaryPanel,
            SurfaceId::ResolvedSecondaryPanel,
            SurfaceId::PreviewPanel,
            SurfaceId::InspectorPanel,
        ]
    );
}

#[test]
fn interaction_tree_uses_visible_project_tab_children() {
    let mut state = AppState::new().expect("state");
    state.ui.active_tab = WorkspaceTab::Project;

    let tree = build_interaction_tree(&state);

    assert_eq!(
        tree.node(SurfaceId::MainWindow).expect("main window").children,
        vec![
            SurfaceId::ProjectConfigPanel,
            SurfaceId::ExportTargetsPanel,
            SurfaceId::EditorPreferencesPanel,
        ]
    );
}

#[test]
fn interaction_tree_child_links_are_reciprocal() {
    let state = AppState::new().expect("state");
    let tree = build_interaction_tree(&state);

    for node_id in [
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::TokensPanel,
        SurfaceId::ParamsPanel,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewTabs,
        SurfaceId::PreviewBody,
        SurfaceId::PalettePanel,
        SurfaceId::ResolvedPrimaryPanel,
        SurfaceId::ResolvedSecondaryPanel,
        SurfaceId::InspectorPanel,
        SurfaceId::ProjectConfigPanel,
        SurfaceId::ExportTargetsPanel,
        SurfaceId::EditorPreferencesPanel,
        SurfaceId::NumericEditorSurface,
        SurfaceId::SourcePicker,
        SurfaceId::ConfigDialog,
        SurfaceId::ShortcutHelp,
    ] {
        let node = tree.node(node_id).expect("node exists");
        for &child in &node.children {
            assert_eq!(
                tree.parent_of(child),
                Some(node_id),
                "child {child:?} should point back to {node_id:?}"
            );
        }
    }
}

#[test]
fn hidden_workspace_panels_do_not_report_an_active_parent() {
    let mut state = AppState::new().expect("state");
    state.ui.active_tab = WorkspaceTab::Project;

    let tree = build_interaction_tree(&state);

    assert_eq!(tree.parent_of(SurfaceId::TokensPanel), None);
    assert_eq!(tree.parent_of(SurfaceId::ParamsPanel), None);
    assert_eq!(tree.parent_of(SurfaceId::PalettePanel), None);
    assert_eq!(tree.parent_of(SurfaceId::ResolvedPrimaryPanel), None);
    assert_eq!(tree.parent_of(SurfaceId::ResolvedSecondaryPanel), None);
    assert_eq!(tree.parent_of(SurfaceId::InspectorPanel), None);
}

#[test]
fn interaction_tree_keeps_resolved_theme_panels_distinct() {
    assert_eq!(
        SurfaceId::workspace_surface(PanelId::ResolvedPrimary),
        SurfaceId::ResolvedPrimaryPanel
    );
    assert_eq!(
        SurfaceId::workspace_surface(PanelId::ResolvedSecondary),
        SurfaceId::ResolvedSecondaryPanel
    );
    assert_ne!(
        SurfaceId::ResolvedPrimaryPanel,
        SurfaceId::ResolvedSecondaryPanel
    );

    let state = AppState::new().expect("state");
    let tree = build_interaction_tree(&state);

    assert_eq!(
        tree.parent_of(SurfaceId::ResolvedPrimaryPanel),
        Some(SurfaceId::MainWindow)
    );
    assert_eq!(
        tree.parent_of(SurfaceId::ResolvedSecondaryPanel),
        Some(SurfaceId::MainWindow)
    );
    assert!(tree.is_visible(SurfaceId::ResolvedPrimaryPanel));
    assert!(tree.is_visible(SurfaceId::ResolvedSecondaryPanel));
}

#[test]
fn interaction_tree_tracks_modal_visibility() {
    let mut state = AppState::new().expect("state");
    state.ui.text_input = Some(crate::app::state::TextInputState {
        target: crate::app::state::TextInputTarget::Control(ControlId::Param(ParamKey::BackgroundHue)),
        buffer: String::new(),
    });

    let tree = build_interaction_tree(&state);
    assert!(tree.is_visible(SurfaceId::NumericEditorSurface));
    assert!(!tree.is_visible(SurfaceId::SourcePicker));

    let mut state = AppState::new().expect("state");
    state.ui.source_picker = Some(crate::app::state::SourcePickerState {
        control: ControlId::Reference(TokenRole::Text, ReferenceField::AliasSource),
        filter: String::new(),
        selected: 0,
    });

    let tree = build_interaction_tree(&state);
    assert!(tree.is_visible(SurfaceId::SourcePicker));
    assert!(!tree.is_visible(SurfaceId::ConfigDialog));
}
