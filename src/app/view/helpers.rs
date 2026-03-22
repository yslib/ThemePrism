use crate::app::controls::ControlId;
use crate::app::state::{AppState, ConfigFieldId, TextInputTarget};
use crate::app::update::default_input_buffer;
use crate::domain::rules::Rule;

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

pub(crate) fn config_field_label(field: ConfigFieldId) -> String {
    match field {
        ConfigFieldId::ProjectName => "Project Name".to_string(),
        ConfigFieldId::ExportEnabled(index) => format!("Target {}", index + 1),
        ConfigFieldId::ExportOutputPath(index) => format!("Output {}", index + 1),
        ConfigFieldId::ExportTemplatePath(index) => format!("Template {}", index + 1),
        ConfigFieldId::EditorProjectPath => "Project File".to_string(),
        ConfigFieldId::EditorAutoLoadProject => "Auto Load".to_string(),
        ConfigFieldId::EditorAutoSaveOnExport => "Auto Save".to_string(),
        ConfigFieldId::EditorStartupFocus => "Startup Focus".to_string(),
    }
}

pub(crate) fn config_field_value(state: &AppState, field: ConfigFieldId) -> String {
    if let Some(input) = &state.ui.text_input {
        if input.target == TextInputTarget::Config(field) {
            return input_preview(&input.buffer);
        }
    }

    match field {
        ConfigFieldId::ProjectName => state.project.name.clone(),
        ConfigFieldId::EditorProjectPath => state.editor.project_path.display().to_string(),
        ConfigFieldId::EditorAutoLoadProject => {
            if state.editor.auto_load_project_on_startup {
                "[x] Load project on startup".to_string()
            } else {
                "[ ] Load project on startup".to_string()
            }
        }
        ConfigFieldId::EditorAutoSaveOnExport => {
            if state.editor.auto_save_project_on_export {
                "[x] Save project before export".to_string()
            } else {
                "[ ] Save project before export".to_string()
            }
        }
        ConfigFieldId::EditorStartupFocus => state.editor.startup_focus.label().to_string(),
        ConfigFieldId::ExportEnabled(index) => state
            .project
            .export_profiles
            .get(index)
            .map(|profile| profile.summary_label())
            .unwrap_or_else(|| "Missing export target".to_string()),
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
            .and_then(|profile| match &profile.format {
                crate::export::ExportFormat::Template { template_path } => {
                    Some(template_path.display().to_string())
                }
                crate::export::ExportFormat::Alacritty => None,
            })
            .unwrap_or_else(|| "-".to_string()),
    }
}

pub(crate) fn export_targets_summary(state: &AppState) -> String {
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .map(|profile| profile.name.as_str())
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => "None enabled".to_string(),
        [name] => format!("1 enabled: {name}"),
        names if names.len() <= 3 => format!("{} enabled: {}", names.len(), names.join(", ")),
        names => format!("{} enabled", names.len()),
    }
}

pub(crate) fn export_outputs_summary(state: &AppState) -> String {
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => "No export targets enabled".to_string(),
        [profile] => profile.output_path.display().to_string(),
        profiles => format!("{} output targets", profiles.len()),
    }
}

pub(crate) fn export_status_summary(state: &AppState) -> String {
    let enabled = state
        .project
        .export_profiles
        .iter()
        .filter(|profile| profile.enabled)
        .map(|profile| profile.name.as_str())
        .collect::<Vec<_>>();

    match enabled.as_slice() {
        [] => "none enabled".to_string(),
        [name] => format!("{name} enabled"),
        names if names.len() <= 3 => format!("{} enabled", names.join(", ")),
        names => format!("{} targets enabled", names.len()),
    }
}
