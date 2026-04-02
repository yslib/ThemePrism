use std::path::PathBuf;

use crate::domain::evaluator::ResolvedTheme;
use crate::domain::params::ThemeParams;
use crate::domain::rules::RuleSet;
use crate::export::ExportProfile;
use crate::persistence::editor_config::EditorConfig;

#[derive(Debug, Clone)]
pub struct ProjectData {
    pub name: String,
    pub params: ThemeParams,
    pub rules: RuleSet,
    pub export_profiles: Vec<ExportProfile>,
}

#[derive(Debug, Clone)]
pub struct EditorConfigData {
    pub config: EditorConfig,
}

#[derive(Debug, Clone)]
pub enum Effect {
    SaveProject {
        path: PathBuf,
        project: ProjectData,
    },
    LoadProject {
        path: PathBuf,
    },
    SaveEditorConfig {
        data: EditorConfigData,
    },
    ExportTheme {
        project_name: String,
        params: ThemeParams,
        profiles: Vec<ExportProfile>,
        theme: ResolvedTheme,
    },
}
