use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use fluent_bundle::FluentValue;
use fluent_templates::{Loader, static_loader};

use crate::app::state::{ConfigFieldId, FocusPane};
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::domain::preview::PreviewMode;
use crate::persistence::editor_config::{EditorKeymapPreset, EditorLocale};

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
    };
}

macro_rules! define_ui_texts {
    ($($variant:ident => $key:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum UiText {
            $($variant),+
        }

        impl UiText {
            #[cfg_attr(not(test), allow(dead_code))]
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub const fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),+
                }
            }
        }
    };
}

define_ui_texts! {
    MenuTitle => "menu-title",
    MenuTabs => "menu-tabs",
    MenuPanels => "menu-panels",
    MenuEdit => "menu-edit",
    MenuHelp => "menu-help",
    MenuConfig => "menu-config",
    TabTheme => "tab-theme",
    TabProject => "tab-project",
    PanelTokenList => "panel-token-list",
    PanelThemeParams => "panel-theme-params",
    PanelPreviewSampleCode => "panel-preview-sample-code",
    PanelPalette => "panel-palette",
    PanelResolvedTokens => "panel-resolved-tokens",
    PanelResolvedTokensSecondary => "panel-resolved-tokens-secondary",
    PanelInspector => "panel-inspector",
    PanelProjectConfig => "panel-project-config",
    PanelExportTargets => "panel-export-targets",
    PanelEditorPreferences => "panel-editor-preferences",
    FieldProject => "field-project",
    FieldExports => "field-exports",
    FieldOutputs => "field-outputs",
    InspectorToken => "inspector-token",
    InspectorColor => "inspector-color",
    InspectorSummary => "inspector-summary",
    InspectorRuleType => "inspector-rule-type",
    InspectorSource => "inspector-source",
    InspectorColorA => "inspector-color-a",
    InspectorColorB => "inspector-color-b",
    InspectorBlend => "inspector-blend",
    InspectorOperation => "inspector-operation",
    InspectorAmount => "inspector-amount",
    InspectorHex => "inspector-hex",
    ProjectConfigFooter => "project-config-footer",
    ExportTargetsFooter => "export-targets-footer",
    EditorPreferencesFooter => "editor-preferences-footer",
    ConfigTitle => "config-title",
    ConfigSectionProjectSaved => "config-section-project-saved",
    ConfigSectionExportSaved => "config-section-export-saved",
    ConfigSectionEditorLocal => "config-section-editor-local",
    ConfigFooterEditHint => "config-footer-edit-hint",
    ConfigFooterProjectSaved => "config-footer-project-saved",
    ConfigFooterEditorLocal => "config-footer-editor-local",
    ConfigFooterCloseHint => "config-footer-close-hint",
    HelpTitle => "help-title",
    HelpFooterScroll => "help-footer-scroll",
    HelpFooterKeymap => "help-footer-keymap",
    NumericEditorTitle => "numeric-editor-title",
    NumericFooter => "numeric-footer",
    SourcePickerTitle => "source-picker-title",
    HelpSectionGlobal => "help-section-global",
    HelpSectionWorkspace => "help-section-workspace",
    HelpSectionPreview => "help-section-preview",
    HelpSectionPickerInput => "help-section-picker-input",
    HelpShortcutHelpLabel => "help-shortcut-help-label",
    HelpShortcutHelpDesc => "help-shortcut-help-desc",
    HelpConfigLabel => "help-config-label",
    HelpConfigDesc => "help-config-desc",
    HelpSaveProjectLabel => "help-save-project-label",
    HelpSaveProjectDesc => "help-save-project-desc",
    HelpLoadProjectLabel => "help-load-project-label",
    HelpLoadProjectDesc => "help-load-project-desc",
    HelpExportLabel => "help-export-label",
    HelpExportDesc => "help-export-desc",
    HelpResetLabel => "help-reset-label",
    HelpResetDesc => "help-reset-desc",
    HelpQuitLabel => "help-quit-label",
    HelpQuitDesc => "help-quit-desc",
    HelpSwitchTabsLabel => "help-switch-tabs-label",
    HelpSwitchTabsDesc => "help-switch-tabs-desc",
    HelpFocusPanelLabel => "help-focus-panel-label",
    HelpFocusPanelDesc => "help-focus-panel-desc",
    HelpCyclePanelsLabel => "help-cycle-panels-label",
    HelpCyclePanelsDesc => "help-cycle-panels-desc",
    HelpMoveSelectionLabel => "help-move-selection-label",
    HelpMoveSelectionDesc => "help-move-selection-desc",
    HelpAdjustValueLabel => "help-adjust-value-label",
    HelpAdjustValueDesc => "help-adjust-value-desc",
    HelpActivateLabel => "help-activate-label",
    HelpActivateDesc => "help-activate-desc",
    HelpPreviewModeLabel => "help-preview-mode-label",
    HelpPreviewModeDesc => "help-preview-mode-desc",
    HelpPreviewCaptureLabel => "help-preview-capture-label",
    HelpPreviewCaptureDesc => "help-preview-capture-desc",
    HelpPreviewReleaseLabel => "help-preview-release-label",
    HelpPreviewReleaseDesc => "help-preview-release-desc",
    HelpTypeFilterLabel => "help-type-filter-label",
    HelpTypeFilterDesc => "help-type-filter-desc",
    HelpDeleteBackwardLabel => "help-delete-backward-label",
    HelpDeleteBackwardDesc => "help-delete-backward-desc",
    HelpClearLabel => "help-clear-label",
    HelpClearDesc => "help-clear-desc",
    HelpNudgeNumericLabel => "help-nudge-numeric-label",
    HelpNudgeNumericDesc => "help-nudge-numeric-desc",
    HelpApplyLabel => "help-apply-label",
    HelpApplyDesc => "help-apply-desc",
    HelpCancelLabel => "help-cancel-label",
    HelpCancelDesc => "help-cancel-desc",
    KeymapPresetTitle => "keymap-preset-title",
    KeymapStandardLabel => "keymap-standard-label",
    KeymapStandardDesc => "keymap-standard-desc",
    KeymapVimLabel => "keymap-vim-label",
    KeymapVimDesc => "keymap-vim-desc",
    ConfigLabelProjectName => "config-label-project-name",
    ConfigLabelTarget => "config-label-target",
    ConfigLabelOutput => "config-label-output",
    ConfigLabelTemplate => "config-label-template",
    ConfigLabelProjectFile => "config-label-project-file",
    ConfigLabelAutoLoad => "config-label-auto-load",
    ConfigLabelAutoSave => "config-label-auto-save",
    ConfigLabelStartupFocus => "config-label-startup-focus",
    ConfigLabelKeymap => "config-label-keymap",
    ConfigLabelLanguage => "config-label-language",
    ConfigValueLoadProjectOnStartup => "config-value-load-project-on-startup",
    ConfigValueSaveProjectBeforeExport => "config-value-save-project-before-export",
    ConfigValueMissingExportTarget => "config-value-missing-export-target",
    SummaryNoneEnabled => "summary-none-enabled",
    SummaryOneEnabledNamed => "summary-one-enabled-named",
    SummaryManyEnabledNamed => "summary-many-enabled-named",
    SummaryManyEnabledCount => "summary-many-enabled-count",
    OutputsNoneEnabled => "outputs-none-enabled",
    OutputsOnePath => "outputs-one-path",
    OutputsManyPaths => "outputs-many-paths",
    ExportStatusNoneEnabled => "export-status-none-enabled",
    ExportStatusOneEnabled => "export-status-one-enabled",
    ExportStatusNamed => "export-status-named",
    ExportStatusCount => "export-status-count",
    StatusBarExports => "status-bar-exports",
    FocusTokens => "focus-tokens",
    FocusParams => "focus-params",
    FocusInspector => "focus-inspector",
    LocaleEnglish => "locale-english",
    LocaleChineseSimplified => "locale-chinese-simplified",
    SurfaceMainWindow => "surface-main-window",
    SurfaceInputEditor => "surface-input-editor",
    SurfaceSourcePicker => "surface-source-picker",
    SurfaceConfigDialog => "surface-config-dialog",
    SurfaceShortcutHelp => "surface-shortcut-help",
    PreviewModeCode => "preview-mode-code",
    PreviewModeShell => "preview-mode-shell",
    PreviewModeLazygit => "preview-mode-lazygit",
    PreviewHeaderSemanticSample => "preview-header-semantic-sample",
    PreviewHeaderSwitchModes => "preview-header-switch-modes",
    PreviewHeaderCaptureActive => "preview-header-capture-active",
    PreviewWaitingTitle => "preview-waiting-title",
    PreviewWaitingDetail => "preview-waiting-detail",
    PreviewExitedTitle => "preview-exited-title",
    WindowTitle => "window-title",
    GuiSectionThemeParameters => "gui-section-theme-parameters",
    GuiSectionPalette => "gui-section-palette",
    GuiSectionPreview => "gui-section-preview",
    GuiSectionInspector => "gui-section-inspector",
    GuiSectionEditorPreferences => "gui-section-editor-preferences",
    GuiSectionActions => "gui-section-actions",
    GuiButtonConfig => "gui-button-config",
    GuiButtonSave => "gui-button-save",
    GuiButtonLoad => "gui-button-load",
    GuiButtonExport => "gui-button-export",
    GuiButtonReset => "gui-button-reset",
    GuiSheetTitle => "gui-sheet-title",
    GuiSheetSubtitle => "gui-sheet-subtitle",
    GuiButtonDone => "gui-button-done",
    GuiSheetSectionProject => "gui-sheet-section-project",
    GuiSheetSectionExportTargets => "gui-sheet-section-export-targets",
    GuiSheetSectionEditorPreferences => "gui-sheet-section-editor-preferences",
    GuiColorPlaceholder => "gui-color-placeholder",
    StatusReady => "status-ready",
    StatusInputCancelled => "status-input-cancelled",
    StatusSourcePickerClosed => "status-source-picker-closed",
    StatusConfigOpened => "status-config-opened",
    StatusConfigClosed => "status-config-closed",
    StatusHelpOpened => "status-help-opened",
    StatusHelpClosed => "status-help-closed",
    StatusSavedProject => "status-saved-project",
    StatusSaveFailed => "status-save-failed",
    StatusLoadedProject => "status-loaded-project",
    StatusLoadRecomputeFailed => "status-load-recompute-failed",
    StatusLoadFailed => "status-load-failed",
    StatusExportNoOutput => "status-export-no-output",
    StatusExportedSingle => "status-exported-single",
    StatusExportedCount => "status-exported-count",
    StatusExportFailed => "status-export-failed",
    StatusEditorConfigSaveFailed => "status-editor-config-save-failed",
    StatusFocusedSurface => "status-focused-surface",
    StatusSurfaceNavigationActive => "status-surface-navigation-active",
    StatusPreviewModeChanged => "status-preview-mode-changed",
    StatusPreviewCaptureActive => "status-preview-capture-active",
    StatusPreviewCaptureReleased => "status-preview-capture-released",
    StatusPreviewProcessExited => "status-preview-process-exited",
    StatusSwitchedTab => "status-switched-tab",
    StatusFocusedPanel => "status-focused-panel",
    StatusTabOnlyHasPanels => "status-tab-only-has-panels",
    StatusSelectedToken => "status-selected-token",
    StatusSelectedParam => "status-selected-param",
    StatusPanelNoListSelection => "status-panel-no-list-selection",
    StatusPanelNoEditableFields => "status-panel-no-editable-fields",
    StatusSelectedField => "status-selected-field",
    StatusControlNoActivation => "status-control-no-activation",
    StatusFailedToUpdateField => "status-failed-to-update-field",
    StatusUpdatedFieldValue => "status-updated-field-value",
    StatusUpdatedEntity => "status-updated-entity",
    StatusRuleChangeRejected => "status-rule-change-rejected",
    StatusEditingNumeric => "status-editing-numeric",
    StatusEditingText => "status-editing-text",
    StatusSelectingSource => "status-selecting-source",
    StatusInvalidCharacter => "status-invalid-character",
    ErrorControlNoTextInput => "error-control-no-text-input",
    ErrorNoSourcesMatch => "error-no-sources-match",
    StatusSourceApplied => "status-source-applied",
    StatusSourceChangeRejected => "status-source-change-rejected",
    StatusBlendUpdated => "status-blend-updated",
    StatusAmountUpdated => "status-amount-updated",
    StatusFixedColorUpdated => "status-fixed-color-updated",
    StatusResetDefaults => "status-reset-defaults",
    StatusResetFailed => "status-reset-failed",
    ErrorProjectNameEmpty => "error-project-name-empty",
    StatusProjectNameUpdated => "status-project-name-updated",
    StatusExportTargetEnabled => "status-export-target-enabled",
    StatusExportTargetDisabled => "status-export-target-disabled",
    StatusMissingExportTarget => "status-missing-export-target",
    StatusExportOutputUpdated => "status-export-output-updated",
    StatusExportTemplateUpdated => "status-export-template-updated",
    ErrorExportNoTemplatePath => "error-export-no-template-path",
    StatusProjectFilePathUpdated => "status-project-file-path-updated",
    StatusAutoLoadEnabled => "status-auto-load-enabled",
    StatusAutoLoadDisabled => "status-auto-load-disabled",
    StatusAutoSaveEnabled => "status-auto-save-enabled",
    StatusAutoSaveDisabled => "status-auto-save-disabled",
    StatusStartupFocusUpdated => "status-startup-focus-updated",
    StatusKeymapUpdated => "status-keymap-updated",
    StatusLanguageUpdated => "status-language-updated",
    ErrorInputEmpty => "error-input-empty",
    ErrorUseToggleChoicePreference => "error-use-toggle-choice-preference",
    ErrorToggleExportTarget => "error-toggle-export-target",
    ErrorInvalidHexColor => "error-invalid-hex-color",
    ErrorInvalidNumber => "error-invalid-number",
    StatusAutoLoadRecomputeFailed => "status-auto-load-recompute-failed",
    StatusAutoLoadedProject => "status-auto-loaded-project",
    StatusAutoLoadFailed => "status-auto-load-failed",
    StatusEditorConfigLoadFailed => "status-editor-config-load-failed",
    FieldProjectNameLower => "field-project-name-lower",
    FieldExportTarget => "field-export-target",
    FieldExportOutputPath => "field-export-output-path",
    FieldExportTemplatePath => "field-export-template-path",
    FieldProjectFilePath => "field-project-file-path",
    FieldAutoLoadProject => "field-auto-load-project",
    FieldAutoSaveProject => "field-auto-save-project",
    FieldStartupFocusLower => "field-startup-focus-lower",
    FieldKeymapPresetLower => "field-keymap-preset-lower",
    FieldLanguageLower => "field-language-lower",
    FooterFilterSources => "footer-filter-sources",
    FooterNumericEditorOpen => "footer-numeric-editor-open",
    FooterFixedColorInput => "footer-fixed-color-input",
    FooterTextInput => "footer-text-input",
    FooterGenericInput => "footer-generic-input",
    FooterReferenceQuick => "footer-reference-quick",
    FooterFixedColorQuick => "footer-fixed-color-quick",
    FooterMixQuick => "footer-mix-quick",
    FooterAdjustQuick => "footer-adjust-quick",
    FooterRuleKindQuick => "footer-rule-kind-quick",
    FooterAdjustOpQuick => "footer-adjust-op-quick",
    FooterParamQuick => "footer-param-quick",
    FooterDefaultQuick => "footer-default-quick",
}

