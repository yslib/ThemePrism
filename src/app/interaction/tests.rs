use crate::app::controls::{ControlId, ReferenceField};
use crate::app::interaction::{
    InteractionMode, InteractionState, SurfaceId, UiAction, build_interaction_tree,
    effective_focus_path, route_ui_action,
};
use crate::app::state::AppState;
use crate::app::view::{PanelBody, ViewNode, build_interaction_panel, build_view};
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::app::{Intent, update};
use crate::domain::params::ParamKey;
use crate::domain::tokens::TokenRole;

#[test]
fn modal_mode_pushes_and_pops_without_losing_owner_focus() {
    let mut interaction = InteractionState::new(SurfaceId::TokensPanel);

    assert_eq!(interaction.current_mode(), InteractionMode::Normal);
    interaction.push_mode(InteractionMode::Capture {
        owner: SurfaceId::PreviewBody,
    });
    assert_eq!(
        interaction.current_mode(),
        InteractionMode::Capture {
            owner: SurfaceId::PreviewBody,
        }
    );
    interaction.push_mode(InteractionMode::Modal {
        owner: SurfaceId::NumericEditorSurface,
    });
    assert_eq!(
        interaction.current_mode(),
        InteractionMode::Modal {
            owner: SurfaceId::NumericEditorSurface,
        }
    );
    interaction.focus_path = vec![
        SurfaceId::MainWindow,
        SurfaceId::ParamsPanel,
        SurfaceId::NumericEditorSurface,
    ];

    interaction.pop_mode();

    assert_eq!(
        interaction.current_mode(),
        InteractionMode::Capture {
            owner: SurfaceId::PreviewBody,
        }
    );
    assert_eq!(interaction.focused_surface(), SurfaceId::ParamsPanel);

    interaction.pop_mode();

    assert_eq!(interaction.current_mode(), InteractionMode::Normal);
}

#[test]
fn capture_mode_can_stack_on_top_of_normal_mode() {
    let mut interaction = InteractionState::new(SurfaceId::PreviewBody);
    assert_eq!(interaction.current_mode(), InteractionMode::Normal);
    interaction.push_mode(InteractionMode::Capture {
        owner: SurfaceId::PreviewBody,
    });

    assert_eq!(
        interaction.current_mode(),
        InteractionMode::Capture {
            owner: SurfaceId::PreviewBody,
        }
    );
    assert!(interaction.has_mode_for(SurfaceId::PreviewBody));

    interaction.pop_mode();

    assert_eq!(interaction.current_mode(), InteractionMode::Normal);
}

#[test]
fn pop_mode_only_pops_focus_when_owner_matches_trailing_surface() {
    let mut interaction = InteractionState::new(SurfaceId::PreviewBody);
    interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];
    interaction.push_mode(InteractionMode::Capture {
        owner: SurfaceId::PreviewBody,
    });
    interaction.focus_path.push(SurfaceId::NumericEditorSurface);

    interaction.pop_mode();

    assert_eq!(interaction.current_mode(), InteractionMode::Normal);
    assert_eq!(
        interaction.focus_path,
        vec![
            SurfaceId::AppRoot,
            SurfaceId::MainWindow,
            SurfaceId::PreviewPanel,
            SurfaceId::PreviewBody,
            SurfaceId::NumericEditorSurface,
        ]
    );
}

#[test]
fn digit_navigation_works_without_scope_navigation_mode() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_panel(PanelId::Tokens);

    let intents = route_ui_action(&state, UiAction::NavigateTo('2'));

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::FocusPanelByNumber(2)]
    ));
}

#[test]
fn open_navigation_on_workspace_surface_enters_main_window_scope_navigation() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_panel(PanelId::Tokens);

    let intents = route_ui_action(&state, UiAction::OpenNavigation);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::SetInteractionMode(
            InteractionMode::NavigateScope(SurfaceId::MainWindow)
        )]
    ));
}

#[test]
fn navigate_to_letter_switches_workspace_tab_immediately() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_root();
    state
        .ui
        .interaction
        .set_mode(InteractionMode::NavigateScope(SurfaceId::MainWindow));

    let intents = route_ui_action(&state, UiAction::NavigateTo('b'));

    assert!(matches!(
        intents.as_slice(),
        [
            crate::app::Intent::SetWorkspaceTab(WorkspaceTab::Project),
            crate::app::Intent::SetInteractionMode(InteractionMode::Normal)
        ]
    ));
}

#[test]
fn letter_navigation_requires_scope_navigation_mode() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_panel(PanelId::Tokens);

    assert!(route_ui_action(&state, UiAction::NavigateTo('b')).is_empty());
}

