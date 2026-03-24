use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::view::{Axis, MainWindowView, PanelView, Size, ViewNode, ViewTree};
use crate::app::workspace::PanelId;
use crate::platform::tui::renderer::panel_body_area;

pub(crate) fn locate_panel_body(
    tree: &ViewTree,
    area: Rect,
    target: PanelId,
) -> Option<(&PanelView, Rect)> {
    let workspace_area = main_window_workspace_area(area);
    locate_panel_in_window(&tree.main_window, workspace_area, target)
        .map(|(panel, panel_area)| (panel, panel_body_area(panel, panel_area)))
}

pub(crate) fn main_window_workspace_area(area: Rect) -> Rect {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(area);
    sections[2]
}

fn locate_panel_in_window<'a>(
    window: &'a MainWindowView,
    area: Rect,
    target: PanelId,
) -> Option<(&'a PanelView, Rect)> {
    if let Some(fullscreen) = window.fullscreen_panel {
        if fullscreen != target {
            return None;
        }
        return find_panel_view(&window.workspace, target).map(|panel| (panel, area));
    }

    locate_panel_in_node(&window.workspace, area, target)
}

fn locate_panel_in_node<'a>(
    node: &'a ViewNode,
    area: Rect,
    target: PanelId,
) -> Option<(&'a PanelView, Rect)> {
    match node {
        ViewNode::Split(split) => {
            let sections = Layout::default()
                .direction(match split.axis {
                    Axis::Horizontal => Direction::Horizontal,
                    Axis::Vertical => Direction::Vertical,
                })
                .constraints(split.constraints.iter().copied().map(to_constraint))
                .split(area);

            split
                .children
                .iter()
                .zip(sections.iter().copied())
                .find_map(|(child, child_area)| locate_panel_in_node(child, child_area, target))
        }
        ViewNode::Panel(panel) if panel.id == target => Some((panel, area)),
        ViewNode::Panel(_) | ViewNode::StatusBar(_) => None,
    }
}

fn find_panel_view(node: &ViewNode, target: PanelId) -> Option<&PanelView> {
    match node {
        ViewNode::Split(split) => split
            .children
            .iter()
            .find_map(|child| find_panel_view(child, target)),
        ViewNode::Panel(panel) if panel.id == target => Some(panel),
        ViewNode::Panel(_) | ViewNode::StatusBar(_) => None,
    }
}

fn to_constraint(size: Size) -> Constraint {
    match size {
        Size::Length(value) => Constraint::Length(value),
        Size::Min(value) => Constraint::Min(value),
        Size::Percentage(value) => Constraint::Percentage(value),
    }
}

#[cfg(test)]
mod tests {
    use super::locate_panel_body;
    use crate::app::AppState;
    use crate::app::interaction::SurfaceId;
    use crate::app::view::build_view;
    use crate::app::workspace::PanelId;
    use ratatui::layout::Rect;

    #[test]
    fn locate_panel_body_uses_fullscreen_geometry_when_enabled() {
        let mut state = AppState::new().expect("state");
        state.ui.fullscreen_surface = Some(SurfaceId::PreviewPanel);
        let view = build_view(&state);

        let (_, body) =
            locate_panel_body(&view, Rect::new(0, 0, 120, 40), PanelId::Preview).expect("panel");

        assert_eq!(body.x, 1);
        assert_eq!(body.y, 5);
        assert_eq!(body.width, 118);
        assert_eq!(body.height, 33);
    }

    #[test]
    fn locate_panel_body_hides_non_fullscreen_panels_while_fullscreen_is_active() {
        let mut state = AppState::new().expect("state");
        state.ui.fullscreen_surface = Some(SurfaceId::PreviewPanel);
        let view = build_view(&state);

        assert!(locate_panel_body(&view, Rect::new(0, 0, 120, 40), PanelId::Inspector).is_none());
    }
}