pub fn text(locale: EditorLocale, key: UiText) -> String {
    let lang = locale.language_identifier();
    LOCALES.lookup(&lang, key.key())
}

pub fn format1(locale: EditorLocale, key: UiText, name: &str, value: impl ToString) -> String {
    let lang = locale.language_identifier();
    let args = HashMap::from([(
        Cow::Owned(name.to_string()),
        FluentValue::from(value.to_string()),
    )]);
    LOCALES.lookup_with_args(&lang, key.key(), &args)
}

pub fn format2(
    locale: EditorLocale,
    key: UiText,
    name1: &str,
    value1: impl ToString,
    name2: &str,
    value2: impl ToString,
) -> String {
    let lang = locale.language_identifier();
    let args = HashMap::from([
        (
            Cow::Owned(name1.to_string()),
            FluentValue::from(value1.to_string()),
        ),
        (
            Cow::Owned(name2.to_string()),
            FluentValue::from(value2.to_string()),
        ),
    ]);
    LOCALES.lookup_with_args(&lang, key.key(), &args)
}

pub fn workspace_tab_label(locale: EditorLocale, tab: WorkspaceTab) -> String {
    match tab {
        WorkspaceTab::Theme => text(locale, UiText::TabTheme),
        WorkspaceTab::Project => text(locale, UiText::TabProject),
    }
}

