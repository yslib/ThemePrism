use crate::app::actions::ActionHint;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::controls::ControlSpec;
use crate::app::view::{
    Axis, DocumentView, FormFieldView, FormView, MainWindowView, MenuBarView, OverlayView,
    PanelBody, PanelView, PickerOverlayView, SelectionListView, SelectionRowView, Size, SpanStyle,
    StatusBarView, StyledLine, StyledSpan, SurfaceBody, SurfaceSize, SurfaceView, SwatchItemView,
    SwatchListView, TabBarView, ViewNode, ViewTheme, ViewTree,
};
use crate::app::workspace::PanelId;
use crate::domain::color::Color;

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
        self.render_tab_bar(frame, sections[1], &window.tab_bar, theme);
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

    fn render_menu_bar(
        self,
        frame: &mut Frame,
        area: Rect,
        view: &MenuBarView,
        theme: &ViewTheme,
        hint_navigation_active: bool,
    ) {
        let mut spans = vec![Span::styled(
            format!(" {} ", view.title),
            Style::default()
                .bg(tui(theme.selection))
                .fg(tui(theme.background))
                .add_modifier(Modifier::BOLD),
        )];
        let title_width = view.title.chars().count() + 2;
        let available = area.width as usize;
        if available > title_width + 2 {
            spans.push(Span::raw("  "));
            spans.extend(fitting_action_spans(
                &view.actions,
                available - title_width - 2,
                theme,
                hint_navigation_active,
            ));
        }

        frame.render_widget(
            Paragraph::new(Line::from(spans))
                .style(Style::default().bg(tui(theme.surface)).fg(tui(theme.text))),
            area,
        );
    }

    fn render_tab_bar(self, frame: &mut Frame, area: Rect, view: &TabBarView, theme: &ViewTheme) {
        let hint_navigation_active = view.tabs.iter().any(|tab| tab.shortcut.is_some());
        let mut spans = Vec::new();
        for (index, tab) in view.tabs.iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled(
                    "  ",
                    Style::default().bg(tui(theme.background)),
                ));
            }

            if let Some(shortcut) = tab.shortcut {
                spans.push(Span::styled(
                    format!(" [{}] ", shortcut),
                    hint_shortcut_style(theme),
                ));
                let label_style = hint_tab_label_style(tab.selected, theme, true);
                spans.push(Span::styled(tab.label.clone(), label_style));
                spans.push(Span::styled(" ", label_style));
            } else {
                let style = hint_tab_label_style(tab.selected, theme, false);
                let style = if hint_navigation_active {
                    style.add_modifier(Modifier::DIM)
                } else {
                    style
                };
                spans.push(Span::styled(format!(" {} ", tab.label), style));
            }
        }

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(
                Style::default()
                    .bg(tui(theme.background))
                    .fg(tui(theme.text)),
            ),
            area,
        );
    }

    fn render_panel(self, frame: &mut Frame, area: Rect, view: &PanelView, theme: &ViewTheme) {
        let border = panel_border_color(view, theme);
        let block = Block::default()
            .title(panel_title_line(view, theme))
            .borders(Borders::ALL)
            .border_style(panel_border_style(view, border));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let sections = panel_body_sections(view, inner);

        if !view.tabs.is_empty() {
            self.render_panel_tabs(frame, sections[0], &view.tabs, theme);
        }

        if !view.header_lines.is_empty() {
            let header = view
                .header_lines
                .iter()
                .map(|line| styled_line_to_tui(line, theme, view.hint_navigation_active))
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(header)
                    .wrap(Wrap { trim: false })
                    .style(hint_content_style(view.hint_navigation_active)),
                sections[1],
            );
        }

        match &view.body {
            PanelBody::SelectionList(list) => self.render_selection_list(
                frame,
                sections[2],
                list,
                theme,
                view.hint_navigation_active,
            ),
            PanelBody::Form(form) => {
                self.render_form(frame, sections[2], form, theme, view.hint_navigation_active)
            }
            PanelBody::Document(document) => {
                self.render_document(frame, sections[2], document, view.hint_navigation_active)
            }
            PanelBody::SwatchList(list) => self.render_swatch_list(
                frame,
                sections[2],
                list,
                theme,
                view.hint_navigation_active,
            ),
        }
    }

    fn render_panel_tabs(
        self,
        frame: &mut Frame,
        area: Rect,
        tabs: &[crate::app::view::PanelTabView],
        theme: &ViewTheme,
    ) {
        let hint_navigation_active = tabs.iter().any(|tab| tab.shortcut.is_some());
        let mut spans = Vec::new();
        for (index, tab) in tabs.iter().enumerate() {
            if index > 0 {
                spans.push(Span::raw(" "));
            }

            if let Some(shortcut) = tab.shortcut {
                spans.push(Span::styled(
                    format!(" [{}] ", shortcut),
                    hint_shortcut_style(theme),
                ));
                let label_style = hint_panel_tab_label_style(tab.active, theme, true);
                spans.push(Span::styled(tab.label.clone(), label_style));
                spans.push(Span::styled(" ", label_style));
            } else {
                let style = hint_panel_tab_label_style(tab.active, theme, false);
                let style = if hint_navigation_active {
                    style.add_modifier(Modifier::DIM)
                } else {
                    style
                };
                spans.push(Span::styled(format!(" {} ", tab.label), style));
            }
        }

        frame.render_widget(
            Paragraph::new(Line::from(spans))
                .style(Style::default().bg(tui(theme.surface)).fg(tui(theme.text))),
            area,
        );
    }

    fn render_selection_list(
        self,
        frame: &mut Frame,
        area: Rect,
        list: &SelectionListView,
        theme: &ViewTheme,
        hint_navigation_active: bool,
    ) {
        let lines = list
            .rows
            .iter()
            .map(|row| match row {
                SelectionRowView::Header(label) => Line::from(Span::styled(
                    label.as_str(),
                    Style::default()
                        .fg(tui(theme.text_muted))
                        .add_modifier(Modifier::BOLD),
                )),
                SelectionRowView::Item {
                    label,
                    color,
                    selected,
                } => {
                    let style = if *selected {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .bg(tui(theme.selection))
                            .fg(tui(theme.background))
                    } else {
                        Style::default().fg(tui(theme.text))
                    };

                    Line::from(vec![
                        Span::styled(if *selected { "> " } else { "  " }, style),
                        Span::styled("■ ", Style::default().fg(tui(*color))),
                        Span::styled(label.as_str(), style),
                    ])
                }
            })
            .collect::<Vec<_>>();

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .style(hint_content_style(hint_navigation_active)),
            area,
        );
    }

    fn render_form(
        self,
        frame: &mut Frame,
        area: Rect,
        form: &FormView,
        theme: &ViewTheme,
        hint_navigation_active: bool,
    ) {
        let mut lines = form
            .header_lines
            .iter()
            .map(|line| styled_line_to_tui(line, theme, hint_navigation_active))
            .collect::<Vec<_>>();

        for field in &form.fields {
            lines.push(self.render_form_field(field, theme));
        }

        if let Some(footer) = &form.footer {
            if !lines.is_empty() {
                lines.push(Line::raw(""));
            }
            lines.push(Line::from(Span::styled(
                footer.as_str(),
                Style::default()
                    .fg(tui(theme.text_muted))
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .style(hint_content_style(hint_navigation_active)),
            area,
        );
    }

    fn render_form_field(self, field: &FormFieldView, theme: &ViewTheme) -> Line<'static> {
        let style = if field.selected {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(tui(theme.selection))
                .fg(tui(theme.background))
        } else {
            Style::default().fg(tui(theme.text))
        };

        let prefix = if field.selected { "> " } else { "  " };
        let label = format!("{:<14}", field.control.label());
        let value = field.control.value_text();

        let mut spans = vec![Span::styled(prefix, style), Span::styled(label, style)];

        if let Some(color) = field.control.swatch() {
            spans.push(Span::styled("    ", Style::default().bg(tui(color))));
            spans.push(Span::raw(" "));
        }

        spans.push(Span::styled(value.to_string(), style));

        if matches!(&field.control, ControlSpec::ReferencePicker(spec) if spec.picker_open) {
            spans.push(Span::styled(
                "  [picker]",
                Style::default().fg(tui(theme.text_muted)),
            ));
        }

        Line::from(spans)
    }

    fn render_document(
        self,
        frame: &mut Frame,
        area: Rect,
        document: &DocumentView,
        hint_navigation_active: bool,
    ) {
        let lines = document
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .cloned()
                    .map(|span| to_tui_span(span, hint_navigation_active))
                    .collect::<Vec<_>>()
            })
            .map(Line::from)
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(hint_content_style(hint_navigation_active));
        let scroll = document.scroll.min(max_document_scroll(&paragraph, area));

        frame.render_widget(paragraph.scroll((scroll, 0)), area);
    }

    fn render_swatch_list(
        self,
        frame: &mut Frame,
        area: Rect,
        list: &SwatchListView,
        theme: &ViewTheme,
        hint_navigation_active: bool,
    ) {
        let lines = list
            .items
            .iter()
            .map(|item| self.render_swatch_item(item, theme))
            .collect::<Vec<_>>();

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .style(hint_content_style(hint_navigation_active)),
            area,
        );
    }

    fn render_swatch_item(self, item: &SwatchItemView, theme: &ViewTheme) -> Line<'static> {
        Line::from(vec![
            Span::styled("    ", Style::default().bg(tui(item.color))),
            Span::raw(" "),
            Span::styled(
                format!("{:<12}", item.label),
                Style::default().fg(tui(theme.text)),
            ),
            Span::styled(
                item.value_text.clone(),
                Style::default().fg(tui(theme.text_muted)),
            ),
        ])
    }

    fn render_status_bar(
        self,
        frame: &mut Frame,
        area: Rect,
        view: &StatusBarView,
        theme: &ViewTheme,
        hint_navigation_active: bool,
    ) {
        let line = Line::from(vec![
            Span::styled(
                format!("Focus: {}  ", view.focus_label),
                Style::default()
                    .fg(tui(theme.background))
                    .bg(tui(theme.selection))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                view.status_text.as_str(),
                if hint_navigation_active {
                    Style::default()
                        .fg(tui(theme.text_muted))
                        .add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(tui(theme.text_muted))
                },
            ),
        ]);

        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(tui(theme.surface)).fg(tui(theme.text))),
            area,
        );
    }

    fn render_overlay(
        self,
        frame: &mut Frame,
        area: Rect,
        overlay: &OverlayView,
        theme: &ViewTheme,
    ) {
        match overlay {
            OverlayView::Picker(view) => self.render_picker(frame, area, view, theme),
            OverlayView::Surface(view) => self.render_surface(frame, area, view, theme),
        }
    }

    fn render_picker(
        self,
        frame: &mut Frame,
        area: Rect,
        overlay: &PickerOverlayView,
        theme: &ViewTheme,
    ) {
        let area = centered_rect(58, 72, area);
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(overlay.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tui(theme.selection)))
            .style(Style::default().bg(tui(theme.surface)).fg(tui(theme.text)));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(6),
                Constraint::Length(2),
            ])
            .split(inner);

        let filter_value = if overlay.filter.is_empty() {
            "_".to_string()
        } else {
            format!("{}_", overlay.filter)
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Filter: ", Style::default().fg(tui(theme.text_muted))),
                Span::styled(filter_value, Style::default().fg(tui(theme.text))),
            ])),
            sections[0],
        );

        let items = overlay
            .rows
            .iter()
            .map(|row| {
                if row.is_header {
                    ListItem::new(Line::from(Span::styled(
                        row.label.as_str(),
                        Style::default()
                            .fg(tui(theme.text_muted))
                            .add_modifier(Modifier::BOLD),
                    )))
                } else {
                    ListItem::new(Line::from(Span::styled(
                        format!("  {}", row.label),
                        Style::default().fg(tui(theme.text)),
                    )))
                }
            })
            .collect::<Vec<_>>();
        let list = List::new(items).highlight_style(
            Style::default()
                .bg(tui(theme.selection))
                .fg(tui(theme.background))
                .add_modifier(Modifier::BOLD),
        );
        let mut list_state = ListState::default();
        list_state.select(overlay.selected_row);
        frame.render_stateful_widget(list, sections[1], &mut list_state);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{} matches", overlay.total_matches),
                    Style::default().fg(tui(theme.text_muted)),
                ),
                Span::raw("  "),
                Span::styled(
                    "Enter apply, Esc close",
                    Style::default().fg(tui(theme.text)),
                ),
            ])),
            sections[2],
        );
    }

    fn render_surface(
        self,
        frame: &mut Frame,
        area: Rect,
        surface: &SurfaceView,
        theme: &ViewTheme,
    ) {
        let area = match surface.size {
            SurfaceSize::Percent { width, height } => centered_rect(width, height, area),
            SurfaceSize::Absolute { width, height } => centered_rect_absolute(width, height, area),
        };
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(surface.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tui(theme.selection)))
            .style(Style::default().bg(tui(theme.surface)).fg(tui(theme.text)));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let footer_height = if surface.footer_lines.is_empty() {
            0
        } else {
            surface.footer_lines.len() as u16 + 1
        };
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(footer_height)])
            .split(inner);

        match &surface.body {
            SurfaceBody::Lines { lines, scroll } => {
                let body = lines
                    .iter()
                    .map(|line| styled_line_to_tui(line, theme, false))
                    .collect::<Vec<_>>();
                let paragraph = Paragraph::new(body).wrap(Wrap { trim: false });
                let scroll = (*scroll).min(max_document_scroll(&paragraph, sections[0]));
                frame.render_widget(paragraph.scroll((scroll, 0)), sections[0]);
            }
            SurfaceBody::Node(node) => self.render_node(frame, sections[0], node, theme),
            SurfaceBody::Window(window) => {
                self.render_main_window(frame, sections[0], window, theme)
            }
        }

        let footer = surface
            .footer_lines
            .iter()
            .map(|line| styled_line_to_tui(line, theme, false))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(footer).wrap(Wrap { trim: false }),
            sections[1],
        );
    }
}

