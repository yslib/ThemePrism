use super::*;

impl TuiRenderer {
    pub(super) fn render_selection_list(
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
                        .fg(style::tui(theme.text_muted))
                        .add_modifier(Modifier::BOLD),
                )),
                SelectionRowView::Item {
                    label,
                    color,
                    value_text,
                    selected,
                } => {
                    let style = if *selected {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .bg(style::tui(theme.selection))
                            .fg(style::tui(theme.background))
                    } else {
                        Style::default().fg(style::tui(theme.text))
                    };

                    Line::from(vec![
                        Span::styled(if *selected { "> " } else { "  " }, style),
                        Span::styled("    ", Style::default().bg(style::tui(*color))),
                        Span::raw(" "),
                        Span::styled(format!("{:<14}", label), style),
                        Span::styled(value_text.clone(), style),
                    ])
                }
            })
            .collect::<Vec<_>>();

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .style(style::hint_content_style(hint_navigation_active)),
            area,
        );
    }

    pub(super) fn render_form(
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
            .map(|line| style::styled_line_to_tui(line, theme, hint_navigation_active))
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
                    .fg(style::tui(theme.text_muted))
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .style(style::hint_content_style(hint_navigation_active)),
            area,
        );
    }

    fn render_form_field(self, field: &FormFieldView, theme: &ViewTheme) -> Line<'static> {
        let style = if field.selected {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(style::tui(theme.selection))
                .fg(style::tui(theme.background))
        } else {
            Style::default().fg(style::tui(theme.text))
        };

        let prefix = if field.selected { "> " } else { "  " };
        let label = format!("{:<14}", field.control.label());
        let value = field.control.value_text();

        let mut spans = vec![Span::styled(prefix, style), Span::styled(label, style)];

        if let Some(color) = field.control.swatch() {
            spans.push(Span::styled("    ", Style::default().bg(style::tui(color))));
            spans.push(Span::raw(" "));
        }

        spans.push(Span::styled(value.to_string(), style));

        if matches!(&field.control, ControlSpec::ReferencePicker(spec) if spec.picker_open) {
            spans.push(Span::styled(
                "  [picker]",
                Style::default().fg(style::tui(theme.text_muted)),
            ));
        }

        Line::from(spans)
    }

    pub(super) fn render_document(
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
                    .map(|span| style::to_tui_span(span, hint_navigation_active))
                    .collect::<Vec<_>>()
            })
            .map(Line::from)
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(style::hint_content_style(hint_navigation_active));
        let scroll = document.scroll.min(max_document_scroll(&paragraph, area));

        frame.render_widget(paragraph.scroll((scroll, 0)), area);
    }

    pub(super) fn render_swatch_list(
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
                .style(style::hint_content_style(hint_navigation_active)),
            area,
        );
    }

    fn render_swatch_item(self, item: &SwatchItemView, theme: &ViewTheme) -> Line<'static> {
        Line::from(vec![
            Span::styled("    ", Style::default().bg(style::tui(item.color))),
            Span::raw(" "),
            Span::styled(
                format!("{:<12}", item.label),
                Style::default().fg(style::tui(theme.text)),
            ),
            Span::styled(
                item.value_text.clone(),
                Style::default().fg(style::tui(theme.text_muted)),
            ),
        ])
    }
}

pub(super) fn max_document_scroll(paragraph: &Paragraph<'_>, area: Rect) -> u16 {
    if area.height == 0 || area.width == 0 {
        return 0;
    }

    let rendered_line_count = paragraph.line_count(area.width);
    rendered_line_count.saturating_sub(area.height as usize) as u16
}
