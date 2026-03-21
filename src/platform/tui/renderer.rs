use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::controls::ControlSpec;
use crate::app::view::{
    Axis, CodePreviewView, ConfigOverlayView, FormFieldView, FormView, NumericEditorOverlayView,
    OverlayView, PanelBody, PanelView, PickerOverlayView, SelectionListView, SelectionRowView,
    Size, SpanStyle, StatusBarView, StyledLine, StyledSpan, SwatchItemView, SwatchListView,
    ViewNode, ViewTheme, ViewTree,
};
use crate::domain::color::Color;

#[derive(Debug, Default, Clone, Copy)]
pub struct TuiRenderer;

impl TuiRenderer {
    pub fn present(self, frame: &mut Frame, tree: &ViewTree) {
        self.render_node(frame, frame.area(), &tree.root, &tree.theme);

        for overlay in &tree.overlays {
            self.render_overlay(frame, frame.area(), overlay, &tree.theme);
        }
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
            ViewNode::StatusBar(view) => self.render_status_bar(frame, area, view, theme),
        }
    }

    fn render_panel(self, frame: &mut Frame, area: Rect, view: &PanelView, theme: &ViewTheme) {
        let border = if view.active {
            theme.selection
        } else {
            theme.border
        };
        let block = Block::default()
            .title(view.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tui(border)));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        match &view.body {
            PanelBody::SelectionList(list) => self.render_selection_list(frame, inner, list, theme),
            PanelBody::Form(form) => self.render_form(frame, inner, form, theme),
            PanelBody::CodePreview(code) => self.render_code_preview(frame, inner, code),
            PanelBody::SwatchList(list) => self.render_swatch_list(frame, inner, list, theme),
        }
    }

    fn render_selection_list(
        self,
        frame: &mut Frame,
        area: Rect,
        list: &SelectionListView,
        theme: &ViewTheme,
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

        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
    }

    fn render_form(self, frame: &mut Frame, area: Rect, form: &FormView, theme: &ViewTheme) {
        let mut lines = form
            .header_lines
            .iter()
            .map(|line| styled_line_to_tui(line, theme))
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

        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
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
        let label = format!("{:<11}", field.control.label());
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

    fn render_code_preview(self, frame: &mut Frame, area: Rect, code: &CodePreviewView) {
        let lines = code
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .cloned()
                    .map(to_tui_span)
                    .collect::<Vec<_>>()
            })
            .map(Line::from)
            .collect::<Vec<_>>();

        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
    }

    fn render_swatch_list(
        self,
        frame: &mut Frame,
        area: Rect,
        list: &SwatchListView,
        theme: &ViewTheme,
    ) {
        let lines = list
            .items
            .iter()
            .map(|item| self.render_swatch_item(item, theme))
            .collect::<Vec<_>>();

        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
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
                format!("{}  ", view.help_text),
                Style::default().fg(tui(theme.text)),
            ),
            Span::styled(
                view.status_text.as_str(),
                Style::default().fg(tui(theme.text_muted)),
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
            OverlayView::Config(view) => self.render_config(frame, area, view, theme),
            OverlayView::NumericEditor(view) => {
                self.render_numeric_editor(frame, area, view, theme)
            }
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

    fn render_config(
        self,
        frame: &mut Frame,
        area: Rect,
        overlay: &ConfigOverlayView,
        theme: &ViewTheme,
    ) {
        let area = centered_rect(68, 74, area);
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(overlay.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tui(theme.selection)))
            .style(Style::default().bg(tui(theme.surface)).fg(tui(theme.text)));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let footer_height = overlay.footer_lines.len() as u16 + 1;
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(footer_height)])
            .split(inner);

        let rows = overlay
            .rows
            .iter()
            .map(|row| {
                if row.is_header {
                    Line::from(Span::styled(
                        row.label.as_str(),
                        Style::default()
                            .fg(tui(theme.text_muted))
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    let style = if row.selected {
                        Style::default()
                            .bg(tui(theme.selection))
                            .fg(tui(theme.background))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(tui(theme.text))
                    };
                    Line::from(vec![
                        Span::styled(if row.selected { "> " } else { "  " }, style),
                        Span::styled(format!("{:<12}", row.label), style),
                        Span::styled(row.value_text.clone(), style),
                    ])
                }
            })
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(rows).wrap(Wrap { trim: false }), sections[0]);

        let footer = overlay
            .footer_lines
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    line.as_str(),
                    Style::default().fg(tui(theme.text_muted)),
                ))
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(footer).wrap(Wrap { trim: false }),
            sections[1],
        );
    }

    fn render_numeric_editor(
        self,
        frame: &mut Frame,
        area: Rect,
        overlay: &NumericEditorOverlayView,
        theme: &ViewTheme,
    ) {
        let area = centered_rect(52, 62, area);
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(overlay.title.as_str())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tui(theme.selection)))
            .style(Style::default().bg(tui(theme.surface)).fg(tui(theme.text)));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let footer_height = overlay.footer_lines.len() as u16 + 1;
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(footer_height)])
            .split(inner);

        let body = overlay
            .body_lines
            .iter()
            .map(|line| styled_line_to_tui(line, theme))
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }), sections[0]);

        let footer = overlay
            .footer_lines
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    line.as_str(),
                    Style::default().fg(tui(theme.text_muted)),
                ))
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(footer).wrap(Wrap { trim: false }),
            sections[1],
        );
    }
}

fn to_constraint(size: Size) -> Constraint {
    match size {
        Size::Length(value) => Constraint::Length(value),
        Size::Min(value) => Constraint::Min(value),
        Size::Percentage(value) => Constraint::Percentage(value),
    }
}

fn styled_line_to_tui(line: &StyledLine, theme: &ViewTheme) -> Line<'static> {
    Line::from(
        line.spans
            .iter()
            .cloned()
            .map(|span| {
                let mut tui_span = to_tui_span(span);
                if tui_span.style.fg.is_none() {
                    tui_span.style = tui_span.style.fg(tui(theme.text_muted));
                }
                tui_span
            })
            .collect::<Vec<_>>(),
    )
}

fn to_tui_span(span: StyledSpan) -> Span<'static> {
    Span::styled(span.text, to_tui_style(span.style))
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

fn tui(color: Color) -> ratatui::style::Color {
    color.into()
}
