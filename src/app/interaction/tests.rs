use crate::app::interaction::{build_interaction_tree, SurfaceId};
use crate::app::controls::{ControlId, ReferenceField};
use crate::app::state::AppState;
use crate::domain::params::ParamKey;
use crate::domain::tokens::TokenRole;
use crate::app::workspace::{PanelId, WorkspaceTab};

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
