use crate::app::ui_meta::panel_spec;
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
    let layout = match tab {
        WorkspaceTab::Theme => default_workspace_layout(),
        WorkspaceTab::Project => project_workspace_layout(),
    };

    debug_assert!(
        panel_order(&layout)
            .into_iter()
            .all(|panel| panel_spec(panel).workspace_tab == tab),
        "workspace layout should only contain panels for its tab"
    );

    layout
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
                child(Size::Percentage(72), panel(PanelId::Preview)),
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
                child(Size::Percentage(32), panel(PanelId::Inspector)),
                child(Size::Percentage(28), panel(PanelId::InteractionInspector)),
                child(Size::Percentage(40), panel(PanelId::Palette)),
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

pub fn visible_panels_for_tab(tab: WorkspaceTab) -> Vec<PanelId> {
    panel_order(&workspace_layout_for_tab(tab))
}

pub fn panel_order(layout: &WorkspaceLayout) -> Vec<PanelId> {
    let mut panels = Vec::new();
    collect_panel_order(layout, &mut panels);
    panels
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
