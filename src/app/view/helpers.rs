use crate::app::controls::ControlId;
use crate::app::state::{AppState, ConfigFieldId, TextInputTarget};
use crate::app::update::default_input_buffer;
use crate::domain::rules::Rule;
use crate::i18n::{self, UiText};
use crate::persistence::editor_config::EditorLocale;

pub(crate) fn input_preview(buffer: &str) -> String {
    if buffer.is_empty() {
        "_".to_string()
    } else {
        format!("{buffer}_")
    }
}

pub(crate) fn display_text_for_control(state: &AppState, control: ControlId) -> String {
    if let Some(input) = &state.ui.text_input {
        if input.target == TextInputTarget::Control(control) {
            return input_preview(&input.buffer);
        }
    }

    match control {
        ControlId::Param(key) => key.format_value(&state.domain.params),
        ControlId::MixRatio(role) => match state.domain.rules.get(role) {
            Some(Rule::Mix { ratio, .. }) => format!("{:>3.0}%", ratio * 100.0),
            _ => String::new(),
        },
        ControlId::AdjustAmount(role) => match state.domain.rules.get(role) {
            Some(Rule::Adjust { amount, .. }) => format!("{:>3.0}%", amount * 100.0),
            _ => String::new(),
        },
        ControlId::FixedColor(role) => match state.domain.rules.get(role) {
            Some(Rule::Fixed { color }) => color.to_hex(),
            _ => String::new(),
        },
        _ => default_input_buffer(state, TextInputTarget::Control(control)),
    }
}

pub(crate) fn config_field_label(locale: EditorLocale, field: ConfigFieldId) -> String {
    i18n::config_field_label(locale, field)
}

pub(crate) fn config_field_value(state: &AppState, field: ConfigFieldId) -> String {
    if let Some(input) = &state.ui.text_input {
        if input.target == TextInputTarget::Config(field) {
            return input_preview(&input.buffer);
        }
    }

    let locale = state.locale();
    match field {
        ConfigFieldId::ProjectName => state.project.name.clone(),
        ConfigFieldId::EditorProjectPath => state.editor.project_path.display().to_string(),
        ConfigFieldId::EditorKeymapPreset => {
            i18n::keymap_preset_label(locale, state.editor.keymap_preset)
        }
        ConfigFieldId::EditorLocale => i18n::locale_label(locale, state.editor.locale),
        ConfigFieldId::ExportEnabled(index) => state
            .project
            .export_profiles
            .get(index)
            .map(|profile| profile.summary_label())
            .unwrap_or_else(|| i18n::text(locale, UiText::ConfigValueMissingExportTarget)),
        ConfigFieldId::ExportOutputPath(index) => state
            .project
            .export_profiles
            .get(index)
            .map(|profile| profile.output_path.display().to_string())
            .unwrap_or_else(|| "-".to_string()),
        ConfigFieldId::ExportTemplatePath(index) => state
            .project
            .export_profiles
            .get(index)
            .map(|profile| profile.template_path().display().to_string())
            .unwrap_or_else(|| "-".to_string()),
    }
}

pub(crate) fn export_status_summary(state: &AppState) -> String {
    let locale = state.locale();
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .map(|profile| profile.name.as_str())
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => i18n::text(locale, UiText::ExportStatusNoneEnabled),
        [name] => i18n::format1(locale, UiText::ExportStatusOneEnabled, "name", name),
        names if names.len() <= 3 => {
            i18n::format1(locale, UiText::ExportStatusNamed, "names", names.join(", "))
        }
        names => i18n::format1(locale, UiText::ExportStatusCount, "count", names.len()),
    }
}
