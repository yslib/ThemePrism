use crate::app::interaction::{build_interaction_tree, SurfaceId};
use crate::app::state::AppState;
use crate::app::workspace::WorkspaceTab;

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
