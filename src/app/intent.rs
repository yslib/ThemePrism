use std::path::PathBuf;

use crate::app::controls::ControlId;
use crate::app::effect::ProjectData;

#[derive(Debug, Clone)]
pub enum Intent {
    QuitRequested,
    MoveFocus(i32),
    MoveSelection(i32),
    AdjustControlByStep(ControlId, i32),
    ActivateControl(ControlId),

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
