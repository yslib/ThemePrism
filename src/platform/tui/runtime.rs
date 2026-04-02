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

use crate::app::interaction::{SurfaceId, has_active_capture};
use crate::app::view::PanelBody;
use crate::core::CoreSession;
use crate::platform::tui::event_adapter::TuiEventAdapter;
use crate::platform::tui::preview::PreviewRuntimeController;
use crate::platform::tui::renderer::{TuiRenderer, max_document_scroll};
use crate::platform::tui::view_metrics::locate_panel_body;
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
    let view = session.view_tree();
    let Some((panel, body_area)) = locate_panel_body(
        &view,
        area,
        crate::app::workspace::PanelId::InteractionInspector,
    ) else {
        return;
    };
    let PanelBody::Document(document) = &panel.body else {
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

#[cfg(test)]
mod tests {
    use super::sync_tui_view_state;
    use crate::app::{AppState, Intent};
    use crate::core::CoreSession;
    use crate::platform::tui::view_metrics::main_window_workspace_area;
    use ratatui::layout::Rect;

    #[test]
    fn sync_tui_view_state_clamps_inspector_scroll_to_current_viewport() {
        let mut session = CoreSession::new(AppState::new().expect("state"));
        session.dispatch(Intent::FocusPanelByNumber(6));
        for _ in 0..256 {
            session.dispatch(Intent::MoveSelection(1));
        }

        let before = session.state().ui.interaction_inspector_scroll;
        sync_tui_view_state(&mut session, Rect::new(0, 0, 72, 18));

        assert!(before > session.state().ui.interaction_inspector_scroll);
    }

    #[test]
    fn sync_tui_view_state_preserves_inspector_scroll_when_panel_is_hidden() {
        let mut session = CoreSession::new(AppState::new().expect("state"));
        session.dispatch(Intent::FocusPanelByNumber(6));
        for _ in 0..24 {
            session.dispatch(Intent::MoveSelection(1));
        }
        sync_tui_view_state(&mut session, Rect::new(0, 0, 90, 28));
        let theme_scroll = session.state().ui.interaction_inspector_scroll;

        session.dispatch(Intent::CycleWorkspaceTab(1));
        sync_tui_view_state(&mut session, Rect::new(0, 0, 90, 28));
        assert_eq!(
            session.state().ui.interaction_inspector_scroll,
            theme_scroll
        );

        session.dispatch(Intent::CycleWorkspaceTab(1));
        sync_tui_view_state(&mut session, Rect::new(0, 0, 90, 28));
        assert_eq!(
            session.state().ui.interaction_inspector_scroll,
            theme_scroll
        );
    }

    #[test]
    fn workspace_area_reserves_menu_tab_and_status_rows() {
        let area = main_window_workspace_area(Rect::new(0, 0, 120, 40));
        assert_eq!(area.y, 2);
        assert_eq!(area.height, 37);
    }
}
