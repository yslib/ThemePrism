mod chrome;
mod content;
mod overlays;
mod panels;
mod style;

#[cfg(test)]
mod tests;

use crate::app::actions::ActionHint;
use crate::app::controls::ControlSpec;
use crate::app::view::{
    Axis, DocumentView, FormFieldView, FormView, MainWindowView, MenuBarView, OverlayView,
    PanelBody, PanelView, PickerOverlayView, SelectionListView, SelectionRowView, Size, SpanStyle,
    StatusBarView, StyledLine, StyledSpan, SurfaceBody, SurfaceSize, SurfaceView, SwatchItemView,
    SwatchListView, TabBarView, ViewNode, ViewTheme, ViewTree,
};
use crate::app::workspace::PanelId;
use crate::domain::color::Color;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

#[derive(Debug, Default, Clone, Copy)]
pub struct TuiRenderer;

impl TuiRenderer {
    pub fn present(self, frame: &mut Frame, tree: &ViewTree) {
        self.render_main_window(frame, frame.area(), &tree.main_window, &tree.theme);

        for overlay in &tree.overlays {
            self.render_overlay(frame, frame.area(), overlay, &tree.theme);
        }
    }

    fn render_main_window(
        self,
        frame: &mut Frame,
        area: Rect,
        window: &MainWindowView,
        theme: &ViewTheme,
    ) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(area);

        self.render_menu_bar(
            frame,
            sections[0],
            &window.menu_bar,
            theme,
            window.hint_navigation_active,
        );
        self.render_tab_bar_with_hint_state(
            frame,
            sections[1],
            &window.tab_bar,
            theme,
            window.hint_navigation_active,
        );
        if let Some(panel) = window.fullscreen_panel {
            if let Some(fullscreen_panel) = find_panel_view(&window.workspace, panel) {
                self.render_panel(frame, sections[2], fullscreen_panel, theme);
            } else {
                self.render_node(frame, sections[2], &window.workspace, theme);
            }
        } else {
            self.render_node(frame, sections[2], &window.workspace, theme);
        }
        self.render_status_bar(
            frame,
            sections[3],
            &window.status_bar,
            theme,
            window.hint_navigation_active,
        );
    }

    fn render_node(self, frame: &mut Frame, area: Rect, node: &ViewNode, theme: &ViewTheme) {
        match node {
            ViewNode::Split(view) => {
                let layout = Layout::default()
                    .direction(match view.axis {
                        Axis::Horizontal => Direction::Horizontal,
                        Axis::Vertical => Direction::Vertical,
                    })
                    .constraints(view.constraints.iter().copied().map(to_constraint))
                    .split(area);

                for (child, child_area) in view.children.iter().zip(layout.iter().copied()) {
                    self.render_node(frame, child_area, child, theme);
                }
            }
            ViewNode::Panel(view) => self.render_panel(frame, area, view, theme),
            ViewNode::StatusBar(view) => self.render_status_bar(frame, area, view, theme, false),
        }
    }
}

pub(crate) fn panel_body_area(panel: &PanelView, area: Rect) -> Rect {
    panels::panel_body_area(panel, area)
}

pub(crate) fn max_document_scroll(paragraph: &Paragraph<'_>, area: Rect) -> u16 {
    content::max_document_scroll(paragraph, area)
}

fn find_panel_view(node: &ViewNode, target: PanelId) -> Option<&PanelView> {
    match node {
        ViewNode::Split(view) => view
            .children
            .iter()
            .find_map(|child| find_panel_view(child, target)),
        ViewNode::Panel(view) if view.id == target => Some(view),
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