pub fn panel_label(locale: EditorLocale, panel: PanelId) -> String {
    match panel {
        PanelId::Tokens => text(locale, UiText::PanelTokenList),
        PanelId::Params => text(locale, UiText::PanelThemeParams),
        PanelId::Preview => text(locale, UiText::PanelPreviewSampleCode),
        PanelId::Palette => text(locale, UiText::PanelPalette),
        PanelId::ResolvedPrimary => text(locale, UiText::PanelResolvedTokens),
        PanelId::ResolvedSecondary => text(locale, UiText::PanelResolvedTokensSecondary),
        PanelId::Inspector => text(locale, UiText::PanelInspector),
        PanelId::ProjectConfig => text(locale, UiText::PanelProjectConfig),
        PanelId::ExportTargets => text(locale, UiText::PanelExportTargets),
        PanelId::EditorPreferences => text(locale, UiText::PanelEditorPreferences),
    }
}

pub fn focus_pane_label(locale: EditorLocale, focus: FocusPane) -> String {
    match focus {
        FocusPane::Tokens => text(locale, UiText::FocusTokens),
        FocusPane::Params => text(locale, UiText::FocusParams),
        FocusPane::Inspector => text(locale, UiText::FocusInspector),
    }
}

pub fn keymap_preset_label(locale: EditorLocale, preset: EditorKeymapPreset) -> String {
    match preset {
        EditorKeymapPreset::Standard => text(locale, UiText::KeymapStandardLabel),
        EditorKeymapPreset::Vim => text(locale, UiText::KeymapVimLabel),
    }
}

