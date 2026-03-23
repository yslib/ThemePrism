use crate::app::Intent;
use crate::app::state::AppState;

use super::{
    BubblePolicy, ChildNavigation, DefaultAction, InteractionMode, InteractionTree, SurfaceId,
    SurfaceNode, TabScope, UiAction, build_interaction_tree, effective_focus_path,
};

pub fn route_ui_action(state: &AppState, action: UiAction) -> Vec<Intent> {
    let tree = build_interaction_tree(state);
    let focus_path = active_routing_path(state);

    route_from_focus(&tree, state, &focus_path, action)
}

fn active_routing_path(state: &AppState) -> Vec<SurfaceId> {
    let focus_path = effective_focus_path(state);

    match state.ui.interaction.current_mode() {
        InteractionMode::Modal { owner } => focus_path
            .iter()
            .position(|surface| *surface == owner)
            .map(|index| focus_path[index..].to_vec())
            .unwrap_or_else(|| vec![owner]),
        _ => focus_path,
    }
}

fn route_from_focus(
    tree: &InteractionTree,
    state: &AppState,
    focus_path: &[SurfaceId],
    action: UiAction,
) -> Vec<Intent> {
    for &surface in focus_path.iter().rev() {
        let Some(node) = tree.node(surface) else {
            continue;
        };

        if let Some(intents) = route_on_node(tree, state, node, action) {
            return intents;
        }

        if node.bubble_policy == BubblePolicy::Stop {
            break;
        }
    }

    Vec::new()
}

fn route_on_node(
    tree: &InteractionTree,
    state: &AppState,
    node: &SurfaceNode,
    action: UiAction,
) -> Option<Vec<Intent>> {
    match action {
        UiAction::PreviousTab => route_tab_action(tree, node, -1),
        UiAction::NextTab => route_tab_action(tree, node, 1),
        UiAction::Activate => route_default_action(state, node)
            .or_else(|| route_surface_action(state, node.id, action)),
        UiAction::Cancel => route_child_navigation_cancel(tree, state, node)
            .or_else(|| route_surface_action(state, node.id, action)),
        UiAction::MoveUp | UiAction::MoveLeft => route_child_navigation_move(tree, state, node, -1)
            .or_else(|| route_surface_action(state, node.id, action)),
        UiAction::MoveDown | UiAction::MoveRight => {
            route_child_navigation_move(tree, state, node, 1)
                .or_else(|| route_surface_action(state, node.id, action))
        }
        UiAction::SelectChild(number) => route_select_child(state, node, number),
        _ => route_surface_action(state, node.id, action),
    }
}

fn route_tab_action(tree: &InteractionTree, node: &SurfaceNode, delta: i32) -> Option<Vec<Intent>> {
    if let Some(owner) = node.tab_scope_owner {
        return tree
            .node(owner)
            .and_then(|owner_node| route_tab_action(tree, owner_node, delta));
    }

    match node.tab_scope {
        TabScope::Global if node.id == SurfaceId::MainWindow => {
            Some(vec![Intent::CycleWorkspaceTab(delta)])
        }
        TabScope::PreviewLocal => Some(vec![Intent::CyclePreviewMode(delta)]),
        TabScope::Global | TabScope::Workspace(_) | TabScope::Modal => None,
    }
}

fn route_default_action(state: &AppState, node: &SurfaceNode) -> Option<Vec<Intent>> {
    match node.default_action {
        DefaultAction::None => None,
        DefaultAction::Activate | DefaultAction::Edit => route_activate_like_action(state, node),
        DefaultAction::Open => route_open_action(node.id),
    }
}

fn route_activate_like_action(state: &AppState, node: &SurfaceNode) -> Option<Vec<Intent>> {
    if let Some(intents) = enter_child_navigation(node) {
        return Some(intents);
    }

    if let Some(control) = state.active_control() {
        return Some(vec![Intent::ActivateControl(control)]);
    }

    if state.active_config_field().is_some() {
        return Some(vec![Intent::ActivateConfigField]);
    }

    None
}

