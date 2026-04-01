use super::{TuiRenderer, max_document_scroll};
use super::{panels::panel_title_line, style::tui};
use crate::app::view::{
    DocumentView, MainWindowView, MenuBarView, PanelBody, PanelTabView, PanelView, StatusBarView,
    TabBarView, TabItemView, ViewNode, ViewTheme, ViewTree,
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

fn sample_view_tree(tab_bar: TabBarView, hint_navigation_active: bool) -> ViewTree {
    ViewTree {
        theme: sample_theme(),
        main_window: MainWindowView {
            hint_navigation_active,
            menu_bar: MenuBarView {
                title: "Theme".to_string(),
                actions: Vec::new(),
            },
            tab_bar,
            fullscreen_panel: None,
            workspace: ViewNode::Panel(sample_panel()),
            status_bar: StatusBarView {
                focus_label: "Preview".to_string(),
                status_text: "Ready".to_string(),
            },
        },
        overlays: Vec::new(),
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
fn non_hint_tabs_dim_when_hint_navigation_is_active() {
    let view = sample_view_tree(
        TabBarView {
            tabs: vec![
                TabItemView {
                    shortcut: None,
                    label: "Theme".to_string(),
                    selected: false,
                },
                TabItemView {
                    shortcut: None,
                    label: "Project".to_string(),
                    selected: false,
                },
            ],
        },
        true,
    );
    let backend = TestBackend::new(40, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| TuiRenderer.present(frame, &view))
        .unwrap();

    let row = row_cells(&terminal, 1, 40);
    let start = find_text_start(&row, "Theme");
    let theme_cells = row[start..start + "Theme".len()].to_vec();

    assert_eq!(cell_symbols(&theme_cells), "Theme");
    for cell in theme_cells {
        assert!(cell.modifier.contains(Modifier::DIM));
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
