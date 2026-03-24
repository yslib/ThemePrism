use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;

use crate::app::view::{PanelBody, build_interaction_panel, panel_area, workspace_layout_for_tab};
use crate::app::workspace::PanelId;
use crate::app::interaction::{SurfaceId, has_active_capture};
use crate::core::CoreSession;
use crate::platform::tui::event_adapter::TuiEventAdapter;
use crate::platform::tui::preview::PreviewRuntimeController;
use crate::platform::tui::renderer::{TuiRenderer, max_document_scroll, panel_body_area};
use crate::platform::{PlatformError, PlatformKind, PlatformRuntime};

#[derive(Debug, Default, Clone, Copy)]
pub struct TuiPlatform;

impl PlatformRuntime for TuiPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Tui
    }

    fn launch(&self, session: CoreSession) -> Result<(), PlatformError> {
        enable_raw_mode().map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;

        let result = run_terminal(&mut terminal, session)
            .map_err(|err| PlatformError::runtime(self.kind(), err.to_string()));

        disable_raw_mode().map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;
        terminal
            .show_cursor()
            .map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;

        result
    }
}

fn run_terminal<B: Backend>(
    terminal: &mut Terminal<B>,
    mut session: CoreSession,
) -> io::Result<()> {
    let adapter = TuiEventAdapter;
    let renderer = TuiRenderer;
    let mut preview = PreviewRuntimeController::default();

    loop {
        let area = terminal.size()?;
        let area = Rect::new(0, 0, area.width, area.height);
        sync_tui_view_state(&mut session, area);
        let view = session.view_tree();
        preview
            .sync(&mut session, &view, area)
            .map_err(io::Error::other)?;
        sync_tui_view_state(&mut session, area);
        let view = session.view_tree();
        terminal.draw(|frame| renderer.present(frame, &view))?;

        if session.should_quit() {
            return Ok(());
        }

        if event::poll(Duration::from_millis(120))? {
            let event = event::read()?;
            if let event::Event::Key(key) = event {
                if key.kind == event::KeyEventKind::Press
                    && has_active_capture(session.state(), SurfaceId::PreviewBody)
                    && preview
                        .handle_capture_key(&mut session, key)
                        .map_err(io::Error::other)?
                {
                    continue;
                }
                let intents = adapter.map_event(&session.state(), event::Event::Key(key));
                session.dispatch_all(intents);
                continue;
            }
            let intents = adapter.map_event(session.state(), event);
            session.dispatch_all(intents);
        }
    }
}

fn sync_tui_view_state(session: &mut CoreSession, area: Rect) {
    let workspace_area = main_window_workspace_area(area);
    let workspace_layout = workspace_layout_for_tab(session.state().ui.active_tab);
    let Some(panel_area) = panel_area(&workspace_layout, workspace_area, PanelId::InteractionInspector)
    else {
        session.clamp_interaction_inspector_scroll(0);
        return;
    };

    let panel = build_interaction_panel(session.state());
    let body_area = panel_body_area(&panel, panel_area);
    let PanelBody::Document(document) = &panel.body else {
        session.clamp_interaction_inspector_scroll(0);
        return;
    };
    let paragraph = ratatui::widgets::Paragraph::new(
        document
            .lines
            .iter()
            .map(|line| {
                ratatui::text::Line::from(
                    line.spans
                        .iter()
                        .map(|span| span.text.as_str())
                        .collect::<Vec<_>>()
                        .join(""),
                )
            })
            .collect::<Vec<_>>(),
    )
    .wrap(ratatui::widgets::Wrap { trim: false });
    session.clamp_interaction_inspector_scroll(max_document_scroll(&paragraph, body_area));
}

fn main_window_workspace_area(area: Rect) -> Rect {
    let sections = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Min(8),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(area);
    sections[2]
}

#[cfg(test)]
mod tests {
    use super::{main_window_workspace_area, sync_tui_view_state};
    use crate::app::{AppState, Intent};
    use crate::core::CoreSession;
    use ratatui::layout::Rect;

    #[test]
    fn sync_tui_view_state_clamps_inspector_scroll_to_current_viewport() {
        let mut session = CoreSession::new(AppState::new().expect("state"));
        session.dispatch(Intent::FocusPanelByNumber(8));
        for _ in 0..256 {
            session.dispatch(Intent::MoveSelection(1));
        }

        let before = session.state().ui.interaction_inspector_scroll;
        sync_tui_view_state(&mut session, Rect::new(0, 0, 90, 28));

        assert!(before > session.state().ui.interaction_inspector_scroll);
    }

    #[test]
    fn workspace_area_reserves_menu_tab_and_status_rows() {
        let area = main_window_workspace_area(Rect::new(0, 0, 120, 40));
        assert_eq!(area.y, 2);
        assert_eq!(area.height, 37);
    }
}
