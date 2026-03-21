use std::path::PathBuf;

use crate::domain::evaluator::ResolvedTheme;
use crate::domain::params::ThemeParams;
use crate::domain::rules::RuleSet;
use crate::export::ExportProfile;

#[derive(Debug, Clone)]
pub struct ProjectData {
    pub name: String,
    pub params: ThemeParams,
    pub rules: RuleSet,
    pub export_profiles: Vec<ExportProfile>,
}

#[derive(Debug, Clone)]
pub enum Effect {
    SaveProject { path: PathBuf, project: ProjectData },
    LoadProject { path: PathBuf },
    ExportTheme {
        profiles: Vec<ExportProfile>,
        theme: ResolvedTheme,
    },
}