fn route_open_action(surface: SurfaceId) -> Option<Vec<Intent>> {
    match surface {
        SurfaceId::PreviewBody => Some(vec![Intent::SetPreviewCapture(true)]),
        _ => None,
    }
}

fn enter_child_navigation(node: &SurfaceNode) -> Option<Vec<Intent>> {
    match node.child_navigation {
        ChildNavigation::None => None,
        ChildNavigation::Numbered => Some(vec![
            Intent::FocusSurface(node.id),
            Intent::SetInteractionMode(InteractionMode::NavigateChildren(node.id)),
        ]),
        ChildNavigation::Sequential => node.children.first().copied().map(|child| {
            vec![
                Intent::FocusSurface(child),
                Intent::SetInteractionMode(InteractionMode::NavigateChildren(node.id)),
            ]
        }),
    }
}

fn route_child_navigation_cancel(
    tree: &InteractionTree,
    state: &AppState,
    node: &SurfaceNode,
) -> Option<Vec<Intent>> {
    let owner = match state.ui.interaction.current_mode() {
        InteractionMode::NavigateChildren(owner) => owner,
        _ => return None,
    };

    let owner_node = resolve_child_navigation_owner(tree, node, owner)?;
    if owner_node.child_navigation != ChildNavigation::None {
        return Some(vec![
            Intent::FocusSurface(owner_node.id),
            Intent::SetInteractionMode(InteractionMode::Normal),
        ]);
    }

    None
}

fn route_child_navigation_move(
    tree: &InteractionTree,
    state: &AppState,
    node: &SurfaceNode,
    delta: i32,
) -> Option<Vec<Intent>> {
    let owner = match state.ui.interaction.current_mode() {
        InteractionMode::NavigateChildren(owner) => owner,
        _ => return None,
    };

    let owner_node = resolve_child_navigation_owner(tree, node, owner)?;
    if owner_node.child_navigation != ChildNavigation::Sequential {
        return None;
    }

    let focused = state.ui.interaction.focused_surface();
    let index = owner_node
        .children
        .iter()
        .position(|child| *child == focused)?;
    let next = if delta < 0 {
        index.saturating_sub(1)
    } else {
        (index + 1).min(owner_node.children.len().saturating_sub(1))
    };

    if next == index {
        Some(Vec::new())
    } else {
        Some(vec![Intent::FocusSurface(owner_node.children[next])])
    }
}

fn resolve_child_navigation_owner<'a>(
    tree: &'a InteractionTree,
    node: &SurfaceNode,
    owner: SurfaceId,
) -> Option<&'a SurfaceNode> {
    if node.id == owner {
        return tree.node(owner);
    }

    if tree.parent_of(node.id) == Some(owner) {
        return tree.node(owner);
    }

    None
}

fn route_select_child(state: &AppState, node: &SurfaceNode, number: u8) -> Option<Vec<Intent>> {
    if state.ui.interaction.current_mode() != InteractionMode::NavigateChildren(node.id) {
        return None;
    }

    match node.child_navigation {
        ChildNavigation::Numbered if node.id == SurfaceId::MainWindow => Some(vec![
            Intent::FocusPanelByNumber(number),
            Intent::SetInteractionMode(InteractionMode::Normal),
        ]),
        ChildNavigation::None | ChildNavigation::Sequential | ChildNavigation::Numbered => None,
    }
}

fn route_surface_action(
    state: &AppState,
    surface: SurfaceId,
    action: UiAction,
) -> Option<Vec<Intent>> {
    if surface.panel_id().is_some() {
        return route_panel_action(state, surface, action);
    }

    match surface {
        SurfaceId::AppRoot => None,
        SurfaceId::MainWindow => route_main_window_action(action),
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

fn route_main_window_action(action: UiAction) -> Option<Vec<Intent>> {
    match action {
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

fn route_panel_action(
    state: &AppState,
    _surface: SurfaceId,
    action: UiAction,
) -> Option<Vec<Intent>> {
    match action {
        UiAction::Cancel => Some(vec![
            Intent::FocusSurface(SurfaceId::MainWindow),
            Intent::SetInteractionMode(InteractionMode::Normal),
        ]),
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
