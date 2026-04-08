use std::path::PathBuf;

use thiserror::Error;

use crate::app::controls::{ControlId, ReferenceField};
use crate::app::interaction::InteractionState;
use crate::app::interaction::SurfaceId;
use crate::app::ui_meta::panel_workspace_tab;
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::domain::color::Color;
use crate::domain::evaluator::{resolve_theme, EvalError, ResolvedTheme};
use crate::domain::palette::{generate_palette, Palette};
use crate::domain::params::{ParamKey, ThemeParams};
use crate::domain::preview::PreviewState;
use crate::domain::rules::RuleSet;
use crate::domain::tokens::{PaletteSlot, TokenRole};
use crate::export::{default_export_profiles, ExportProfile};
use crate::i18n::{self, UiText};
use crate::persistence::editor_config::{
    EditorConfig, EditorKeymapPreset, EditorLocale, DEFAULT_PROJECT_PATH,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputState {
    pub target: TextInputTarget,
    pub buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputTarget {
    Control(ControlId),
    Config(ConfigFieldId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePickerState {
    pub control: ControlId,
    pub filter: String,
    pub selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFieldId {
    ProjectName,
    ExportEnabled(usize),
    ExportOutputPath(usize),
    ExportTemplatePath(usize),
    EditorProjectPath,
    EditorKeymapPreset,
    EditorLocale,
}

impl ConfigFieldId {
    pub fn supports_text_input(self) -> bool {
        matches!(
            self,
            Self::ProjectName
                | Self::ExportOutputPath(_)
                | Self::ExportTemplatePath(_)
                | Self::EditorProjectPath
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigModalState {
    pub selected_field: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteState {
    pub query: String,
    pub selected: usize,
}

#[derive(Debug, Clone)]
pub struct DomainState {
    pub params: ThemeParams,
    pub rules: RuleSet,
    pub palette: Palette,
    pub resolved: ResolvedTheme,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub selected_token: usize,
    pub selected_param: usize,
    pub inspector_field: usize,
    pub active_tab: WorkspaceTab,
    pub theme_panel: PanelId,
    pub project_panel: PanelId,
    pub project_field: usize,
    pub export_field: usize,
    pub editor_field: usize,
    pub interaction_inspector_scroll: u16,
    pub status: String,
    pub should_quit: bool,
    pub fullscreen_surface: Option<SurfaceId>,
    pub text_input: Option<TextInputState>,
    pub source_picker: Option<SourcePickerState>,
    pub config_modal: Option<ConfigModalState>,
    pub command_palette: Option<CommandPaletteState>,
    pub shortcut_help_open: bool,
    pub shortcut_help_scroll: u16,
    pub interaction: InteractionState,
}

#[derive(Debug, Clone)]
pub struct ProjectState {
    pub name: String,
    pub export_profiles: Vec<ExportProfile>,
}

#[derive(Debug, Clone)]
pub struct EditorState {
    pub project_path: PathBuf,
    pub keymap_preset: EditorKeymapPreset,
    pub locale: EditorLocale,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub domain: DomainState,
    pub ui: UiState,
    pub project: ProjectState,
    pub editor: EditorState,
    pub preview: PreviewState,
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct AppStateError(#[from] EvalError);

impl AppState {
    pub fn new() -> Result<Self, AppStateError> {
        let params = ThemeParams::default();
        let rules = RuleSet::default();
        let palette = generate_palette(&params);
        let resolved = resolve_theme(palette.clone(), &rules)?;

        Ok(Self {
            domain: DomainState {
                params,
                rules,
                palette,
                resolved,
            },
            ui: UiState {
                selected_token: 0,
                selected_param: 0,
                inspector_field: 0,
                active_tab: WorkspaceTab::Theme,
                theme_panel: PanelId::Tokens,
                project_panel: PanelId::ProjectConfig,
                project_field: 0,
                export_field: 0,
                editor_field: 0,
                interaction_inspector_scroll: 0,
                status: i18n::text(EditorLocale::EnUs, UiText::StatusReady),
                should_quit: false,
                fullscreen_surface: None,
                text_input: None,
                source_picker: None,
                config_modal: None,
                command_palette: None,
                shortcut_help_open: false,
                shortcut_help_scroll: 0,
                interaction: InteractionState::new(PanelId::Tokens),
            },
            project: ProjectState {
                name: "Untitled Theme".to_string(),
                export_profiles: default_export_profiles(),
            },
            editor: EditorState {
                project_path: PathBuf::from(DEFAULT_PROJECT_PATH),
                keymap_preset: EditorKeymapPreset::Standard,
                locale: EditorLocale::EnUs,
            },
            preview: PreviewState::default(),
        })
    }

    pub fn recompute(&mut self) -> Result<(), EvalError> {
        self.domain.palette = generate_palette(&self.domain.params);
        self.domain.resolved = resolve_theme(self.domain.palette.clone(), &self.domain.rules)?;
        Ok(())
    }

    pub fn editor_config(&self) -> EditorConfig {
        EditorConfig {
            project_path: self.editor.project_path.clone(),
            keymap_preset: self.editor.keymap_preset,
            locale: self.editor.locale,
        }
    }

    pub const fn locale(&self) -> EditorLocale {
        self.editor.locale
    }

    pub fn apply_project_data(
        &mut self,
        project: crate::app::effect::ProjectData,
    ) -> Result<(), String> {
        self.project.name = project.name;
        self.domain.params = project.params;
        self.domain.rules = project.rules;
        self.project.export_profiles = project.export_profiles;
        self.ui.text_input = None;
        self.ui.source_picker = None;
        self.ui.config_modal = None;
        self.ui.command_palette = None;
        self.ui.shortcut_help_open = false;
        self.ui.fullscreen_surface = None;
        self.preview.capture_active = false;
        self.ui
            .interaction
            .set_mode(crate::app::interaction::InteractionMode::Normal);
        self.recompute().map_err(|err| err.to_string())?;
        self.ui.inspector_field = self
            .ui
            .inspector_field
            .min(self.inspector_field_count().saturating_sub(1));
        self.ui.export_field = self
            .ui
            .export_field
            .min(self.export_fields().len().saturating_sub(1));
        Ok(())
    }

    pub fn active_panel(&self) -> PanelId {
        match self.ui.active_tab {
            WorkspaceTab::Theme => self.ui.theme_panel,
            WorkspaceTab::Project => self.ui.project_panel,
        }
    }

    pub fn set_active_panel(&mut self, panel: PanelId) {
        self.ui.active_tab = panel_workspace_tab(panel);
        match panel_workspace_tab(panel) {
            WorkspaceTab::Theme => self.ui.theme_panel = panel,
            WorkspaceTab::Project => self.ui.project_panel = panel,
        }
    }

    pub fn selected_role(&self) -> TokenRole {
        TokenRole::ALL[self.ui.selected_token]
    }

    pub fn selected_param_key(&self) -> ParamKey {
        ParamKey::ALL[self.ui.selected_param]
    }

    pub fn selected_rule(&self) -> &crate::domain::rules::Rule {
        self.domain
            .rules
            .get(self.selected_role())
            .expect("every token should have a rule")
    }

    pub fn current_token_color(&self) -> Color {
        self.domain
            .resolved
            .token(self.selected_role())
            .expect("resolved theme should contain every token")
    }

    pub fn theme_color(&self, role: TokenRole) -> Color {
        self.domain
            .resolved
            .token(role)
            .expect("resolved theme should contain every token")
    }

    pub fn palette_color(&self, slot: PaletteSlot) -> Color {
        self.domain
            .palette
            .get(slot)
            .expect("palette should contain every slot")
    }

    pub fn inspector_field_count(&self) -> usize {
        1 + match self.selected_rule() {
            crate::domain::rules::Rule::Alias { .. } => 1,
            crate::domain::rules::Rule::Mix { .. } => 3,
            crate::domain::rules::Rule::Adjust { .. } => 3,
            crate::domain::rules::Rule::Fixed { .. } => 1,
        }
    }

    pub fn project_fields(&self) -> [ConfigFieldId; 1] {
        [ConfigFieldId::ProjectName]
    }

    pub fn export_fields(&self) -> Vec<ConfigFieldId> {
        let mut fields = Vec::new();
        for (index, _) in self.project.export_profiles.iter().enumerate() {
            fields.push(ConfigFieldId::ExportEnabled(index));
            fields.push(ConfigFieldId::ExportOutputPath(index));
            fields.push(ConfigFieldId::ExportTemplatePath(index));
        }
        fields
    }

    pub fn editor_fields(&self) -> [ConfigFieldId; 3] {
        [
            ConfigFieldId::EditorProjectPath,
            ConfigFieldId::EditorKeymapPreset,
            ConfigFieldId::EditorLocale,
        ]
    }

    pub fn active_config_field(&self) -> Option<ConfigFieldId> {
        match self.active_panel() {
            PanelId::ProjectConfig => {
                let fields = self.project_fields();
                fields
                    .get(self.ui.project_field.min(fields.len().saturating_sub(1)))
                    .copied()
            }
            PanelId::ExportTargets => {
                let fields = self.export_fields();
                fields
                    .get(self.ui.export_field.min(fields.len().saturating_sub(1)))
                    .copied()
            }
            PanelId::EditorPreferences => {
                let fields = self.editor_fields();
                fields
                    .get(self.ui.editor_field.min(fields.len().saturating_sub(1)))
                    .copied()
            }
            _ => None,
        }
    }

    pub fn active_control(&self) -> Option<ControlId> {
        match self.active_panel() {
            PanelId::Tokens
            | PanelId::Preview
            | PanelId::Palette
            | PanelId::ResolvedPrimary
            | PanelId::ResolvedSecondary
            | PanelId::InteractionInspector => None,
            PanelId::Params => Some(ControlId::Param(self.selected_param_key())),
            PanelId::Inspector => match (self.selected_rule(), self.ui.inspector_field) {
                (crate::domain::rules::Rule::Alias { .. }, 0) => {
                    Some(ControlId::RuleKind(self.selected_role()))
                }
                (crate::domain::rules::Rule::Alias { .. }, 1) => Some(ControlId::Reference(
                    self.selected_role(),
                    ReferenceField::AliasSource,
                )),
                (crate::domain::rules::Rule::Mix { .. }, 0) => {
                    Some(ControlId::RuleKind(self.selected_role()))
                }
                (crate::domain::rules::Rule::Mix { .. }, 1) => Some(ControlId::Reference(
                    self.selected_role(),
                    ReferenceField::MixA,
                )),
                (crate::domain::rules::Rule::Mix { .. }, 2) => Some(ControlId::Reference(
                    self.selected_role(),
                    ReferenceField::MixB,
                )),
                (crate::domain::rules::Rule::Mix { .. }, 3) => {
                    Some(ControlId::MixRatio(self.selected_role()))
                }
                (crate::domain::rules::Rule::Adjust { .. }, 0) => {
                    Some(ControlId::RuleKind(self.selected_role()))
                }
                (crate::domain::rules::Rule::Adjust { .. }, 1) => Some(ControlId::Reference(
                    self.selected_role(),
                    ReferenceField::AdjustSource,
                )),
                (crate::domain::rules::Rule::Adjust { .. }, 2) => {
                    Some(ControlId::AdjustOp(self.selected_role()))
                }
                (crate::domain::rules::Rule::Adjust { .. }, 3) => {
                    Some(ControlId::AdjustAmount(self.selected_role()))
                }
                (crate::domain::rules::Rule::Fixed { .. }, 0) => {
                    Some(ControlId::RuleKind(self.selected_role()))
                }
                (crate::domain::rules::Rule::Fixed { .. }, 1) => {
                    Some(ControlId::FixedColor(self.selected_role()))
                }
                _ => None,
            },
            PanelId::ProjectConfig | PanelId::ExportTargets | PanelId::EditorPreferences => None,
        }
    }

    pub fn fixed_color_options(&self) -> Vec<Color> {
        let mut options = Vec::new();
        for slot in PaletteSlot::ALL {
            options.push(self.palette_color(slot));
        }
        for role in TokenRole::ALL {
            options.push(self.theme_color(role));
        }
        options.extend([
            Color::from_hex("#FF6B6B").expect("valid color"),
            Color::from_hex("#FFD166").expect("valid color"),
            Color::from_hex("#06D6A0").expect("valid color"),
            Color::from_hex("#4DABF7").expect("valid color"),
            Color::from_hex("#C77DFF").expect("valid color"),
        ]);
        options
    }
}
