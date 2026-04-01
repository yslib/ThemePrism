use crate::app::controls::ControlId;
use crate::app::effect::Effect;
use crate::app::state::{AppState, SourcePickerState, TextInputState, TextInputTarget};
use crate::domain::color::Color;
use crate::domain::params::ParamKey;
use crate::domain::rules::{Rule, SourceOption, available_source_options};
use crate::domain::tokens::TokenRole;
use crate::export::ExportFormat;
use crate::i18n::UiText;

use super::{config, cycle_index, inspector, modals, navigation, tr, tr1, tr2};

pub(super) fn open_text_input(state: &mut AppState, target: TextInputTarget) {
    let buffer = default_input_buffer(state, target);
    modals::close_shortcut_help_surface(state);
    state.ui.text_input = Some(TextInputState { target, buffer });
    modals::push_modal_owner(
        state,
        crate::app::interaction::SurfaceId::NumericEditorSurface,
    );
    navigation::focus_surface(
        state,
        crate::app::interaction::SurfaceId::NumericEditorSurface,
    );
    state.ui.status = match target {
        TextInputTarget::Control(control) if control.supports_numeric_editor() => tr1(
            state,
            UiText::StatusEditingNumeric,
            "field",
            input_target_label(state, target),
        ),
        _ => tr1(
            state,
            UiText::StatusEditingText,
            "field",
            input_target_label(state, target),
        ),
    };
}

pub(super) fn open_source_picker(state: &mut AppState, control: ControlId) {
    let role = inspector::role_for_reference_control(control)
        .expect("control should be reference control");
    let current = state
        .domain
        .rules
        .get(role)
        .and_then(|rule| inspector::current_source_for_control(control, rule));
    let options = available_source_options(role);
    let selected = current
        .as_ref()
        .and_then(|source| options.iter().position(|option| option.source == *source))
        .unwrap_or_default();

    modals::close_shortcut_help_surface(state);
    state.ui.source_picker = Some(SourcePickerState {
        control,
        filter: String::new(),
        selected,
    });
    modals::push_modal_owner(state, crate::app::interaction::SurfaceId::SourcePicker);
    navigation::focus_surface(state, crate::app::interaction::SurfaceId::SourcePicker);
    state.ui.status = tr1(
        state,
        UiText::StatusSelectingSource,
        "control",
        control.label(),
    );
}

pub(super) fn adjust_active_numeric_input(state: &mut AppState, delta: i32) {
    let Some(target) = state.ui.text_input.as_ref().map(|input| input.target) else {
        return;
    };
    let TextInputTarget::Control(control) = target else {
        return;
    };
    if !control.supports_numeric_editor() {
        return;
    }

    inspector::adjust_control_by_step(state, control, delta);
    let buffer = default_input_buffer(state, target);
    if let Some(input) = &mut state.ui.text_input {
        input.buffer = buffer;
    }
}

pub(super) fn append_text_input(state: &mut AppState, ch: char) {
    let label = state
        .ui
        .text_input
        .as_ref()
        .map(|input| input_target_label(state, input.target))
        .unwrap_or_else(|| "input field".to_string());

    if let Some(input) = &mut state.ui.text_input {
        if accepts_input_char(input.target, ch, &input.buffer) {
            let ch = match input.target {
                TextInputTarget::Control(ControlId::FixedColor(_)) => ch.to_ascii_uppercase(),
                _ => ch,
            };
            input.buffer.push(ch);
        } else {
            state.ui.status = tr2(
                state,
                UiText::StatusInvalidCharacter,
                "char",
                ch,
                "field",
                label,
            );
        }
    }
}

pub(super) fn backspace_text_input(state: &mut AppState) {
    if let Some(input) = &mut state.ui.text_input {
        input.buffer.pop();
    }
}

pub(super) fn clear_text_input(state: &mut AppState) {
    if let Some(input) = &mut state.ui.text_input {
        input.buffer.clear();
    }
}

pub(super) fn commit_text_input(state: &mut AppState) -> Vec<Effect> {
    let Some(input) = state.ui.text_input.clone() else {
        return Vec::new();
    };

    let result = match input.target {
        TextInputTarget::Control(control) => match control {
            ControlId::Param(key) => apply_param_input(state, key, &input.buffer),
            ControlId::MixRatio(role) => apply_mix_ratio_input(state, role, &input.buffer),
            ControlId::AdjustAmount(role) => apply_adjust_amount_input(state, role, &input.buffer),
            ControlId::FixedColor(role) => apply_fixed_color_input(state, role, &input.buffer),
            _ => Err(tr1(
                state,
                UiText::ErrorControlNoTextInput,
                "control",
                control.label(),
            )),
        },
        TextInputTarget::Config(field) => config::apply_config_input(state, field, &input.buffer),
    };

    match result {
        Ok(status) => {
            modals::close_text_input_surface(state);
            state.ui.status = status;
            config::effects_for_text_target(state, input.target)
        }
        Err(err) => {
            state.ui.status = err;
            Vec::new()
        }
    }
}

