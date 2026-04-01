use super::*;

impl TuiRenderer {
    pub(super) fn render_overlay(
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
            .border_style(Style::default().fg(style::tui(theme.selection)))
            .style(Style::default().bg(style::tui(theme.surface)).fg(style::tui(theme.text)));
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
                Span::styled(
                    "Filter: ",
                    Style::default().fg(style::tui(theme.text_muted)),
                ),
                Span::styled(filter_value, Style::default().fg(style::tui(theme.text))),
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
                            .fg(style::tui(theme.text_muted))
                            .add_modifier(Modifier::BOLD),
                    )))
                } else {
                    ListItem::new(Line::from(Span::styled(
                        format!("  {}", row.label),
                        Style::default().fg(style::tui(theme.text)),
                    )))
                }
            })
            .collect::<Vec<_>>();
        let list = List::new(items).highlight_style(
            Style::default()
                .bg(style::tui(theme.selection))
                .fg(style::tui(theme.background))
                .add_modifier(Modifier::BOLD),
        );
        let mut list_state = ListState::default();
        list_state.select(overlay.selected_row);
        frame.render_stateful_widget(list, sections[1], &mut list_state);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{} matches", overlay.total_matches),
                    Style::default().fg(style::tui(theme.text_muted)),
                ),
                Span::raw("  "),
                Span::styled(
                    "Enter apply, Esc close",
                    Style::default().fg(style::tui(theme.text)),
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
            .border_style(Style::default().fg(style::tui(theme.selection)))
            .style(Style::default().bg(style::tui(theme.surface)).fg(style::tui(theme.text)));
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
                    .map(|line| style::styled_line_to_tui(line, theme, false))
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
            .map(|line| style::styled_line_to_tui(line, theme, false))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(footer).wrap(Wrap { trim: false }),
            sections[1],
        );
    }
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