pub fn locale_label(locale: EditorLocale, choice: EditorLocale) -> String {
    match choice {
        EditorLocale::EnUs => text(locale, UiText::LocaleEnglish),
        EditorLocale::ZhCn => text(locale, UiText::LocaleChineseSimplified),
    }
}

pub fn preview_mode_label(locale: EditorLocale, mode: PreviewMode) -> String {
    match mode {
        PreviewMode::Code => text(locale, UiText::PreviewModeCode),
        PreviewMode::Shell => text(locale, UiText::PreviewModeShell),
        PreviewMode::Lazygit => text(locale, UiText::PreviewModeLazygit),
    }
}

pub fn config_field_label(locale: EditorLocale, field: ConfigFieldId) -> String {
    match field {
        ConfigFieldId::ProjectName => text(locale, UiText::ConfigLabelProjectName),
        ConfigFieldId::ExportEnabled(index) => {
            format1(locale, UiText::ConfigLabelTarget, "index", index + 1)
        }
        ConfigFieldId::ExportOutputPath(index) => {
            format1(locale, UiText::ConfigLabelOutput, "index", index + 1)
        }
        ConfigFieldId::ExportTemplatePath(index) => {
            format1(locale, UiText::ConfigLabelTemplate, "index", index + 1)
        }
        ConfigFieldId::EditorProjectPath => text(locale, UiText::ConfigLabelProjectFile),
        ConfigFieldId::EditorAutoLoadProject => text(locale, UiText::ConfigLabelAutoLoad),
        ConfigFieldId::EditorAutoSaveOnExport => text(locale, UiText::ConfigLabelAutoSave),
        ConfigFieldId::EditorStartupFocus => text(locale, UiText::ConfigLabelStartupFocus),
        ConfigFieldId::EditorKeymapPreset => text(locale, UiText::ConfigLabelKeymap),
        ConfigFieldId::EditorLocale => text(locale, UiText::ConfigLabelLanguage),
    }
}

