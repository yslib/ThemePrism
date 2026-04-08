use std::path::PathBuf;

use crate::app::effect::{EditorConfigData, Effect};
use crate::app::state::{AppState, ConfigFieldId, TextInputTarget};
use crate::i18n::{self, UiText};
use crate::persistence::editor_config::{EditorKeymapPreset, EditorLocale};

use super::{cycle_index, text_input, tr, tr1};

pub fn config_fields(state: &AppState) -> Vec<ConfigFieldId> {
    let mut fields = vec![ConfigFieldId::ProjectName];
    for (index, _) in state.project.export_profiles.iter().enumerate() {
        fields.push(ConfigFieldId::ExportEnabled(index));
        fields.push(ConfigFieldId::ExportOutputPath(index));
        fields.push(ConfigFieldId::ExportTemplatePath(index));
    }
    fields.push(ConfigFieldId::EditorProjectPath);
    fields.push(ConfigFieldId::EditorKeymapPreset);
    fields.push(ConfigFieldId::EditorLocale);
    fields
}

pub(super) fn save_editor_config_effects(state: &AppState) -> Vec<Effect> {
    vec![Effect::SaveEditorConfig {
        data: EditorConfigData {
            config: state.editor_config(),
        },
    }]
}

pub(super) fn set_editor_project_path(state: &mut AppState, path: PathBuf) -> Vec<Effect> {
    state.editor.project_path = path;
    state.ui.status = tr1(
        state,
        UiText::StatusProjectFilePathUpdated,
        "path",
        state.editor.project_path.display(),
    );
    save_editor_config_effects(state)
}

pub(super) fn set_editor_keymap_preset(
    state: &mut AppState,
    preset: EditorKeymapPreset,
) -> Vec<Effect> {
    state.editor.keymap_preset = preset;
    state.ui.status = tr1(
        state,
        UiText::StatusKeymapUpdated,
        "preset",
        i18n::keymap_preset_label(state.locale(), preset),
    );
    save_editor_config_effects(state)
}

pub(super) fn set_editor_locale(state: &mut AppState, locale: EditorLocale) -> Vec<Effect> {
    state.editor.locale = locale;
    state.ui.status = tr1(
        state,
        UiText::StatusLanguageUpdated,
        "locale",
        i18n::locale_label(state.locale(), locale),
    );
    save_editor_config_effects(state)
}

pub(super) fn move_config_selection(state: &mut AppState, delta: i32) {
    let len = config_fields(state).len();
    if len == 0 {
        return;
    }

    if let Some(modal) = &mut state.ui.config_modal {
        modal.selected_field = cycle_index(modal.selected_field.min(len - 1), len, delta);
    }
}

pub(super) fn activate_config_field(state: &mut AppState) -> Vec<Effect> {
    let field = if let Some(modal) = &state.ui.config_modal {
        let fields = config_fields(state);
        match fields
            .get(modal.selected_field.min(fields.len().saturating_sub(1)))
            .copied()
        {
            Some(field) => field,
            None => return Vec::new(),
        }
    } else {
        match state.active_config_field() {
            Some(field) => field,
            None => return Vec::new(),
        }
    };

    activate_config_field_by_id(state, field)
}

pub(super) fn apply_config_input(
    state: &mut AppState,
    field: ConfigFieldId,
    buffer: &str,
) -> Result<String, String> {
    let value = buffer.trim();
    if value.is_empty() {
        return Err(tr(state, UiText::ErrorInputEmpty));
    }

    let locale = state.locale();
    match field {
        ConfigFieldId::ProjectName => {
            state.project.name = value.to_string();
            Ok(tr1(
                state,
                UiText::StatusProjectNameUpdated,
                "name",
                &state.project.name,
            ))
        }
        ConfigFieldId::EditorProjectPath => {
            state.editor.project_path = value.into();
            Ok(tr1(
                state,
                UiText::StatusProjectFilePathUpdated,
                "path",
                state.editor.project_path.display(),
            ))
        }
        ConfigFieldId::EditorKeymapPreset | ConfigFieldId::EditorLocale => {
            Err(tr(state, UiText::ErrorUseToggleChoicePreference))
        }
        ConfigFieldId::ExportOutputPath(index) => {
            let profile = state
                .project
                .export_profiles
                .get_mut(index)
                .ok_or_else(|| {
                    i18n::format1(
                        locale,
                        UiText::StatusMissingExportTarget,
                        "index",
                        index + 1,
                    )
                })?;
            profile.output_path = value.into();
            Ok(i18n::format2(
                locale,
                UiText::StatusExportOutputUpdated,
                "name",
                &profile.name,
                "path",
                profile.output_path.display(),
            ))
        }
        ConfigFieldId::ExportTemplatePath(index) => {
            let profile = state
                .project
                .export_profiles
                .get_mut(index)
                .ok_or_else(|| {
                    i18n::format1(
                        locale,
                        UiText::StatusMissingExportTarget,
                        "index",
                        index + 1,
                    )
                })?;
            profile.set_template_path(value.into());
            Ok(i18n::format2(
                locale,
                UiText::StatusExportTemplateUpdated,
                "name",
                &profile.name,
                "path",
                profile.configured_template_path().display(),
            ))
        }
        ConfigFieldId::ExportEnabled(index) => Err(tr1(
            state,
            UiText::ErrorToggleExportTarget,
            "index",
            index + 1,
        )),
    }
}

pub(super) fn effects_for_text_target(state: &AppState, target: TextInputTarget) -> Vec<Effect> {
    match target {
        TextInputTarget::Config(
            ConfigFieldId::EditorProjectPath
            | ConfigFieldId::EditorKeymapPreset
            | ConfigFieldId::EditorLocale,
        ) => save_editor_config_effects(state),
        _ => Vec::new(),
    }
}

fn activate_config_field_by_id(state: &mut AppState, field: ConfigFieldId) -> Vec<Effect> {
    if field.supports_text_input() {
        text_input::open_text_input(state, TextInputTarget::Config(field));
        return Vec::new();
    }

    match field {
        ConfigFieldId::ExportEnabled(index) => {
            let locale = state.locale();
            if let Some(profile) = state.project.export_profiles.get_mut(index) {
                profile.enabled = !profile.enabled;
                state.ui.status = if profile.enabled {
                    i18n::format1(
                        locale,
                        UiText::StatusExportTargetEnabled,
                        "name",
                        &profile.name,
                    )
                } else {
                    i18n::format1(
                        locale,
                        UiText::StatusExportTargetDisabled,
                        "name",
                        &profile.name,
                    )
                };
            } else {
                state.ui.status = i18n::format1(
                    locale,
                    UiText::StatusMissingExportTarget,
                    "index",
                    index + 1,
                );
            }
        }
        ConfigFieldId::EditorKeymapPreset => {
            return set_editor_keymap_preset(state, state.editor.keymap_preset.next());
        }
        ConfigFieldId::EditorLocale => {
            return set_editor_locale(state, state.editor.locale.next());
        }
        ConfigFieldId::ProjectName
        | ConfigFieldId::ExportOutputPath(_)
        | ConfigFieldId::ExportTemplatePath(_)
        | ConfigFieldId::EditorProjectPath => {}
    }

    Vec::new()
}