#[test]
fn navigate_to_preview_tab_switches_mode_and_focuses_preview_tabs() {
    let mut state = AppState::new().expect("state");
    state.set_active_panel(PanelId::Tokens);
    state.ui.interaction.focus_panel(PanelId::Tokens);
    state
        .ui
        .interaction
        .set_mode(InteractionMode::NavigateScope(SurfaceId::MainWindow));

    let intents = route_ui_action(&state, UiAction::NavigateTo('d'));

    assert!(matches!(
        intents.as_slice(),
        [
            crate::app::Intent::SetPreviewMode(crate::preview::PreviewMode::Shell),
            crate::app::Intent::FocusSurface(SurfaceId::PreviewTabs),
            crate::app::Intent::SetInteractionMode(InteractionMode::Normal)
        ]
    ));
}

#[test]
fn cancel_on_main_window_scope_navigation_only_exits_nav_mode() {
    let mut state = AppState::new().expect("state");
    state.set_active_panel(PanelId::Inspector);
    state.ui.interaction.focus_panel(PanelId::Inspector);
    state
        .ui
        .interaction
        .set_mode(InteractionMode::NavigateScope(SurfaceId::MainWindow));

    let intents = route_ui_action(&state, UiAction::Cancel);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::SetInteractionMode(
            InteractionMode::Normal
        )]
    ));
}

#[test]
fn switch_tab_bubbles_from_tokens_panel_to_main_window() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::TokensPanel,
    ];

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::CycleWorkspaceTab(1)]
    ));
}

#[test]
fn switch_tab_bubbles_from_preview_panel_to_main_window_when_preview_tabs_are_not_focused() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
    ];

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::CycleWorkspaceTab(1)]
    ));
}

#[test]
fn switch_tab_is_handled_locally_by_preview_tabs() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewTabs,
    ];

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::CyclePreviewMode(1)]
    ));
}

#[test]
fn activate_on_preview_panel_enters_child_navigation_at_preview_tabs() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
    ];

    let intents = route_ui_action(&state, UiAction::Activate);

    assert!(matches!(
        intents.as_slice(),
        [
            crate::app::Intent::FocusSurface(SurfaceId::PreviewTabs),
            crate::app::Intent::SetInteractionMode(InteractionMode::NavigateChildren(
                SurfaceId::PreviewPanel
            ))
        ]
    ));
}

#[test]
fn activate_on_preview_body_enters_capture_mode() {
    let mut state = AppState::new().expect("state");
    state.preview.active_mode = crate::preview::PreviewMode::Shell;
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];

    let intents = route_ui_action(&state, UiAction::Activate);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::SetPreviewCapture(true)]
    ));
}

#[test]
fn command_palette_is_visible_as_a_modal_surface_when_open() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);

    let tree = build_interaction_tree(&state);

    assert!(tree.is_visible(SurfaceId::CommandPalette));
}

#[test]
fn interaction_inspector_lists_focus_path_and_modes() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];
    state.ui.interaction.push_mode(InteractionMode::Capture {
        owner: SurfaceId::PreviewBody,
    });

    let panel = build_interaction_panel(&state);

    let PanelBody::Document(document) = &panel.body else {
        panic!("expected document body");
    };
    let body = document
        .lines
        .iter()
        .flat_map(|line| line.spans.iter().map(|span| span.text.as_str()))
        .collect::<Vec<_>>()
        .join("");
    assert!(body.contains("PreviewBody"));
    assert!(body.contains("Capture"));
    assert!(body.contains("[visible]"));
}

#[test]
fn theme_workspace_view_includes_interaction_inspector_as_panel_8() {
    let state = AppState::new().expect("state");
    let view = build_view(&state);
    let mut panels = Vec::new();

    collect_panels(&view.main_window.workspace, &mut panels);

    assert_eq!(
        panels.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![
            PanelId::Tokens,
            PanelId::Params,
            PanelId::Palette,
            PanelId::ResolvedPrimary,
            PanelId::ResolvedSecondary,
            PanelId::Preview,
            PanelId::Inspector,
            PanelId::InteractionInspector,
        ]
    );
    assert_eq!(
        panels
            .iter()
            .find(|(id, _)| *id == PanelId::InteractionInspector)
            .map(|(_, shortcut)| *shortcut),
        Some(Some(8))
    );
}

#[test]
fn switch_tab_on_preview_body_bubbles_to_preview_tabs_owner() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::CyclePreviewMode(1)]
    ));
}

#[test]
fn move_right_advances_between_preview_children_while_navigation_is_active() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewTabs,
    ];
    state
        .ui
        .interaction
        .set_mode(InteractionMode::NavigateChildren(SurfaceId::PreviewPanel));

    let intents = route_ui_action(&state, UiAction::MoveRight);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::FocusSurface(SurfaceId::PreviewBody)]
    ));
}

#[test]
fn move_down_advances_between_preview_children_while_navigation_is_active() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewTabs,
    ];
    state
        .ui
        .interaction
        .set_mode(InteractionMode::NavigateChildren(SurfaceId::PreviewPanel));

    let intents = route_ui_action(&state, UiAction::MoveDown);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::FocusSurface(SurfaceId::PreviewBody)]
    ));
}

