use std::path::PathBuf;

use crate::app::controls::ControlId;
use crate::app::effect::ProjectData;
use crate::domain::color::Color;
use crate::domain::params::ParamKey;
use crate::domain::rules::{AdjustOp, RuleKind, SourceRef};
use crate::domain::tokens::TokenRole;

#[derive(Debug, Clone)]
pub enum Intent {
    QuitRequested,
    MoveFocus(i32),
    MoveSelection(i32),
    SelectToken(usize),
    AdjustControlByStep(ControlId, i32),
    ActivateControl(ControlId),

    SetParamValue(ParamKey, f32),
    SetRuleKind(TokenRole, RuleKind),
    SetReferenceSource(ControlId, SourceRef),
    SetMixRatio(TokenRole, f32),
    SetAdjustOp(TokenRole, AdjustOp),
    SetAdjustAmount(TokenRole, f32),
    SetFixedColor(TokenRole, Color),

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

    SaveProjectRequested,
    LoadProjectRequested,
    ExportThemeRequested,
    ResetRequested,

    ProjectSaved(Result<PathBuf, String>),
    ProjectLoaded(Result<ProjectData, String>),
    ThemeExported(Result<PathBuf, String>),
}
