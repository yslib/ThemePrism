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

use crate::core::CoreSession;
use crate::platform::tui::event_adapter::TuiEventAdapter;
use crate::platform::tui::preview::PreviewRuntimeController;
use crate::platform::tui::renderer::TuiRenderer;
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
        let view = session.view_tree();
        preview
            .sync(&mut session, &view, area)
            .map_err(io::Error::other)?;
        let view = session.view_tree();
        terminal.draw(|frame| renderer.present(frame, &view))?;

        if session.should_quit() {
            return Ok(());
        }

        if event::poll(Duration::from_millis(120))? {
            let event = event::read()?;
            if let event::Event::Key(key) = event {
                if key.kind == event::KeyEventKind::Press
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
