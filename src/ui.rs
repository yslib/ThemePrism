use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{App, FocusPane};
use crate::preview::sample_code;
use crate::rules::Rule;
use crate::tokens::{PaletteSlot, TokenRole};

pub fn render(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(12), Constraint::Length(2)])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(34),
            Constraint::Min(48),
            Constraint::Length(38),
        ])
        .split(root[0]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(main[0]);

    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(28),
            Constraint::Percentage(27),
        ])
        .split(main[1]);

    render_tokens(frame, left[0], app);
    render_params(frame, left[1], app);
    render_sample_code(frame, center[0], app);
    render_palette(frame, center[1], app);
    render_token_swatches(frame, center[2], app);
    render_inspector(frame, main[2], app);
    render_status(frame, root[1], app);
}

fn render_tokens(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();
    let mut current_category = None;

    for role in TokenRole::ALL {
        if current_category != Some(role.category()) {
            current_category = Some(role.category());
            lines.push(Line::from(Span::styled(
                format!("{}:", role.category().label()),
                Style::default().add_modifier(Modifier::BOLD),
            )));
        }

        let selected = role == app.selected_role();
        let color = app.theme_color(role);
        let mut style = Style::default().fg(tui(color));
        if selected {
            style = style
                .add_modifier(Modifier::BOLD)
                .bg(tui(app.theme_color(TokenRole::Selection)))
                .fg(tui(app.theme_color(TokenRole::Background)));
        }

        let prefix = if selected { ">" } else { " " };
        lines.push(Line::from(vec![
            Span::styled(format!("{prefix} "), style),
            Span::styled("■ ", Style::default().fg(tui(color))),
            Span::styled(role.label(), style),
        ]));
    }

    let block = pane_block("Token List", app.focus == FocusPane::Tokens, app);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_params(frame: &mut Frame, area: Rect, app: &App) {
    let lines = crate::params::ParamKey::ALL
        .into_iter()
        .enumerate()
        .map(|(index, key)| {
            let selected = index == app.selected_param;
            let style = if selected {
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(tui(app.theme_color(TokenRole::Selection)))
                    .fg(tui(app.theme_color(TokenRole::Background)))
            } else {
                Style::default().fg(tui(app.theme_color(TokenRole::Text)))
            };
            Line::from(vec![
                Span::styled(if selected { "> " } else { "  " }, style),
                Span::styled(format!("{:<18}", key.label()), style),
                Span::styled(key.format_value(&app.params), style),
            ])
        })
        .collect::<Vec<_>>();

    let block = pane_block("Theme Params", app.focus == FocusPane::Params, app);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_sample_code(frame: &mut Frame, area: Rect, app: &App) {
    let background = app.theme_color(TokenRole::Background);
    let lines = sample_code()
        .into_iter()
        .map(|segments| {
            let spans = segments
                .into_iter()
                .map(|segment| {
                    let role = segment.role.unwrap_or(TokenRole::Text);
                    Span::styled(
                        segment.text,
                        Style::default()
                            .fg(tui(app.theme_color(role)))
                            .bg(tui(background)),
                    )
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect::<Vec<_>>();

    let block = Block::default()
        .title("Preview / Sample Code")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(tui(app.theme_color(TokenRole::Border))));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_palette(frame: &mut Frame, area: Rect, app: &App) {
    let rows = PaletteSlot::ALL
        .into_iter()
        .map(|slot| {
            let color = app.palette_color(slot);
            Line::from(vec![
                Span::styled("    ", Style::default().bg(tui(color))),
                Span::raw(" "),
                Span::styled(
                    format!("{:<10}", slot.label()),
                    Style::default().fg(tui(app.theme_color(TokenRole::Text))),
                ),
                Span::styled(
                    color.to_hex(),
                    Style::default().fg(tui(app.theme_color(TokenRole::TextMuted))),
                ),
            ])
        })
        .collect::<Vec<_>>();

    let block = Block::default()
        .title("Palette")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(tui(app.theme_color(TokenRole::Border))));
    frame.render_widget(
        Paragraph::new(rows).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_token_swatches(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_roles = &TokenRole::ALL[..10];
    let right_roles = &TokenRole::ALL[10..];

    render_swatch_column(frame, columns[0], left_roles, app, "Resolved Tokens");
    render_swatch_column(frame, columns[1], right_roles, app, " ");
}

fn render_swatch_column(
    frame: &mut Frame,
    area: Rect,
    roles: &[TokenRole],
    app: &App,
    title: &str,
) {
    let lines = roles
        .iter()
        .map(|role| {
            let color = app.theme_color(*role);
            Line::from(vec![
                Span::styled("   ", Style::default().bg(tui(color))),
                Span::raw(" "),
                Span::styled(
                    format!("{:<12}", role.label()),
                    Style::default().fg(tui(app.theme_color(TokenRole::Text))),
                ),
                Span::styled(
                    color.to_hex(),
                    Style::default().fg(tui(app.theme_color(TokenRole::TextMuted))),
                ),
            ])
        })
        .collect::<Vec<_>>();

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(tui(app.theme_color(TokenRole::Border))));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_inspector(frame: &mut Frame, area: Rect, app: &App) {
    let color = app.current_token_color();
    let rule = app.selected_rule();
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Token: ", label_style(app)),
            Span::styled(app.selected_role().label(), value_style(app)),
        ]),
        Line::from(vec![
            Span::styled("Color: ", label_style(app)),
            Span::styled("    ", Style::default().bg(tui(color))),
            Span::raw(" "),
            Span::styled(color.to_hex(), value_style(app)),
        ]),
        Line::from(vec![
            Span::styled("Summary: ", label_style(app)),
            Span::styled(rule.summary(), value_style(app)),
        ]),
        Line::raw(""),
    ];

    let fields = inspector_lines(app);
    lines.extend(fields);

    let footer = match rule {
        Rule::Fixed { .. } => "Fixed color cycles through palette + token colors.",
        Rule::Mix { .. } => "Adjust ratio with left/right in the inspector.",
        Rule::Adjust { .. } => "Adjust amount with left/right in the inspector.",
        Rule::Alias { .. } => "Swap source with left/right in the inspector.",
    };

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        footer,
        Style::default()
            .fg(tui(app.theme_color(TokenRole::TextMuted)))
            .italic(),
    )));

    let block = pane_block("Inspector", app.focus == FocusPane::Inspector, app);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn inspector_lines(app: &App) -> Vec<Line<'static>> {
    let selected_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(tui(app.theme_color(TokenRole::Selection)))
        .fg(tui(app.theme_color(TokenRole::Background)));
    let base_style = Style::default().fg(tui(app.theme_color(TokenRole::Text)));

    let mut lines = Vec::new();
    let mut push_field = |index: usize, label: String, value: String| {
        let style = if app.focus == FocusPane::Inspector && app.inspector_field == index {
            selected_style
        } else {
            base_style
        };
        lines.push(Line::from(vec![
            Span::styled(
                if app.focus == FocusPane::Inspector && app.inspector_field == index {
                    "> "
                } else {
                    "  "
                },
                style,
            ),
            Span::styled(format!("{label:<11}"), style),
            Span::styled(value, style),
        ]));
    };

    push_field(
        0,
        "Rule Type".to_string(),
        app.selected_rule().kind().label().to_string(),
    );

    match app.selected_rule() {
        Rule::Alias { source } => {
            push_field(1, "Source".to_string(), source.label());
        }
        Rule::Mix { a, b, ratio } => {
            push_field(1, "Color A".to_string(), a.label());
            push_field(2, "Color B".to_string(), b.label());
            push_field(3, "Blend".to_string(), format!("{:>3.0}%", ratio * 100.0));
        }
        Rule::Adjust { source, op, amount } => {
            push_field(1, "Source".to_string(), source.label());
            push_field(2, "Operation".to_string(), op.label().to_string());
            push_field(3, "Amount".to_string(), format!("{:>3.0}%", amount * 100.0));
        }
        Rule::Fixed { color } => {
            push_field(1, "Hex".to_string(), color.to_hex());
        }
    }

    lines
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let line = Line::from(vec![
        Span::styled(
            format!("Focus: {}  ", app.focus.label()),
            Style::default()
                .fg(tui(app.theme_color(TokenRole::Background)))
                .bg(tui(app.theme_color(TokenRole::Selection)))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Tab focus  |  ↑↓ select  |  ←→ adjust  |  e export  |  r reset  |  q quit  ",
            Style::default().fg(tui(app.theme_color(TokenRole::Text))),
        ),
        Span::styled(
            &app.status,
            Style::default().fg(tui(app.theme_color(TokenRole::TextMuted))),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(line).style(
            Style::default()
                .bg(tui(app.theme_color(TokenRole::Surface)))
                .fg(tui(app.theme_color(TokenRole::Text))),
        ),
        area,
    );
}

fn pane_block<'a>(title: &'a str, active: bool, app: &App) -> Block<'a> {
    let border_color = if active {
        app.theme_color(TokenRole::Selection)
    } else {
        app.theme_color(TokenRole::Border)
    };

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(tui(border_color)))
}

fn label_style(app: &App) -> Style {
    Style::default().fg(tui(app.theme_color(TokenRole::TextMuted)))
}

fn value_style(app: &App) -> Style {
    Style::default().fg(tui(app.theme_color(TokenRole::Text)))
}

fn tui(color: crate::color::Color) -> ratatui::style::Color {
    color.into()
}