#[test]
fn move_up_rewinds_between_preview_children_while_navigation_is_active() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];
    state
        .ui
        .interaction
        .set_mode(InteractionMode::NavigateChildren(SurfaceId::PreviewPanel));

    let intents = route_ui_action(&state, UiAction::MoveUp);

    assert!(matches!(
        intents.as_slice(),
        [crate::app::Intent::FocusSurface(SurfaceId::PreviewTabs)]
    ));
}

#[test]
fn move_up_on_first_preview_child_is_a_noop_during_navigation() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewTabs,
    ];
    state
        .ui
        .interaction
        .set_mode(InteractionMode::NavigateChildren(SurfaceId::PreviewPanel));

    let intents = route_ui_action(&state, UiAction::MoveUp);

    assert!(intents.is_empty());
}

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
        tree.node(SurfaceId::MainWindow)
            .expect("main window")
            .children,
        vec![
            SurfaceId::TokensPanel,
            SurfaceId::ParamsPanel,
            SurfaceId::PalettePanel,
            SurfaceId::ResolvedPrimaryPanel,
            SurfaceId::ResolvedSecondaryPanel,
            SurfaceId::PreviewPanel,
            SurfaceId::InspectorPanel,
            SurfaceId::InteractionInspectorPanel,
        ]
    );
}

#[test]
fn interaction_tree_uses_visible_project_tab_children() {
    let mut state = AppState::new().expect("state");
    state.ui.active_tab = WorkspaceTab::Project;

    let tree = build_interaction_tree(&state);

    assert_eq!(
        tree.node(SurfaceId::MainWindow)
            .expect("main window")
            .children,
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
        SurfaceId::InteractionInspectorPanel,
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
    assert_eq!(tree.parent_of(SurfaceId::InteractionInspectorPanel), None);
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
        target: crate::app::state::TextInputTarget::Control(ControlId::Param(
            ParamKey::BackgroundHue,
        )),
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

#[test]
fn source_picker_routes_within_its_modal_subtree() {
    let mut state = AppState::new().expect("state");
    state.ui.source_picker = Some(crate::app::state::SourcePickerState {
        control: ControlId::Reference(TokenRole::Background, ReferenceField::AliasSource),
        filter: String::new(),
        selected: 0,
    });
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::InspectorPanel,
        SurfaceId::SourcePicker,
    ];
    state.ui.interaction.push_mode(InteractionMode::Modal {
        owner: SurfaceId::SourcePicker,
    });

    let tree = build_interaction_tree(&state);
    let intents = route_ui_action(&state, UiAction::NextTab);

    assert_eq!(
        tree.parent_of(SurfaceId::SourcePicker),
        Some(SurfaceId::InspectorPanel)
    );
    assert!(intents.is_empty());
}

#[test]
fn source_picker_blocks_main_window_shortcuts_while_modal_is_open() {
    let mut state = AppState::new().expect("state");
    state.ui.source_picker = Some(crate::app::state::SourcePickerState {
        control: ControlId::Reference(TokenRole::Background, ReferenceField::AliasSource),
        filter: String::new(),
        selected: 0,
    });
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::InspectorPanel,
        SurfaceId::SourcePicker,
    ];
    state.ui.interaction.push_mode(InteractionMode::Modal {
        owner: SurfaceId::SourcePicker,
    });

    let intents = route_ui_action(&state, UiAction::OpenConfig);

    assert!(intents.is_empty());
}

#[test]
fn effective_focus_path_does_not_append_modal_owner() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_panel(PanelId::Params);
    state.ui.interaction.push_mode(InteractionMode::Modal {
        owner: SurfaceId::NumericEditorSurface,
    });

    assert_eq!(
        effective_focus_path(&state),
        vec![
            SurfaceId::AppRoot,
            SurfaceId::MainWindow,
            SurfaceId::ParamsPanel,
        ]
    );
}

#[test]
fn effective_focus_path_uses_stack_owner_instead_of_ui_flags() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_panel(PanelId::Preview);
    state.preview.capture_active = true;

    assert_eq!(
        effective_focus_path(&state),
        vec![
            SurfaceId::AppRoot,
            SurfaceId::MainWindow,
            SurfaceId::PreviewPanel,
        ]
    );

    state.ui.interaction.push_mode(InteractionMode::Capture {
        owner: SurfaceId::PreviewBody,
    });

    assert_eq!(
        effective_focus_path(&state),
        vec![
            SurfaceId::AppRoot,
            SurfaceId::MainWindow,
            SurfaceId::PreviewPanel,
            SurfaceId::PreviewBody,
        ]
    );
}

fn collect_panels(node: &ViewNode, panels: &mut Vec<(PanelId, Option<u8>)>) {
    match node {
        ViewNode::Split(split) => {
            for child in &split.children {
                collect_panels(child, panels);
            }
        }
        ViewNode::Panel(panel) => panels.push((panel.id, panel.shortcut)),
        ViewNode::StatusBar(_) => {}
    }
}
