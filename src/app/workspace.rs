#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTab {
    Theme,
    Project,
}

impl WorkspaceTab {
    pub const ALL: [Self; 2] = [Self::Theme, Self::Project];

    pub const fn next(self) -> Self {
        match self {
            Self::Theme => Self::Project,
            Self::Project => Self::Theme,
        }
    }

    pub const fn previous(self) -> Self {
        match self {
            Self::Theme => Self::Project,
            Self::Project => Self::Theme,
        }
    }

    #[allow(dead_code)]
    pub const fn default_panel(self) -> PanelId {
        match self {
            Self::Theme => PanelId::Tokens,
            Self::Project => PanelId::ProjectConfig,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelId {
    Tokens,
    Params,
    Preview,
    Palette,
    ResolvedPrimary,
    ResolvedSecondary,
    Inspector,
    InteractionInspector,
    ProjectConfig,
    ExportTargets,
    EditorPreferences,
}
