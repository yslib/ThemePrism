use crossterm::event::{KeyCode, KeyEvent};

use crate::i18n::{self, UiText};
use crate::persistence::editor_config::EditorKeymapPreset;
use crate::persistence::editor_config::EditorLocale;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionId {
    SwitchTabs,
    FocusPanels,
    MoveSelection,
    AdjustValue,
    Activate,
    Toggle,
    TypeInput,
    Filter,
    Nudge,
    Apply,
    Cancel,
    Clear,
    SwitchPreviewMode,
    CapturePreview,
    ReleasePreviewCapture,
    OpenConfig,
    OpenHelp,
    SaveProject,
    LoadProject,
    ExportTheme,
    Reset,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundAction {
    PreviousTab,
    NextTab,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Activate,
    Toggle,
    Apply,
    Cancel,
    Clear,
    Backspace,
    ReleasePreviewCapture,
    OpenConfig,
    OpenHelp,
    SaveProject,
    LoadProject,
    ExportTheme,
    Reset,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionHint {
    pub id: ActionId,
    pub shortcut: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionHelpSection {
    pub title: String,
    pub entries: Vec<ActionHelpEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionHelpEntry {
    pub shortcut: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyBinding {
    Char(char),
    Ctrl(char),
    Space,
    Enter,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Delete,
}

impl KeyBinding {
    fn matches(self, key: &KeyEvent) -> bool {
        match self {
            Self::Char(ch) => {
                !key.modifiers.intersects(
                    crossterm::event::KeyModifiers::CONTROL
                        | crossterm::event::KeyModifiers::ALT
                        | crossterm::event::KeyModifiers::SUPER,
                ) && matches!(key.code, KeyCode::Char(actual) if actual == ch)
            }
            Self::Ctrl(ch) => {
                key.modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                    && matches!(key.code, KeyCode::Char(actual) if actual.eq_ignore_ascii_case(&ch))
            }
            Self::Space => {
                !key.modifiers.intersects(
                    crossterm::event::KeyModifiers::CONTROL
                        | crossterm::event::KeyModifiers::ALT
                        | crossterm::event::KeyModifiers::SUPER,
                ) && matches!(key.code, KeyCode::Char(' '))
            }
            Self::Enter => matches!(key.code, KeyCode::Enter),
            Self::Esc => matches!(key.code, KeyCode::Esc),
            Self::Left => matches!(key.code, KeyCode::Left),
            Self::Right => matches!(key.code, KeyCode::Right),
            Self::Up => matches!(key.code, KeyCode::Up),
            Self::Down => matches!(key.code, KeyCode::Down),
            Self::Backspace => matches!(key.code, KeyCode::Backspace),
            Self::Delete => matches!(key.code, KeyCode::Delete),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Char(ch) => match ch {
                '?' => "?",
                '[' => "[",
                ']' => "]",
                _ => {
                    const FALLBACK: &str = "";
                    if ch == ' ' { "Space" } else { FALLBACK }
                }
            },
            Self::Ctrl(ch) => match ch {
                'a'..='z' => match ch {
                    'g' => "Ctrl+G",
                    _ => "Ctrl",
                },
                _ => "Ctrl",
            },
            Self::Space => "Space",
            Self::Enter => "Enter",
            Self::Esc => "Esc",
            Self::Left => "←",
            Self::Right => "→",
            Self::Up => "↑",
            Self::Down => "↓",
            Self::Backspace => "⌫",
            Self::Delete => "Del",
        }
    }
}

pub fn menu_bar_actions(locale: EditorLocale, preset: EditorKeymapPreset) -> Vec<ActionHint> {
    vec![
        hint(
            ActionId::SwitchTabs,
            switch_tabs_shortcut_label(preset),
            i18n::text(locale, UiText::MenuTabs),
        ),
        hint(
            ActionId::FocusPanels,
            "1-9",
            i18n::text(locale, UiText::MenuPanels),
        ),
        hint(
            ActionId::Activate,
            activate_shortcut_label(preset),
            i18n::text(locale, UiText::MenuEdit),
        ),
        hint(
            ActionId::OpenHelp,
            binding_label(preset, BoundAction::OpenHelp),
            i18n::text(locale, UiText::MenuHelp),
        ),
        hint(
            ActionId::OpenConfig,
            binding_label(preset, BoundAction::OpenConfig),
            i18n::text(locale, UiText::MenuConfig),
        ),
    ]
}

pub fn shortcut_help_sections(
    locale: EditorLocale,
    preset: EditorKeymapPreset,
) -> Vec<ActionHelpSection> {
    vec![
        ActionHelpSection {
            title: i18n::text(locale, UiText::HelpSectionGlobal),
            entries: vec![
                entry(
                    binding_label(preset, BoundAction::OpenHelp),
                    i18n::text(locale, UiText::HelpShortcutHelpLabel),
                    i18n::text(locale, UiText::HelpShortcutHelpDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::OpenConfig),
                    i18n::text(locale, UiText::HelpConfigLabel),
                    i18n::text(locale, UiText::HelpConfigDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::SaveProject),
                    i18n::text(locale, UiText::HelpSaveProjectLabel),
                    i18n::text(locale, UiText::HelpSaveProjectDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::LoadProject),
                    i18n::text(locale, UiText::HelpLoadProjectLabel),
                    i18n::text(locale, UiText::HelpLoadProjectDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::ExportTheme),
                    i18n::text(locale, UiText::HelpExportLabel),
                    i18n::text(locale, UiText::HelpExportDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::Reset),
                    i18n::text(locale, UiText::HelpResetLabel),
                    i18n::text(locale, UiText::HelpResetDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::Quit),
                    i18n::text(locale, UiText::HelpQuitLabel),
                    i18n::text(locale, UiText::HelpQuitDesc),
                ),
            ],
        },
        ActionHelpSection {
            title: i18n::text(locale, UiText::HelpSectionWorkspace),
            entries: vec![
                entry(
                    switch_tabs_shortcut_label(preset),
                    i18n::text(locale, UiText::HelpSwitchTabsLabel),
                    i18n::text(locale, UiText::HelpSwitchTabsDesc),
                ),
                entry(
                    "1-9".to_string(),
                    i18n::text(locale, UiText::HelpFocusPanelLabel),
                    i18n::text(locale, UiText::HelpFocusPanelDesc),
                ),
                entry(
                    move_selection_shortcut_label(preset),
                    i18n::text(locale, UiText::HelpMoveSelectionLabel),
                    i18n::text(locale, UiText::HelpMoveSelectionDesc),
                ),
                entry(
                    adjust_value_shortcut_label(preset),
                    i18n::text(locale, UiText::HelpAdjustValueLabel),
                    i18n::text(locale, UiText::HelpAdjustValueDesc),
                ),
                entry(
                    activate_shortcut_label(preset),
                    i18n::text(locale, UiText::HelpActivateLabel),
                    i18n::text(locale, UiText::HelpActivateDesc),
                ),
            ],
        },
        ActionHelpSection {
            title: i18n::text(locale, UiText::HelpSectionPreview),
            entries: vec![
                entry(
                    switch_tabs_shortcut_label(preset),
                    i18n::text(locale, UiText::HelpPreviewModeLabel),
                    i18n::text(locale, UiText::HelpPreviewModeDesc),
                ),
                entry(
                    activate_shortcut_label(preset),
                    i18n::text(locale, UiText::HelpPreviewCaptureLabel),
                    i18n::text(locale, UiText::HelpPreviewCaptureDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::ReleasePreviewCapture),
                    i18n::text(locale, UiText::HelpPreviewReleaseLabel),
                    i18n::text(locale, UiText::HelpPreviewReleaseDesc),
                ),
            ],
        },
        ActionHelpSection {
            title: i18n::text(locale, UiText::HelpSectionPickerInput),
            entries: vec![
                entry(
                    "type".to_string(),
                    i18n::text(locale, UiText::HelpTypeFilterLabel),
                    i18n::text(locale, UiText::HelpTypeFilterDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::Backspace),
                    i18n::text(locale, UiText::HelpDeleteBackwardLabel),
                    i18n::text(locale, UiText::HelpDeleteBackwardDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::Clear),
                    i18n::text(locale, UiText::HelpClearLabel),
                    i18n::text(locale, UiText::HelpClearDesc),
                ),
                entry(
                    adjust_value_shortcut_label(preset),
                    i18n::text(locale, UiText::HelpNudgeNumericLabel),
                    i18n::text(locale, UiText::HelpNudgeNumericDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::Apply),
                    i18n::text(locale, UiText::HelpApplyLabel),
                    i18n::text(locale, UiText::HelpApplyDesc),
                ),
                entry(
                    binding_label(preset, BoundAction::Cancel),
                    i18n::text(locale, UiText::HelpCancelLabel),
                    i18n::text(locale, UiText::HelpCancelDesc),
                ),
            ],
        },
        ActionHelpSection {
            title: i18n::format1(
                locale,
                UiText::KeymapPresetTitle,
                "preset",
                i18n::keymap_preset_label(locale, preset),
            ),
            entries: vec![match preset {
                EditorKeymapPreset::Standard => entry(
                    "arrows + enter".to_string(),
                    i18n::text(locale, UiText::KeymapStandardLabel),
                    i18n::text(locale, UiText::KeymapStandardDesc),
                ),
                EditorKeymapPreset::Vim => entry(
                    "arrows + hjkl + i".to_string(),
                    i18n::text(locale, UiText::KeymapVimLabel),
                    i18n::text(locale, UiText::KeymapVimDesc),
                ),
            }],
        },
    ]
}

pub fn matches_bound_action(
    preset: EditorKeymapPreset,
    action: BoundAction,
    key: &KeyEvent,
) -> bool {
    bindings_for(preset, action)
        .iter()
        .any(|binding| binding.matches(key))
}

pub fn binding_label(preset: EditorKeymapPreset, action: BoundAction) -> String {
    let bindings = bindings_for(preset, action);
    let mut out = String::new();
    for (index, binding) in bindings.iter().enumerate() {
        if index > 0 {
            out.push('/');
        }
        out.push_str(match binding {
            KeyBinding::Char(ch) => match ch {
                '?' | '[' | ']' => binding.label(),
                _ => {
                    out.push(*ch);
                    continue;
                }
            },
            KeyBinding::Ctrl(_) => binding.label(),
            _ => binding.label(),
        });
    }
    out
}

fn hint(id: ActionId, shortcut: impl Into<String>, label: impl Into<String>) -> ActionHint {
    ActionHint {
        id,
        shortcut: shortcut.into(),
        label: label.into(),
    }
}

fn entry(
    shortcut: impl Into<String>,
    label: impl Into<String>,
    description: impl Into<String>,
) -> ActionHelpEntry {
    ActionHelpEntry {
        shortcut: shortcut.into(),
        label: label.into(),
        description: description.into(),
    }
}

fn switch_tabs_shortcut_label(preset: EditorKeymapPreset) -> String {
    let previous = binding_label(preset, BoundAction::PreviousTab);
    let next = binding_label(preset, BoundAction::NextTab);
    if previous == "[" && next == "]" {
        "[/]".to_string()
    } else {
        format!("{previous}/{next}")
    }
}

fn move_selection_shortcut_label(preset: EditorKeymapPreset) -> String {
    match preset {
        EditorKeymapPreset::Standard => "↑↓".to_string(),
        EditorKeymapPreset::Vim => "↑↓/jk".to_string(),
    }
}

fn adjust_value_shortcut_label(preset: EditorKeymapPreset) -> String {
    match preset {
        EditorKeymapPreset::Standard => "←→".to_string(),
        EditorKeymapPreset::Vim => "←→/hl".to_string(),
    }
}

fn activate_shortcut_label(preset: EditorKeymapPreset) -> String {
    match preset {
        EditorKeymapPreset::Standard => "Enter".to_string(),
        EditorKeymapPreset::Vim => "Enter/i".to_string(),
    }
}

fn bindings_for(preset: EditorKeymapPreset, action: BoundAction) -> &'static [KeyBinding] {
    use BoundAction as Action;
    use EditorKeymapPreset as Preset;
    use KeyBinding as Key;

    match (preset, action) {
        (_, Action::PreviousTab) => &[Key::Char('[')],
        (_, Action::NextTab) => &[Key::Char(']')],
        (Preset::Standard, Action::MoveUp) => &[Key::Up],
        (Preset::Vim, Action::MoveUp) => &[Key::Up, Key::Char('k')],
        (Preset::Standard, Action::MoveDown) => &[Key::Down],
        (Preset::Vim, Action::MoveDown) => &[Key::Down, Key::Char('j')],
        (Preset::Standard, Action::MoveLeft) => &[Key::Left],
        (Preset::Vim, Action::MoveLeft) => &[Key::Left, Key::Char('h')],
        (Preset::Standard, Action::MoveRight) => &[Key::Right],
        (Preset::Vim, Action::MoveRight) => &[Key::Right, Key::Char('l')],
        (Preset::Standard, Action::Activate) => &[Key::Enter],
        (Preset::Vim, Action::Activate) => &[Key::Enter, Key::Char('i')],
        (_, Action::Toggle) => &[Key::Space],
        (_, Action::Apply) => &[Key::Enter],
        (_, Action::Cancel) => &[Key::Esc],
        (_, Action::Clear) => &[Key::Delete],
        (_, Action::Backspace) => &[Key::Backspace],
        (_, Action::ReleasePreviewCapture) => &[Key::Ctrl('g')],
        (_, Action::OpenConfig) => &[Key::Char('c')],
        (_, Action::OpenHelp) => &[Key::Char('?')],
        (_, Action::SaveProject) => &[Key::Char('s')],
        (_, Action::LoadProject) => &[Key::Char('o')],
        (_, Action::ExportTheme) => &[Key::Char('e')],
        (_, Action::Reset) => &[Key::Char('r')],
        (_, Action::Quit) => &[Key::Char('q')],
    }
}

#[cfg(test)]
mod tests {
    use super::shortcut_help_sections;
    use crate::persistence::editor_config::{EditorKeymapPreset, EditorLocale};

    #[test]
    fn preview_help_mentions_tabs_and_body_scopes() {
        let sections = shortcut_help_sections(EditorLocale::EnUs, EditorKeymapPreset::Standard);
        let preview = &sections[2];

        assert_eq!(preview.entries[0].label, "Switch Preview Tabs");
        assert_eq!(
            preview.entries[0].description,
            "While Preview Tabs are focused, cycle between built-in and runtime-backed preview modes."
        );
        assert_eq!(preview.entries[1].label, "Capture Preview Body");
        assert_eq!(
            preview.entries[1].description,
            "While Preview Body is focused, enter the interactive preview session and send keys directly to the child process."
        );
    }
}
