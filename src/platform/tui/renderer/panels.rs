use super::*;

impl TuiRenderer {
    pub(super) fn render_panel(
        self,
        frame: &mut Frame,
        area: Rect,
        view: &PanelView,
        theme: &ViewTheme,
    ) {
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
                .map(|line| style::styled_line_to_tui(line, theme, view.hint_navigation_active))
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(header)
                    .wrap(Wrap { trim: false })
                    .style(style::hint_content_style(view.hint_navigation_active)),
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

    pub(super) fn render_panel_tabs(
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
                    style::hint_shortcut_style(theme),
                ));
                let label_style = style::hint_panel_tab_label_style(tab.active, theme, true);
                spans.push(Span::styled(tab.label.clone(), label_style));
                spans.push(Span::styled(" ", label_style));
            } else {
                let style = style::hint_panel_tab_label_style(tab.active, theme, false);
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
                .style(Style::default().bg(style::tui(theme.surface)).fg(style::tui(theme.text))),
            area,
        );
    }
}

pub(super) fn panel_body_area(panel: &PanelView, area: Rect) -> Rect {
    let block = Block::default()
        .title(panel.title.as_str())
        .borders(Borders::ALL);
    let inner = block.inner(area);
    panel_body_sections(panel, inner)[2]
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

fn panel_border_color(view: &PanelView, theme: &ViewTheme) -> Color {
    if view.active || is_hint_target_panel(view) {
        theme.selection
    } else {
        theme.border
    }
}

fn panel_border_style(view: &PanelView, border: Color) -> Style {
    let style = Style::default().fg(style::tui(border));
    if is_hint_target_panel(view) || view.active {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

fn is_hint_target_panel(view: &PanelView) -> bool {
    view.hint_navigation_active && view.shortcut.is_some()
}

pub(super) fn panel_title_line(view: &PanelView, theme: &ViewTheme) -> Line<'static> {
    let mut spans = Vec::new();
    if let Some(shortcut) = view.shortcut {
        let shortcut_style = if view.hint_navigation_active {
            style::hint_shortcut_style(theme)
        } else if view.active {
            Style::default()
                .fg(style::tui(theme.selection))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(style::tui(theme.text_muted))
        };
        spans.push(Span::styled(format!("[{shortcut}]"), shortcut_style));
        spans.push(Span::raw(" "));
    }
    let title_style = if is_hint_target_panel(view) {
        Style::default()
            .fg(style::tui(theme.text))
            .add_modifier(Modifier::BOLD)
    } else if view.hint_navigation_active {
        Style::default()
            .fg(style::tui(theme.text_muted))
            .add_modifier(Modifier::DIM)
    } else if view.active {
        Style::default()
            .fg(style::tui(theme.text))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(style::tui(theme.text))
    };
    spans.push(Span::styled(view.title.clone(), title_style));
    Line::from(spans)
}
