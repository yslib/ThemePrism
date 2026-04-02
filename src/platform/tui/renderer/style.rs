use super::*;

pub(super) fn styled_line_to_tui(
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

pub(super) fn to_tui_span(span: StyledSpan, hint_navigation_active: bool) -> Span<'static> {
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

pub(super) fn tui(color: Color) -> ratatui::style::Color {
    color.into()
}

pub(super) fn hint_content_style(hint_navigation_active: bool) -> Style {
    if hint_navigation_active {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default()
    }
}

pub(super) fn hint_shortcut_style(theme: &ViewTheme) -> Style {
    Style::default()
        .bg(tui(theme.selection))
        .fg(tui(theme.background))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn hint_tab_label_style(selected: bool, theme: &ViewTheme, hinted: bool) -> Style {
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

pub(super) fn hint_panel_tab_label_style(active: bool, theme: &ViewTheme, hinted: bool) -> Style {
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
