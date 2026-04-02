use std::path::PathBuf;

use crate::app::controls::ControlId;
use crate::app::effect::ProjectData;
use crate::app::interaction::{InteractionMode, SurfaceId};
use crate::app::workspace::WorkspaceTab;
use crate::domain::color::Color;
use crate::domain::params::ParamKey;
use crate::domain::preview::PreviewRuntimeEvent;
use crate::domain::rules::{AdjustOp, RuleKind, SourceRef};
use crate::domain::tokens::TokenRole;
use crate::export::ExportArtifact;
use crate::persistence::editor_config::{EditorKeymapPreset, EditorLocale};
use crate::preview::PreviewMode;

#[derive(Debug, Clone)]
pub enum Intent {
    QuitRequested,
    CycleWorkspaceTab(i32),
    SetWorkspaceTab(WorkspaceTab),
    FocusPanelByNumber(u8),
    FocusSurface(SurfaceId),
    SetInteractionMode(InteractionMode),
    MoveSelection(i32),
    SelectToken(usize),
    AdjustControlByStep(ControlId, i32),
    ActivateControl(ControlId),
    AdjustActiveNumericInputByStep(i32),
    CyclePreviewMode(i32),
    SetPreviewMode(PreviewMode),
    SetPreviewCapture(bool),
    ToggleFullscreenRequested,

    SetParamValue(ParamKey, f32),
    SetRuleKind(TokenRole, RuleKind),
    SetReferenceSource(ControlId, SourceRef),
    SetMixRatio(TokenRole, f32),
    SetAdjustOp(TokenRole, AdjustOp),
    SetAdjustAmount(TokenRole, f32),
    SetFixedColor(TokenRole, Color),
    SetProjectName(String),
    SetExportEnabled(usize, bool),
    SetExportOutputPath(usize, PathBuf),
    SetExportTemplatePath(usize, PathBuf),
    SetEditorProjectPath(PathBuf),
    SetEditorKeymapPreset(EditorKeymapPreset),
    SetEditorLocale(EditorLocale),

    AppendTextInput(char),
    BackspaceTextInput,
    ClearTextInput,
    CommitTextInput,
    CancelTextInput,

    AppendSourcePickerFilter(char),
    BackspaceSourcePickerFilter,
    ClearSourcePickerFilter,
    MoveSourcePickerSelection(i32),
    ApplySourcePickerSelection,
    CloseSourcePicker,

    OpenConfigRequested,
    CloseConfigRequested,
    OpenCommandPaletteRequested,
    CloseCommandPaletteRequested,
    SetCommandPaletteQuery(String),
    AppendCommandPaletteQuery(char),
    BackspaceCommandPaletteQuery,
    ClearCommandPaletteQuery,
    MoveCommandPaletteSelection(i32),
    RunSelectedCommandPaletteItem,
    ToggleShortcutHelpRequested,
    ScrollShortcutHelp(i32),
    MoveConfigSelection(i32),
    ActivateConfigField,

    SaveProjectRequested,
    LoadProjectRequested,
    ExportThemeRequested,
    ResetRequested,

    ProjectSaved(Result<PathBuf, String>),
    ProjectLoaded(Result<ProjectData, String>),
    EditorConfigSaved(Result<PathBuf, String>),
    ThemeExported(Result<Vec<ExportArtifact>, String>),
    PreviewRuntimeEvent(PreviewRuntimeEvent),
}
