use std::path::PathBuf;

use crate::domain::evaluator::ResolvedTheme;
use crate::domain::params::ThemeParams;
use crate::domain::rules::RuleSet;

#[derive(Debug, Clone)]
pub struct ProjectData {
    pub params: ThemeParams,
    pub rules: RuleSet,
}

#[derive(Debug, Clone)]
pub enum Effect {
    SaveProject { path: PathBuf, project: ProjectData },
    LoadProject { path: PathBuf },
    ExportAlacritty { path: PathBuf, theme: ResolvedTheme },
}
