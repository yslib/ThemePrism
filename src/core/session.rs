use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::Path;

use crate::app::snapshot::{AppSnapshot, build_snapshot};
use crate::app::view::{ViewTree, build_view};
use crate::app::{AppState, Effect, Intent, update};
use crate::export::{ExportArtifact, export_with_profile};
use crate::persistence::editor_config::save_editor_config;
use crate::persistence::project_file::{load_project, save_project};

#[derive(Debug)]
pub struct CoreSession {
    state: AppState,
}

impl CoreSession {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn should_quit(&self) -> bool {
        self.state.ui.should_quit
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.state.ui.status = message.into();
    }

    pub fn clamp_interaction_inspector_scroll(&mut self, max_scroll: u16) {
        self.state.ui.interaction_inspector_scroll =
            self.state.ui.interaction_inspector_scroll.min(max_scroll);
    }

    pub fn dispatch(&mut self, intent: Intent) {
        self.dispatch_all([intent]);
    }

    pub fn dispatch_all(&mut self, intents: impl IntoIterator<Item = Intent>) {
        let mut queue = intents.into_iter().collect::<VecDeque<_>>();

        while let Some(intent) = queue.pop_front() {
            let effects = update(&mut self.state, intent);
            for effect in effects {
                queue.push_back(self.run_effect(effect));
            }
        }
    }

    pub fn view_tree(&self) -> ViewTree {
        build_view(&self.state)
    }

    pub fn snapshot(&self) -> AppSnapshot {
        build_snapshot(&self.state)
    }

    fn run_effect(&self, effect: Effect) -> Intent {
        match effect {
            Effect::SaveProject { path, project } => {
                let result = save_project(&path, &project)
                    .map(|()| path.clone())
                    .map_err(|err| err.to_string());
                Intent::ProjectSaved(result)
            }
            Effect::LoadProject { path } => {
                let result = load_project(&path).map_err(|err| err.to_string());
                let _ = path;
                Intent::ProjectLoaded(result)
            }
            Effect::SaveEditorConfig { data } => {
                let result = save_editor_config(&data.config).map_err(|err| err.to_string());
                Intent::EditorConfigSaved(result)
            }
            Effect::ExportTheme {
                project_name,
                params,
                profiles,
                theme,
            } => {
                let result = (|| -> Result<Vec<ExportArtifact>, String> {
                    let enabled = profiles
                        .into_iter()
                        .filter(|profile| profile.enabled)
                        .collect::<Vec<_>>();

                    if enabled.is_empty() {
                        return Err("no export targets are enabled".to_string());
                    }

                    let mut artifacts = Vec::new();
                    for profile in enabled {
                        let content = export_with_profile(&profile, &project_name, &theme, &params)
                            .map_err(|err| err.to_string())?;
                        write_export(&profile.output_path, &content)
                            .map_err(|err| err.to_string())?;
                        artifacts.push(ExportArtifact {
                            profile_name: profile.name.clone(),
                            output_path: profile.output_path.clone(),
                        });
                    }

                    Ok(artifacts)
                })();
                Intent::ThemeExported(result)
            }
        }
    }
}

fn write_export(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use crate::app::{AppState, Intent};
    use crate::export::ExportFormat;

    use super::CoreSession;

    #[test]
    fn export_flow_threads_project_profile_path_and_param_data() {
        let template_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        fs::write(
            template_file.path(),
            "project={{meta.project_name}}\nprofile={{meta.profile_name}}\nformat={{meta.profile_format}}\noutput={{meta.output_path}}\ncontrast={{param.contrast}}\nexporter={{meta.exporter}}\n",
        )
        .unwrap();

        let mut state = AppState::new().unwrap();
        state.project.name = "Session Project".to_string();
        state.domain.params.contrast = 0.42;
        state.recompute().unwrap();
        for profile in &mut state.project.export_profiles {
            profile.enabled = false;
        }
        let profile = state
            .project
            .export_profiles
            .iter_mut()
            .find(|profile| matches!(profile.format, ExportFormat::Template { .. }))
            .unwrap();
        profile.name = "Session Template".to_string();
        profile.output_path = output_file.path().to_path_buf();
        if let ExportFormat::Template { template_path } = &mut profile.format {
            *template_path = template_file.path().to_path_buf();
        }
        profile.enabled = true;

        let mut session = CoreSession::new(state);
        session.dispatch(Intent::ExportThemeRequested);

        let output = fs::read_to_string(output_file.path()).unwrap();
        assert!(output.contains("project=Session Project"));
        assert!(output.contains("profile=Session Template"));
        assert!(output.contains("format=template"));
        assert!(output.contains(&format!("output={}", output_file.path().display())));
        assert!(output.contains("contrast=0.42"));
        assert!(output.contains("exporter=template"));
    }
}
