use super::{Axis, PanelView, Size, SplitView, StatusBarView, ViewNode};

// A small declarative layout DSL for the TUI workspace.
//
// Keep this as plain Rust instead of proc-macros or codegen so layout changes stay easy to
// debug, grep, and refactor. The intended editing loop is:
//
// 1. Pick a slot from `WorkspaceSlot`
// 2. Wrap it with `panel(...)`
// 3. Compose rows / columns with `row(...)` and `column(...)`
// 4. Assign sizes through `child(Size::..., ...)`
//
// Example:
//
// column(vec![
//     child(Size::Min(12), row(vec![
//         child(Size::Length(32), panel(WorkspaceSlot::Tokens)),
//         child(Size::Min(48), panel(WorkspaceSlot::Preview)),
//     ])),
//     child(Size::Length(2), status_bar()),
// ])

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSlot {
    Tokens,
    Params,
    Preview,
    Palette,
    ResolvedPrimary,
    ResolvedSecondary,
    Inspector,
}

#[derive(Debug, Clone)]
pub struct LayoutChild {
    pub size: Size,
    pub node: WorkspaceLayout,
}

#[derive(Debug, Clone)]
pub enum WorkspaceLayout {
    Row(Vec<LayoutChild>),
    Column(Vec<LayoutChild>),
    Panel(WorkspaceSlot),
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

pub fn panel(slot: WorkspaceSlot) -> WorkspaceLayout {
    WorkspaceLayout::Panel(slot)
}

pub fn status_bar() -> WorkspaceLayout {
    WorkspaceLayout::StatusBar
}

pub fn default_workspace_layout() -> WorkspaceLayout {
    column(vec![
        child(
            Size::Min(12),
            row(vec![
                child(
                    Size::Length(34),
                    column(vec![
                        child(Size::Percentage(58), panel(WorkspaceSlot::Tokens)),
                        child(Size::Percentage(42), panel(WorkspaceSlot::Params)),
                    ]),
                ),
                child(
                    Size::Min(48),
                    column(vec![
                        child(Size::Percentage(45), panel(WorkspaceSlot::Preview)),
                        child(Size::Percentage(28), panel(WorkspaceSlot::Palette)),
                        child(
                            Size::Percentage(27),
                            row(vec![
                                child(Size::Percentage(50), panel(WorkspaceSlot::ResolvedPrimary)),
                                child(
                                    Size::Percentage(50),
                                    panel(WorkspaceSlot::ResolvedSecondary),
                                ),
                            ]),
                        ),
                    ]),
                ),
                child(Size::Length(38), panel(WorkspaceSlot::Inspector)),
            ]),
        ),
        child(Size::Length(2), status_bar()),
    ])
}

#[allow(dead_code)]
pub fn preview_focus_layout() -> WorkspaceLayout {
    column(vec![
        child(
            Size::Min(12),
            row(vec![
                child(
                    Size::Length(34),
                    column(vec![
                        child(Size::Percentage(55), panel(WorkspaceSlot::Tokens)),
                        child(Size::Percentage(45), panel(WorkspaceSlot::Params)),
                    ]),
                ),
                child(Size::Min(56), panel(WorkspaceSlot::Preview)),
                child(
                    Size::Length(36),
                    column(vec![
                        child(Size::Percentage(38), panel(WorkspaceSlot::Inspector)),
                        child(Size::Percentage(28), panel(WorkspaceSlot::Palette)),
                        child(Size::Percentage(17), panel(WorkspaceSlot::ResolvedPrimary)),
                        child(
                            Size::Percentage(17),
                            panel(WorkspaceSlot::ResolvedSecondary),
                        ),
                    ]),
                ),
            ]),
        ),
        child(Size::Length(2), status_bar()),
    ])
}

pub fn compose_layout<P, S>(
    layout: &WorkspaceLayout,
    panel_for_slot: &mut P,
    status_bar_view: &mut S,
) -> ViewNode
where
    P: FnMut(WorkspaceSlot) -> PanelView,
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
