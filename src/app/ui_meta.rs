use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::i18n::UiText;
use crate::preview::PreviewMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelSpec {
    pub id: PanelId,
    pub ui_text: UiText,
    pub workspace_tab: WorkspaceTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceTabSpec {
    pub id: WorkspaceTab,
    pub ui_text: UiText,
    pub hint_navigation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewModeSpec {
    pub id: PreviewMode,
    pub ui_text: UiText,
    pub hint_navigation: bool,
}

const PANEL_SPECS: &[PanelSpec] = &[
    PanelSpec {
        id: PanelId::Tokens,
        ui_text: UiText::PanelTokenList,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::Params,
        ui_text: UiText::PanelThemeParams,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::Preview,
        ui_text: UiText::PanelPreviewSampleCode,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::Palette,
        ui_text: UiText::PanelPalette,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::ResolvedPrimary,
        ui_text: UiText::PanelResolvedTokens,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::ResolvedSecondary,
        ui_text: UiText::PanelResolvedTokensSecondary,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::Inspector,
        ui_text: UiText::PanelInspector,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::InteractionInspector,
        ui_text: UiText::PanelInteractionInspector,
        workspace_tab: WorkspaceTab::Theme,
    },
    PanelSpec {
        id: PanelId::ProjectConfig,
        ui_text: UiText::PanelProjectConfig,
        workspace_tab: WorkspaceTab::Project,
    },
    PanelSpec {
        id: PanelId::ExportTargets,
        ui_text: UiText::PanelExportTargets,
        workspace_tab: WorkspaceTab::Project,
    },
    PanelSpec {
        id: PanelId::EditorPreferences,
        ui_text: UiText::PanelEditorPreferences,
        workspace_tab: WorkspaceTab::Project,
    },
];

const WORKSPACE_TAB_SPECS: &[WorkspaceTabSpec] = &[
    WorkspaceTabSpec {
        id: WorkspaceTab::Theme,
        ui_text: UiText::TabTheme,
        hint_navigation: true,
    },
    WorkspaceTabSpec {
        id: WorkspaceTab::Project,
        ui_text: UiText::TabProject,
        hint_navigation: true,
    },
];

const PREVIEW_MODE_SPECS: &[PreviewModeSpec] = &[
    PreviewModeSpec {
        id: PreviewMode::Code,
        ui_text: UiText::PreviewModeCode,
        hint_navigation: true,
    },
    PreviewModeSpec {
        id: PreviewMode::Shell,
        ui_text: UiText::PreviewModeShell,
        hint_navigation: true,
    },
    PreviewModeSpec {
        id: PreviewMode::Lazygit,
        ui_text: UiText::PreviewModeLazygit,
        hint_navigation: true,
    },
];

pub fn panel_spec(id: PanelId) -> Option<&'static PanelSpec> {
    PANEL_SPECS.iter().find(|spec| spec.id == id)
}

pub fn workspace_tab_spec(id: WorkspaceTab) -> Option<&'static WorkspaceTabSpec> {
    WORKSPACE_TAB_SPECS.iter().find(|spec| spec.id == id)
}

pub fn preview_mode_spec(id: PreviewMode) -> Option<&'static PreviewModeSpec> {
    PREVIEW_MODE_SPECS.iter().find(|spec| spec.id == id)
}

#[cfg(test)]
mod tests {
    use super::{panel_spec, preview_mode_spec, workspace_tab_spec};
    use crate::app::workspace::{PanelId, WorkspaceTab};
    use crate::preview::PreviewMode;

    const ALL_PANELS: [PanelId; 11] = [
        PanelId::Tokens,
        PanelId::Params,
        PanelId::Preview,
        PanelId::Palette,
        PanelId::ResolvedPrimary,
        PanelId::ResolvedSecondary,
        PanelId::Inspector,
        PanelId::InteractionInspector,
        PanelId::ProjectConfig,
        PanelId::ExportTargets,
        PanelId::EditorPreferences,
    ];

    #[test]
    fn panel_specs_cover_every_panel_id() {
        let missing = ALL_PANELS
            .into_iter()
            .filter(|panel| panel_spec(*panel).is_none())
            .collect::<Vec<_>>();

        assert!(missing.is_empty(), "missing panel specs for: {missing:?}");
    }

    #[test]
    fn preview_mode_specs_cover_every_preview_mode() {
        let missing = PreviewMode::ALL
            .iter()
            .copied()
            .filter(|mode| preview_mode_spec(*mode).is_none())
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "missing preview mode specs for: {missing:?}"
        );
    }

    #[test]
    fn workspace_tab_specs_cover_every_workspace_tab() {
        let missing = WorkspaceTab::ALL
            .iter()
            .copied()
            .filter(|tab| workspace_tab_spec(*tab).is_none())
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "missing workspace tab specs for: {missing:?}"
        );
    }
}