pub(crate) fn panel_body_area(panel: &PanelView, area: Rect) -> Rect {
    let block = Block::default()
        .title(panel.title.as_str())
        .borders(Borders::ALL);
    let inner = block.inner(area);
    panel_body_sections(panel, inner)[2]
}

fn fitting_action_spans(
    actions: &[ActionHint],
    max_width: usize,
    theme: &ViewTheme,
    hint_navigation_active: bool,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut used_width = 0;

    for (index, action) in actions.iter().enumerate() {
        let action_width = action.shortcut.chars().count() + 1 + action.label.chars().count();
        let separator_width = if index > 0 { 2 } else { 0 };
        if used_width + separator_width + action_width > max_width {
            break;
        }
        if index > 0 {
            spans.push(Span::styled(
                "  ",
                Style::default().fg(tui(theme.text_muted)),
            ));
            used_width += 2;
        }
        spans.extend(action_hint_spans(
            action,
            Style::default()
                .fg(tui(theme.text_muted))
                .add_modifier(if hint_navigation_active {
                    Modifier::BOLD | Modifier::DIM
                } else {
                    Modifier::BOLD
                }),
            if hint_navigation_active {
                Style::default()
                    .fg(tui(theme.text_muted))
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(tui(theme.text))
            },
        ));
        used_width += action_width;
    }

    spans
}

