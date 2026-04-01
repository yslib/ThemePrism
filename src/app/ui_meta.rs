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

pub const fn panel_spec(id: PanelId) -> PanelSpec {
    match id {
        PanelId::Tokens => PanelSpec {
            id,
            ui_text: UiText::PanelTokenList,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::Params => PanelSpec {
            id,
            ui_text: UiText::PanelThemeParams,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::Preview => PanelSpec {
            id,
            ui_text: UiText::PanelPreviewSampleCode,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::Palette => PanelSpec {
            id,
            ui_text: UiText::PanelPalette,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::ResolvedPrimary => PanelSpec {
            id,
            ui_text: UiText::PanelResolvedTokens,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::ResolvedSecondary => PanelSpec {
            id,
            ui_text: UiText::PanelResolvedTokensSecondary,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::Inspector => PanelSpec {
            id,
            ui_text: UiText::PanelInspector,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::InteractionInspector => PanelSpec {
            id,
            ui_text: UiText::PanelInteractionInspector,
            workspace_tab: WorkspaceTab::Theme,
        },
        PanelId::ProjectConfig => PanelSpec {
            id,
            ui_text: UiText::PanelProjectConfig,
            workspace_tab: WorkspaceTab::Project,
        },
        PanelId::ExportTargets => PanelSpec {
            id,
            ui_text: UiText::PanelExportTargets,
            workspace_tab: WorkspaceTab::Project,
        },
        PanelId::EditorPreferences => PanelSpec {
            id,
            ui_text: UiText::PanelEditorPreferences,
            workspace_tab: WorkspaceTab::Project,
        },
    }
}

pub const fn panel_workspace_tab(id: PanelId) -> WorkspaceTab {
    panel_spec(id).workspace_tab
}

pub const fn workspace_tab_spec(id: WorkspaceTab) -> WorkspaceTabSpec {
    match id {
        WorkspaceTab::Theme => WorkspaceTabSpec {
            id,
            ui_text: UiText::TabTheme,
            hint_navigation: true,
        },
        WorkspaceTab::Project => WorkspaceTabSpec {
            id,
            ui_text: UiText::TabProject,
            hint_navigation: true,
        },
    }
}

pub const fn preview_mode_spec(id: PreviewMode) -> PreviewModeSpec {
    match id {
        PreviewMode::Code => PreviewModeSpec {
            id,
            ui_text: UiText::PreviewModeCode,
            hint_navigation: true,
        },
        PreviewMode::Shell => PreviewModeSpec {
            id,
            ui_text: UiText::PreviewModeShell,
            hint_navigation: true,
        },
        PreviewMode::Lazygit => PreviewModeSpec {
            id,
            ui_text: UiText::PreviewModeLazygit,
            hint_navigation: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
        for panel in ALL_PANELS {
            let spec = panel_spec(panel);
            assert_eq!(spec.id, panel);
        }
    }

    #[test]
    fn preview_mode_specs_cover_every_preview_mode() {
        for mode in PreviewMode::ALL {
            let spec = preview_mode_spec(mode);
            assert_eq!(spec.id, mode);
        }
    }

    #[test]
    fn workspace_tab_specs_cover_every_workspace_tab() {
        for tab in WorkspaceTab::ALL {
            let spec = workspace_tab_spec(tab);
            assert_eq!(spec.id, tab);
        }
    }

    #[test]
    fn panel_workspace_membership_is_not_duplicated_in_workspace_module() {
        let workspace_source = source_file("src/app/workspace.rs");

        assert!(
            !workspace_source.contains("pub const fn tab(self) -> WorkspaceTab"),
            "panel ownership mapping should live outside workspace.rs"
        );
    }

    #[test]
    fn panel_spec_lookup_is_not_optional() {
        let ui_meta_source = source_file("src/app/ui_meta.rs");
        let production_source = ui_meta_source
            .split("#[cfg(test)]")
            .next()
            .expect("ui_meta.rs should have production code before tests");

        assert!(
            !production_source.contains("-> Option<&'static PanelSpec>"),
            "panel_spec should be exhaustive instead of optional"
        );
    }

    fn source_file(relative_path: &str) -> String {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(root.join(relative_path)).expect("source file should be readable")
    }
}
