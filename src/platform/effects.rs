use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::Path;

use crate::app::effect::ProjectData;
use crate::app::{AppState, Effect, Intent, update};
use crate::export::Exporter;
use crate::export::alacritty::AlacrittyExporter;
use crate::persistence::project_file::{load_project, save_project};

#[derive(Debug, Default, Clone, Copy)]
pub struct AppEffectRunner;

impl AppEffectRunner {
    pub fn run(&self, effect: Effect) -> Intent {
        match effect {
            Effect::SaveProject { path, project } => {
                let result = save_project(&path, &project.params, &project.rules)
                    .map(|()| path.clone())
                    .map_err(|err| err.to_string());
                Intent::ProjectSaved(result)
            }
            Effect::LoadProject { path } => {
                let result = load_project(&path)
                    .map(|(params, rules)| ProjectData { params, rules })
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

pub fn dispatch_intents(state: &mut AppState, intents: Vec<Intent>, runner: &AppEffectRunner) {
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