fn action_hint_spans(
    action: &ActionHint,
    shortcut_style: Style,
    label_style: Style,
) -> Vec<Span<'static>> {
    vec![
        Span::styled(format!("{} ", action.shortcut), shortcut_style),
        Span::styled(action.label.clone(), label_style),
    ]
}

fn panel_body_sections(panel: &PanelView, inner: Rect) -> [Rect; 3] {
    let tab_height = u16::from(!panel.tabs.is_empty());
    let header_height = panel.header_lines.len() as u16;
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(tab_height),
            Constraint::Length(header_height),
            Constraint::Min(1),
        ])
        .split(inner);

    [sections[0], sections[1], sections[2]]
}

pub(crate) fn max_document_scroll(paragraph: &Paragraph<'_>, area: Rect) -> u16 {
    if area.height == 0 || area.width == 0 {
        return 0;
    }

    let rendered_line_count = paragraph.line_count(area.width);
    let max_scroll = rendered_line_count.saturating_sub(area.height as usize) as u16;
    max_scroll
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
    use super::{TuiRenderer, max_document_scroll, panel_title_line, tui};
    use crate::app::view::{
        DocumentView, PanelBody, PanelTabView, PanelView, TabBarView, TabItemView, ViewTheme,
    };
    use crate::app::workspace::PanelId;
    use crate::domain::color::Color;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Cell;
    use ratatui::layout::Rect;
    use ratatui::style::Modifier;
    use ratatui::widgets::{Paragraph, Wrap};

    fn sample_theme() -> ViewTheme {
        ViewTheme {
            background: Color::from_hex("#0D1017").unwrap(),
            surface: Color::from_hex("#1A2028").unwrap(),
            border: Color::from_hex("#34404D").unwrap(),
            selection: Color::from_hex("#5AA9E6").unwrap(),
            text: Color::from_hex("#E6EDF3").unwrap(),
            text_muted: Color::from_hex("#92A1B2").unwrap(),
        }
    }

    fn sample_panel() -> PanelView {
        PanelView {
            id: PanelId::Preview,
            title: "Preview".to_string(),
            active: false,
            hint_navigation_active: true,
            shortcut: Some(3),
            tabs: Vec::new(),
            header_lines: Vec::new(),
            body: PanelBody::Document(DocumentView {
                lines: Vec::new(),
                scroll: 0,
            }),
        }
    }

    fn cell_symbols(cells: &[Cell]) -> String {
        cells.iter().map(Cell::symbol).collect::<Vec<_>>().join("")
    }

    fn row_cells(terminal: &Terminal<TestBackend>, y: u16, width: u16) -> Vec<Cell> {
        let buffer = terminal.backend().buffer();
        (0..width)
            .map(|x| buffer.cell((x, y)).unwrap().clone())
            .collect::<Vec<_>>()
    }

    fn find_text_start(cells: &[Cell], text: &str) -> usize {
        let row = cell_symbols(cells);
        row.find(text)
            .unwrap_or_else(|| panic!("could not find `{text}` in row `{row}`"))
    }

    #[test]
    fn vertical_scroll_is_clamped_to_last_visible_line() {
        let paragraph = Paragraph::new("1234 5678 90").wrap(Wrap { trim: false });
        assert_eq!(max_document_scroll(&paragraph, Rect::new(0, 0, 4, 2)), 1);
    }

    #[test]
    fn vertical_scroll_is_zero_when_viewport_is_taller_than_content() {
        let paragraph = Paragraph::new("short").wrap(Wrap { trim: false });
        assert_eq!(max_document_scroll(&paragraph, Rect::new(0, 0, 10, 8)), 0);
    }

    #[test]
    fn vertical_scroll_is_zero_for_zero_height_viewports() {
        let paragraph = Paragraph::new("wrapped text").wrap(Wrap { trim: false });
        assert_eq!(max_document_scroll(&paragraph, Rect::new(0, 0, 10, 0)), 0);
    }

    #[test]
    fn hint_active_panel_title_uses_primary_text_without_dimming() {
        let theme = sample_theme();
        let title = panel_title_line(&sample_panel(), &theme);

        let title_span = &title.spans[2];
        assert_eq!(title_span.style.fg, Some(tui(theme.text)));
        assert!(!title_span.style.add_modifier.contains(Modifier::DIM));
        assert!(title_span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn hint_active_panel_border_uses_selection_color() {
        let theme = sample_theme();
        let panel = sample_panel();
        let backend = TestBackend::new(24, 8);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TuiRenderer.render_panel(frame, Rect::new(0, 0, 24, 8), &panel, &theme);
            })
            .unwrap();

        let border_cell = terminal.backend().buffer().cell((0, 0)).unwrap();
        assert_eq!(border_cell.fg, tui(theme.selection));
    }

    #[test]
    fn hint_active_workspace_tabs_promote_target_labels() {
        let theme = sample_theme();
        let view = TabBarView {
            tabs: vec![
                TabItemView {
                    shortcut: Some('a'),
                    label: "Theme".to_string(),
                    selected: false,
                },
                TabItemView {
                    shortcut: Some('b'),
                    label: "Project".to_string(),
                    selected: false,
                },
            ],
        };
        let backend = TestBackend::new(40, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TuiRenderer.render_tab_bar(frame, Rect::new(0, 0, 40, 1), &view, &theme);
            })
            .unwrap();

        let row = row_cells(&terminal, 0, 40);
        let start = find_text_start(&row, "Theme");
        let theme_cells = row[start..start + "Theme".len()].to_vec();

        assert_eq!(cell_symbols(&theme_cells), "Theme");
        for cell in theme_cells {
            assert_eq!(cell.fg, tui(theme.text));
            assert!(cell.modifier.contains(Modifier::BOLD));
        }
    }

    #[test]
    fn hint_active_preview_tabs_promote_target_labels() {
        let theme = sample_theme();
        let tabs = vec![
            PanelTabView {
                shortcut: Some('c'),
                label: "Code".to_string(),
                active: false,
            },
            PanelTabView {
                shortcut: Some('d'),
                label: "Shell".to_string(),
                active: false,
            },
        ];
        let backend = TestBackend::new(32, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TuiRenderer.render_panel_tabs(frame, Rect::new(0, 0, 32, 1), &tabs, &theme);
            })
            .unwrap();

        let row = row_cells(&terminal, 0, 32);
        let start = find_text_start(&row, "Code");
        let code_cells = row[start..start + "Code".len()].to_vec();

        assert_eq!(cell_symbols(&code_cells), "Code");
        for cell in code_cells {
            assert_eq!(cell.fg, tui(theme.text));
            assert!(cell.modifier.contains(Modifier::BOLD));
        }
    }
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

