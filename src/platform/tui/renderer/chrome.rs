use super::*;

impl TuiRenderer {
    pub(super) fn render_menu_bar(
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
                .bg(style::tui(theme.selection))
                .fg(style::tui(theme.background))
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
            Paragraph::new(Line::from(spans)).style(
                Style::default()
                    .bg(style::tui(theme.surface))
                    .fg(style::tui(theme.text)),
            ),
            area,
        );
    }

    #[cfg(test)]
    pub(super) fn render_tab_bar(
        self,
        frame: &mut Frame,
        area: Rect,
        view: &TabBarView,
        theme: &ViewTheme,
    ) {
        let hint_navigation_active = view.tabs.iter().any(|tab| tab.shortcut.is_some());
        self.render_tab_bar_with_hint_state(frame, area, view, theme, hint_navigation_active);
    }

    pub(super) fn render_tab_bar_with_hint_state(
        self,
        frame: &mut Frame,
        area: Rect,
        view: &TabBarView,
        theme: &ViewTheme,
        hint_navigation_active: bool,
    ) {
        let mut spans = Vec::new();
        for (index, tab) in view.tabs.iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled(
                    "  ",
                    Style::default().bg(style::tui(theme.background)),
                ));
            }

            if let Some(shortcut) = tab.shortcut {
                spans.push(Span::styled(
                    format!(" [{}] ", shortcut),
                    style::hint_shortcut_style(theme),
                ));
                let label_style = style::hint_tab_label_style(tab.selected, theme, true);
                spans.push(Span::styled(tab.label.clone(), label_style));
                spans.push(Span::styled(" ", label_style));
            } else {
                let style = style::hint_tab_label_style(tab.selected, theme, false);
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
                    .bg(style::tui(theme.background))
                    .fg(style::tui(theme.text)),
            ),
            area,
        );
    }

    pub(super) fn render_status_bar(
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
                    .fg(style::tui(theme.background))
                    .bg(style::tui(theme.selection))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                view.status_text.as_str(),
                if hint_navigation_active {
                    Style::default()
                        .fg(style::tui(theme.text_muted))
                        .add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(style::tui(theme.text_muted))
                },
            ),
        ]);

        frame.render_widget(
            Paragraph::new(line).style(
                Style::default()
                    .bg(style::tui(theme.surface))
                    .fg(style::tui(theme.text)),
            ),
            area,
        );
    }
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
                Style::default().fg(style::tui(theme.text_muted)),
            ));
            used_width += 2;
        }
        spans.extend(action_hint_spans(
            action,
            Style::default()
                .fg(style::tui(theme.text_muted))
                .add_modifier(if hint_navigation_active {
                    Modifier::BOLD | Modifier::DIM
                } else {
                    Modifier::BOLD
                }),
            if hint_navigation_active {
                Style::default()
                    .fg(style::tui(theme.text_muted))
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(style::tui(theme.text))
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
