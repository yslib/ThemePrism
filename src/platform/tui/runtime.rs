use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use crossterm::event;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::backend::CrosstermBackend;

use crate::app::{AppState, Effect, Intent, build_view, update};
use crate::export::Exporter;
use crate::export::alacritty::AlacrittyExporter;
use crate::persistence::project_file::{load_project, save_project};
use crate::platform::tui::event_adapter::TuiEventAdapter;
use crate::platform::tui::renderer::TuiRenderer;
use crate::platform::{PlatformError, PlatformKind, PlatformRuntime};

#[derive(Debug, Default)]
struct TuiEffectRunner;

impl TuiEffectRunner {
    fn run(&self, effect: Effect) -> Intent {
        match effect {
            Effect::SaveProject { path, project } => {
                let result = save_project(&path, &project.params, &project.rules)
                    .map(|()| path.clone())
                    .map_err(|err| err.to_string());
                Intent::ProjectSaved(result)
            }
            Effect::LoadProject { path } => {
                let result = load_project(&path)
                    .map(|(params, rules)| crate::app::effect::ProjectData { params, rules })
                    .map_err(|err| err.to_string());
                let _ = path;
                Intent::ProjectLoaded(result)
            }
            Effect::ExportAlacritty { path, theme } => {
                let exporter = AlacrittyExporter;
                let result = exporter
                    .export(&theme)
                    .map_err(|err| err.to_string())
                    .and_then(|content| {
                        write_export(&path, &content)
                            .map(|()| path.clone())
                            .map_err(|err| err.to_string())
                    });
                Intent::ThemeExported(result)
            }
        }
    }
}

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
    let effects = TuiEffectRunner;

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

fn dispatch_intents(state: &mut AppState, intents: Vec<Intent>, runner: &TuiEffectRunner) {
    let mut queue = intents.into_iter().collect::<VecDeque<_>>();

    while let Some(intent) = queue.pop_front() {
        let effects = update(state, intent);
        for effect in effects {
            queue.push_back(runner.run(effect));
        }
    }
}

fn write_export(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}