fn styled_line_to_tui(
    line: &StyledLine,
    theme: &ViewTheme,
    hint_navigation_active: bool,
) -> Line<'static> {
    Line::from(
        line.spans
            .iter()
            .cloned()
            .map(|span| {
                let mut tui_span = to_tui_span(span, hint_navigation_active);
                if tui_span.style.fg.is_none() {
                    tui_span.style = tui_span.style.fg(tui(theme.text_muted));
                }
                tui_span
            })
            .collect::<Vec<_>>(),
    )
}

fn to_tui_span(span: StyledSpan, hint_navigation_active: bool) -> Span<'static> {
    let mut style = to_tui_style(span.style);
    if hint_navigation_active {
        style = style.add_modifier(Modifier::DIM);
    }
    Span::styled(span.text, style)
}

fn to_tui_style(style: SpanStyle) -> Style {
    let mut result = Style::default();

    if let Some(fg) = style.fg {
        result = result.fg(tui(fg));
    }
    if let Some(bg) = style.bg {
        result = result.bg(tui(bg));
    }
    if style.bold {
        result = result.add_modifier(Modifier::BOLD);
    }
    if style.italic {
        result = result.add_modifier(Modifier::ITALIC);
    }

    result
}

fn centered_rect(height_percent: u16, width_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

fn centered_rect_absolute(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width.saturating_sub(2)).max(1);
    let height = height.min(area.height.saturating_sub(2)).max(1);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width, height)
}