pub(super) fn move_source_picker_selection(state: &mut AppState, delta: i32) {
    let len = state
        .ui
        .source_picker
        .as_ref()
        .map(|picker| filtered_source_options(picker).len())
        .unwrap_or_default();
    if len == 0 {
        return;
    }

    if let Some(picker) = &mut state.ui.source_picker {
        picker.selected = cycle_index(picker.selected.min(len - 1), len, delta);
    }
}

pub(super) fn apply_source_picker_selection(state: &mut AppState) {
    let Some(picker) = state.ui.source_picker.clone() else {
        return;
    };

    let options = filtered_source_options(&picker);
    if options.is_empty() {
        state.ui.status = tr(state, UiText::ErrorNoSourcesMatch);
        return;
    }

    let selected = options[picker.selected.min(options.len() - 1)].clone();
    let source_label = selected.source.label();
    let result = inspector::try_mutate_rules(state, |rules| {
        let role = inspector::role_for_reference_control(picker.control)
            .expect("picker control should be reference");
        let rule = rules
            .get_mut(role)
            .expect("selected token should have a rule");
        inspector::apply_source_to_control(picker.control, rule, selected.source.clone());
    });

    match result {
        Ok(()) => {
            modals::close_source_picker_surface(state);
            state.ui.status = tr2(
                state,
                UiText::StatusSourceApplied,
                "control",
                picker.control.label(),
                "source",
                source_label,
            );
        }
        Err(err) => {
            state.ui.status = tr1(state, UiText::StatusSourceChangeRejected, "error", err);
        }
    }
}

pub fn filtered_source_options(picker: &SourcePickerState) -> Vec<SourceOption> {
    let filter = picker.filter.trim().to_ascii_lowercase();
    let role = inspector::role_for_reference_control(picker.control)
        .expect("picker control should be reference");
    available_source_options(role)
        .into_iter()
        .filter(|option| {
            filter.is_empty()
                || option.source.label().to_ascii_lowercase().contains(&filter)
                || option.group.label().to_ascii_lowercase().contains(&filter)
        })
        .collect()
}

pub fn default_input_buffer(state: &AppState, target: TextInputTarget) -> String {
    match target {
        TextInputTarget::Control(control) => match control {
            ControlId::Param(key) => match key {
                ParamKey::BackgroundHue | ParamKey::AccentHue => {
                    format!("{:.1}", key.get(&state.domain.params))
                }
                _ => format!("{:.0}", key.get(&state.domain.params) * 100.0),
            },
            ControlId::MixRatio(role) => match state.domain.rules.get(role) {
                Some(Rule::Mix { ratio, .. }) => format!("{:.0}", ratio * 100.0),
                _ => String::new(),
            },
            ControlId::AdjustAmount(role) => match state.domain.rules.get(role) {
                Some(Rule::Adjust { amount, .. }) => format!("{:.0}", amount * 100.0),
                _ => String::new(),
            },
            ControlId::FixedColor(role) => match state.domain.rules.get(role) {
                Some(Rule::Fixed { color }) => color.to_hex(),
                _ => String::new(),
            },
            _ => String::new(),
        },
        TextInputTarget::Config(field) => match field {
            crate::app::state::ConfigFieldId::ProjectName => state.project.name.clone(),
            crate::app::state::ConfigFieldId::EditorProjectPath => {
                state.editor.project_path.display().to_string()
            }
            crate::app::state::ConfigFieldId::EditorAutoLoadProject
            | crate::app::state::ConfigFieldId::EditorAutoSaveOnExport
            | crate::app::state::ConfigFieldId::EditorStartupFocus
            | crate::app::state::ConfigFieldId::EditorKeymapPreset
            | crate::app::state::ConfigFieldId::EditorLocale => String::new(),
            crate::app::state::ConfigFieldId::ExportOutputPath(index) => state
                .project
                .export_profiles
                .get(index)
                .map(|profile| profile.output_path.display().to_string())
                .unwrap_or_default(),
            crate::app::state::ConfigFieldId::ExportTemplatePath(index) => state
                .project
                .export_profiles
                .get(index)
                .and_then(|profile| match &profile.format {
                    ExportFormat::Template { template_path } => {
                        Some(template_path.display().to_string())
                    }
                    ExportFormat::Alacritty => None,
                })
                .unwrap_or_default(),
            crate::app::state::ConfigFieldId::ExportEnabled(_) => String::new(),
        },
    }
}

