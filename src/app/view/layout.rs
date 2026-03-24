use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::workspace::{PanelId, WorkspaceTab};

use super::{Axis, PanelView, Size, SplitView, StatusBarView, ViewNode};

// A small declarative layout DSL for the TUI workspace.
//
// Keep this as plain Rust instead of proc-macros or codegen so layout changes stay easy to
// debug, grep, and refactor. The intended editing loop is:
//
// 1. Pick a `PanelId`
// 2. Wrap it with `panel(...)`
// 3. Compose rows / columns with `row(...)` and `column(...)`
// 4. Assign sizes through `child(Size::..., ...)`

#[derive(Debug, Clone)]
pub struct LayoutChild {
    pub size: Size,
    pub node: WorkspaceLayout,
}

#[derive(Debug, Clone)]
pub enum WorkspaceLayout {
    Row(Vec<LayoutChild>),
    Column(Vec<LayoutChild>),
    Panel(PanelId),
    #[allow(dead_code)]
    StatusBar,
}

pub fn child(size: Size, node: WorkspaceLayout) -> LayoutChild {
    LayoutChild { size, node }
}

pub fn row(children: impl Into<Vec<LayoutChild>>) -> WorkspaceLayout {
    WorkspaceLayout::Row(children.into())
}

pub fn column(children: impl Into<Vec<LayoutChild>>) -> WorkspaceLayout {
    WorkspaceLayout::Column(children.into())
}

pub fn panel(id: PanelId) -> WorkspaceLayout {
    WorkspaceLayout::Panel(id)
}

#[allow(dead_code)]
pub fn status_bar() -> WorkspaceLayout {
    WorkspaceLayout::StatusBar
}

pub fn workspace_layout_for_tab(tab: WorkspaceTab) -> WorkspaceLayout {
    match tab {
        WorkspaceTab::Theme => default_workspace_layout(),
        WorkspaceTab::Project => project_workspace_layout(),
    }
}

pub fn default_workspace_layout() -> WorkspaceLayout {
    row(vec![
        child(
            Size::Length(34),
            column(vec![
                child(Size::Percentage(58), panel(PanelId::Tokens)),
                child(Size::Percentage(42), panel(PanelId::Params)),
            ]),
        ),
        child(
            Size::Min(48),
            column(vec![
                child(Size::Percentage(28), panel(PanelId::Palette)),
                child(
                    Size::Percentage(27),
                    row(vec![
                        child(Size::Percentage(50), panel(PanelId::ResolvedPrimary)),
                        child(Size::Percentage(50), panel(PanelId::ResolvedSecondary)),
                    ]),
                ),
                child(Size::Percentage(45), panel(PanelId::Preview)),
            ]),
        ),
        child(
            Size::Length(40),
            column(vec![
                child(Size::Percentage(50), panel(PanelId::Inspector)),
                child(Size::Percentage(50), panel(PanelId::InteractionInspector)),
            ]),
        ),
    ])
}

#[allow(dead_code)]
pub fn preview_focus_layout() -> WorkspaceLayout {
    row(vec![
        child(
            Size::Length(34),
            column(vec![
                child(Size::Percentage(55), panel(PanelId::Tokens)),
                child(Size::Percentage(45), panel(PanelId::Params)),
            ]),
        ),
        child(Size::Min(56), panel(PanelId::Preview)),
        child(
            Size::Length(36),
            column(vec![
                child(Size::Percentage(28), panel(PanelId::Inspector)),
                child(Size::Percentage(24), panel(PanelId::InteractionInspector)),
                child(Size::Percentage(24), panel(PanelId::Palette)),
                child(Size::Percentage(12), panel(PanelId::ResolvedPrimary)),
                child(Size::Percentage(12), panel(PanelId::ResolvedSecondary)),
            ]),
        ),
    ])
}

pub fn project_workspace_layout() -> WorkspaceLayout {
    row(vec![
        child(Size::Length(32), panel(PanelId::ProjectConfig)),
        child(
            Size::Min(46),
            column(vec![
                child(Size::Percentage(68), panel(PanelId::ExportTargets)),
                child(Size::Percentage(32), panel(PanelId::EditorPreferences)),
            ]),
        ),
    ])
}

pub fn panel_order(layout: &WorkspaceLayout) -> Vec<PanelId> {
    let mut panels = Vec::new();
    collect_panel_order(layout, &mut panels);
    panels
}

pub fn panel_area(layout: &WorkspaceLayout, area: Rect, target: PanelId) -> Option<Rect> {
    match layout {
        WorkspaceLayout::Row(children) | WorkspaceLayout::Column(children) => {
            let sections = Layout::default()
                .direction(match layout {
                    WorkspaceLayout::Row(_) => Direction::Horizontal,
                    WorkspaceLayout::Column(_) => Direction::Vertical,
                    WorkspaceLayout::Panel(_) | WorkspaceLayout::StatusBar => unreachable!(),
                })
                .constraints(children.iter().map(|child| to_constraint(child.size)))
                .split(area);

            children
                .iter()
                .zip(sections.iter().copied())
                .find_map(|(child, child_area)| panel_area(&child.node, child_area, target))
        }
        WorkspaceLayout::Panel(id) => (*id == target).then_some(area),
        WorkspaceLayout::StatusBar => None,
    }
}

fn collect_panel_order(layout: &WorkspaceLayout, panels: &mut Vec<PanelId>) {
    match layout {
        WorkspaceLayout::Row(children) | WorkspaceLayout::Column(children) => {
            for child in children {
                collect_panel_order(&child.node, panels);
            }
        }
        WorkspaceLayout::Panel(id) => panels.push(*id),
        WorkspaceLayout::StatusBar => {}
    }
}

fn to_constraint(size: Size) -> Constraint {
    match size {
        Size::Length(value) => Constraint::Length(value),
        Size::Min(value) => Constraint::Min(value),
        Size::Percentage(value) => Constraint::Percentage(value),
    }
}

pub fn compose_layout<P, S>(
    layout: &WorkspaceLayout,
    panel_for_slot: &mut P,
    status_bar_view: &mut S,
) -> ViewNode
where
    P: FnMut(PanelId) -> PanelView,
    S: FnMut() -> StatusBarView,
{
    match layout {
        WorkspaceLayout::Row(children) => ViewNode::Split(SplitView {
            axis: Axis::Horizontal,
            constraints: children.iter().map(|child| child.size).collect(),
            children: children
                .iter()
                .map(|child| compose_layout(&child.node, panel_for_slot, status_bar_view))
                .collect(),
        }),
        WorkspaceLayout::Column(children) => ViewNode::Split(SplitView {
            axis: Axis::Vertical,
            constraints: children.iter().map(|child| child.size).collect(),
            children: children
                .iter()
                .map(|child| compose_layout(&child.node, panel_for_slot, status_bar_view))
                .collect(),
        }),
        WorkspaceLayout::Panel(slot) => ViewNode::Panel(panel_for_slot(*slot)),
        WorkspaceLayout::StatusBar => ViewNode::StatusBar(status_bar_view()),
    }
}