pub fn window_title(locale: EditorLocale, project_name: &str) -> String {
    format1(locale, UiText::WindowTitle, "name", project_name)
}

#[cfg_attr(not(test), allow(dead_code))]
fn locale_file_message_ids(path: &Path) -> std::collections::BTreeSet<String> {
    let content = std::fs::read_to_string(path).expect("locale file should be readable");
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('.') {
                return None;
            }

            let (id, _) = trimmed.split_once('=')?;
            Some(id.trim().to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{UiText, locale_file_message_ids, text};
    use crate::persistence::editor_config::EditorLocale;

    #[test]
    fn every_ui_text_key_exists_in_both_locales() {
        let expected = UiText::ALL
            .iter()
            .map(|key| key.key().to_string())
            .collect::<BTreeSet<_>>();

        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for relative in ["locales/en-US/ui.ftl", "locales/zh-CN/ui.ftl"] {
            let actual = locale_file_message_ids(&root.join(relative));
            let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
            assert!(
                missing.is_empty(),
                "missing translation keys in {relative}: {missing:?}"
            );
        }
    }

    #[test]
    fn locales_return_different_ui_copy() {
        assert_eq!(text(EditorLocale::EnUs, UiText::MenuHelp), "Help");
        assert_eq!(text(EditorLocale::ZhCn, UiText::MenuHelp), "帮助");
    }
}