pub(super) fn input_target_label(state: &AppState, target: TextInputTarget) -> String {
    match target {
        TextInputTarget::Control(control) => control.label().to_string(),
        TextInputTarget::Config(field) => match field {
            crate::app::state::ConfigFieldId::ProjectName => {
                tr(state, UiText::FieldProjectNameLower)
            }
            crate::app::state::ConfigFieldId::ExportEnabled(index) => {
                tr1(state, UiText::FieldExportTarget, "index", index + 1)
            }
            crate::app::state::ConfigFieldId::ExportOutputPath(index) => {
                tr1(state, UiText::FieldExportOutputPath, "index", index + 1)
            }
            crate::app::state::ConfigFieldId::ExportTemplatePath(index) => {
                tr1(state, UiText::FieldExportTemplatePath, "index", index + 1)
            }
            crate::app::state::ConfigFieldId::EditorProjectPath => {
                tr(state, UiText::FieldProjectFilePath)
            }
            crate::app::state::ConfigFieldId::EditorAutoLoadProject => {
                tr(state, UiText::FieldAutoLoadProject)
            }
            crate::app::state::ConfigFieldId::EditorAutoSaveOnExport => {
                tr(state, UiText::FieldAutoSaveProject)
            }
            crate::app::state::ConfigFieldId::EditorStartupFocus => {
                tr(state, UiText::FieldStartupFocusLower)
            }
            crate::app::state::ConfigFieldId::EditorKeymapPreset => {
                tr(state, UiText::FieldKeymapPresetLower)
            }
            crate::app::state::ConfigFieldId::EditorLocale => tr(state, UiText::FieldLanguageLower),
        },
    }
}

fn apply_param_input(state: &mut AppState, key: ParamKey, buffer: &str) -> Result<String, String> {
    let value = parse_param_input(state, key, buffer)?;
    let mut params = state.domain.params.clone();
    key.set(&mut params, value);
    state.domain.params = params;
    state.recompute().map_err(|err| err.to_string())?;
    Ok(tr2(
        state,
        UiText::StatusUpdatedFieldValue,
        "field",
        key.label(),
        "value",
        key.format_value(&state.domain.params),
    ))
}

fn apply_mix_ratio_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let ratio = parse_fraction_input(state, buffer)?;
    inspector::try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio: current, .. }) = rules.get_mut(role) {
            *current = ratio;
        }
    })?;
    Ok(tr2(
        state,
        UiText::StatusBlendUpdated,
        "token",
        role.label(),
        "value",
        format!("{:>3.0}%", ratio * 100.0),
    ))
}

fn apply_adjust_amount_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let amount = parse_fraction_input(state, buffer)?;
    inspector::try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust {
            amount: current, ..
        }) = rules.get_mut(role)
        {
            *current = amount;
        }
    })?;
    Ok(tr2(
        state,
        UiText::StatusAmountUpdated,
        "token",
        role.label(),
        "value",
        format!("{:>3.0}%", amount * 100.0),
    ))
}

fn apply_fixed_color_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let color = Color::from_hex(buffer).map_err(|_| tr(state, UiText::ErrorInvalidHexColor))?;
    inspector::try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color: current }) = rules.get_mut(role) {
            *current = color;
        }
    })?;
    Ok(tr2(
        state,
        UiText::StatusFixedColorUpdated,
        "token",
        role.label(),
        "value",
        color.to_hex(),
    ))
}

fn parse_param_input(state: &AppState, key: ParamKey, buffer: &str) -> Result<f32, String> {
    match key {
        ParamKey::BackgroundHue | ParamKey::AccentHue => parse_float_input(state, buffer),
        _ => parse_fraction_input(state, buffer),
    }
}

fn parse_float_input(state: &AppState, buffer: &str) -> Result<f32, String> {
    let trimmed = buffer.trim().trim_end_matches('%').trim();
    if trimmed.is_empty() {
        return Err(tr(state, UiText::ErrorInputEmpty));
    }
    trimmed
        .parse::<f32>()
        .map_err(|_| tr1(state, UiText::ErrorInvalidNumber, "value", buffer))
}

fn parse_fraction_input(state: &AppState, buffer: &str) -> Result<f32, String> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Err(tr(state, UiText::ErrorInputEmpty));
    }

    let is_percent = trimmed.ends_with('%');
    let number = trimmed
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .map_err(|_| tr1(state, UiText::ErrorInvalidNumber, "value", buffer))?;

    let value = if is_percent || number > 1.0 {
        number / 100.0
    } else {
        number
    };

    Ok(value.clamp(0.0, 1.0))
}

fn accepts_input_char(target: TextInputTarget, ch: char, existing: &str) -> bool {
    match target {
        TextInputTarget::Control(control) => match control {
            ControlId::Param(_) | ControlId::MixRatio(_) | ControlId::AdjustAmount(_) => {
                ch.is_ascii_digit() || ch == '.' || ch == '%'
            }
            ControlId::FixedColor(_) => {
                let hex_len = existing.trim_start_matches('#').len();
                (ch.is_ascii_hexdigit() && hex_len < 8)
                    || (ch == '#' && !existing.contains('#') && existing.is_empty())
            }
            _ => false,
        },
        TextInputTarget::Config(_) => !ch.is_control(),
    }
}
