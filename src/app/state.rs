use std::fmt;
use std::path::PathBuf;

use crate::app::controls::{ControlId, ReferenceField};
use crate::domain::color::Color;
use crate::domain::evaluator::{EvalError, ResolvedTheme, resolve_theme};
use crate::domain::palette::{Palette, generate_palette};
use crate::domain::params::{ParamKey, ThemeParams};
use crate::domain::rules::RuleSet;
use crate::domain::tokens::{PaletteSlot, TokenRole};
use crate::export::{ExportProfile, default_export_profiles};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Tokens,
    Params,
    Inspector,
}

impl FocusPane {
    pub const fn next(self) -> Self {
        match self {
            Self::Tokens => Self::Params,
            Self::Params => Self::Inspector,
            Self::Inspector => Self::Tokens,
        }
    }

    pub const fn previous(self) -> Self {
        match self {
            Self::Tokens => Self::Inspector,
            Self::Params => Self::Tokens,
            Self::Inspector => Self::Params,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Tokens => "Tokens",
            Self::Params => "Params",
            Self::Inspector => "Inspector",
        }
    }
}

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
}

impl ConfigFieldId {
    pub fn supports_text_input(self) -> bool {
        matches!(
            self,
            Self::ProjectName | Self::ExportOutputPath(_) | Self::ExportTemplatePath(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigModalState {
    pub selected_field: usize,
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
    pub focus: FocusPane,
    pub status: String,
    pub should_quit: bool,
    pub text_input: Option<TextInputState>,
    pub source_picker: Option<SourcePickerState>,
    pub config_modal: Option<ConfigModalState>,
}

#[derive(Debug, Clone)]
pub struct ProjectState {
    pub name: String,
    pub project_path: PathBuf,
    pub export_profiles: Vec<ExportProfile>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub domain: DomainState,
    pub ui: UiState,
    pub project: ProjectState,
}

#[derive(Debug)]
pub struct AppStateError(EvalError);

impl fmt::Display for AppStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for AppStateError {}

impl From<EvalError> for AppStateError {
    fn from(value: EvalError) -> Self {
        Self(value)
    }
}

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
                focus: FocusPane::Tokens,
                status: "Theme generator ready.".to_string(),
                should_quit: false,
                text_input: None,
                source_picker: None,
                config_modal: None,
            },
            project: ProjectState {
                name: "Untitled Theme".to_string(),
                project_path: PathBuf::from("projects/theme-project.toml"),
                export_profiles: default_export_profiles(),
            },
        })
    }

    pub fn recompute(&mut self) -> Result<(), EvalError> {
        self.domain.palette = generate_palette(&self.domain.params);
        self.domain.resolved = resolve_theme(self.domain.palette.clone(), &self.domain.rules)?;
        Ok(())
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

    pub fn active_control(&self) -> Option<ControlId> {
        match self.ui.focus {
            FocusPane::Tokens => None,
            FocusPane::Params => Some(ControlId::Param(self.selected_param_key())),
            FocusPane::Inspector => match (self.selected_rule(), self.ui.inspector_field) {
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