fn tui(color: Color) -> ratatui::style::Color {
    color.into()
}

fn hint_content_style(hint_navigation_active: bool) -> Style {
    if hint_navigation_active {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default()
    }
}

fn hint_shortcut_style(theme: &ViewTheme) -> Style {
    Style::default()
        .bg(tui(theme.selection))
        .fg(tui(theme.background))
        .add_modifier(Modifier::BOLD)
}

fn panel_border_color(view: &PanelView, theme: &ViewTheme) -> Color {
    if view.active || is_hint_target_panel(view) {
        theme.selection
    } else {
        theme.border
    }
}

fn panel_border_style(view: &PanelView, border: Color) -> Style {
    let style = Style::default().fg(tui(border));
    if is_hint_target_panel(view) || view.active {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

fn is_hint_target_panel(view: &PanelView) -> bool {
    view.hint_navigation_active && view.shortcut.is_some()
}

fn hint_tab_label_style(selected: bool, theme: &ViewTheme, hinted: bool) -> Style {
    if selected {
        Style::default()
            .bg(tui(theme.selection))
            .fg(tui(theme.background))
            .add_modifier(Modifier::BOLD)
    } else if hinted {
        Style::default()
            .bg(tui(theme.background))
            .fg(tui(theme.text))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(tui(theme.background))
            .fg(tui(theme.text_muted))
    }
}

fn hint_panel_tab_label_style(active: bool, theme: &ViewTheme, hinted: bool) -> Style {
    if active {
        Style::default()
            .bg(tui(theme.selection))
            .fg(tui(theme.background))
            .add_modifier(Modifier::BOLD)
    } else if hinted {
        Style::default()
            .bg(tui(theme.surface))
            .fg(tui(theme.text))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(tui(theme.surface))
            .fg(tui(theme.text_muted))
    }
}

fn panel_title_line(view: &PanelView, theme: &ViewTheme) -> Line<'static> {
    let mut spans = Vec::new();
    if let Some(shortcut) = view.shortcut {
        let shortcut_style = if view.hint_navigation_active {
            hint_shortcut_style(theme)
        } else if view.active {
            Style::default()
                .fg(tui(theme.selection))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(tui(theme.text_muted))
        };
        spans.push(Span::styled(format!("[{shortcut}]"), shortcut_style));
        spans.push(Span::raw(" "));
    }
    let title_style = if is_hint_target_panel(view) {
        Style::default()
            .fg(tui(theme.text))
            .add_modifier(Modifier::BOLD)
    } else if view.hint_navigation_active {
        Style::default()
            .fg(tui(theme.text_muted))
            .add_modifier(Modifier::DIM)
    } else if view.active {
        Style::default()
            .fg(tui(theme.text))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(tui(theme.text))
    };
    spans.push(Span::styled(view.title.clone(), title_style));
    Line::from(spans)
}
