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

use crate::app::AppState;
use crate::app::build_view;
use crate::platform::effects::{AppEffectRunner, dispatch_intents};
use crate::platform::tui::event_adapter::TuiEventAdapter;
use crate::platform::tui::renderer::TuiRenderer;
use crate::platform::{PlatformError, PlatformKind, PlatformRuntime};

#[derive(Debug, Default, Clone, Copy)]
pub struct TuiPlatform;

impl PlatformRuntime for TuiPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Tui
    }

    fn launch(&self, state: AppState) -> Result<(), PlatformError> {
        enable_raw_mode().map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|err| PlatformError::runtime(self.kind(), err.to_string()))?;

        let result = run_terminal(&mut terminal, state)
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

fn run_terminal<B: Backend>(terminal: &mut Terminal<B>, mut state: AppState) -> io::Result<()> {
    let adapter = TuiEventAdapter;
    let renderer = TuiRenderer;
    let effects = AppEffectRunner;

    loop {
        let view = build_view(&state);
        terminal.draw(|frame| renderer.present(frame, &view))?;

        if state.ui.should_quit {
            return Ok(());
        }

        if event::poll(Duration::from_millis(120))? {
            let event = event::read()?;
            let intents = adapter.map_event(&state, event);
            dispatch_intents(&mut state, intents, &effects);
        }
    }
}
