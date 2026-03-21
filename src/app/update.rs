use crate::app::controls::{ControlId, ReferenceField};
use crate::app::effect::{EditorConfigData, Effect, ProjectData};
use crate::app::intent::Intent;
use crate::app::state::{
    AppState, ConfigFieldId, ConfigModalState, FocusPane, SourcePickerState, TextInputState,
    TextInputTarget,
};
use crate::domain::color::Color;
use crate::domain::params::{ParamKey, ThemeParams};
use crate::domain::rules::{
    AdjustOp, Rule, RuleKind, RuleSet, SourceOption, SourceRef, available_source_options,
    available_sources, starter_rule,
};
use crate::domain::tokens::TokenRole;
use crate::export::{ExportFormat, default_export_profiles};

pub fn update(state: &mut AppState, intent: Intent) -> Vec<Effect> {
    match intent {
        Intent::QuitRequested => {
            state.ui.should_quit = true;
            Vec::new()
        }
        Intent::MoveFocus(delta) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
            {
                return Vec::new();
            }
            state.ui.focus = if delta >= 0 {
                state.ui.focus.next()
            } else {
                state.ui.focus.previous()
            };
            Vec::new()
        }
        Intent::MoveSelection(delta) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
            {
                return Vec::new();
            }
            move_selection(state, delta);
            Vec::new()
        }
        Intent::SelectToken(index) => {
            select_token(state, index);
            Vec::new()
        }
        Intent::AdjustControlByStep(control, delta) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
            {
                return Vec::new();
            }
            adjust_control_by_step(state, control, delta);
            Vec::new()
        }
        Intent::ActivateControl(control) => {
            if state.ui.source_picker.is_some()
                || state.ui.text_input.is_some()
                || state.ui.config_modal.is_some()
            {
                return Vec::new();
            }
            activate_control(state, control);
            Vec::new()
        }
        Intent::SetParamValue(key, value) => {
            set_param_value(state, key, value);
            Vec::new()
        }
        Intent::SetRuleKind(role, kind) => {
            set_rule_kind_for_role(state, role, kind);
            Vec::new()
        }
        Intent::SetReferenceSource(control, source) => {
            set_reference_source(state, control, source);
            Vec::new()
        }
        Intent::SetMixRatio(role, ratio) => {
            set_mix_ratio(state, role, ratio);
            Vec::new()
        }
        Intent::SetAdjustOp(role, op) => {
            set_adjust_op(state, role, op);
            Vec::new()
        }
        Intent::SetAdjustAmount(role, amount) => {
            set_adjust_amount(state, role, amount);
            Vec::new()
        }
        Intent::SetFixedColor(role, color) => {
            set_fixed_color(state, role, color);
            Vec::new()
        }
        Intent::SetProjectName(name) => set_project_name(state, name),
        Intent::SetExportEnabled(index, enabled) => set_export_enabled(state, index, enabled),
        Intent::SetExportOutputPath(index, path) => set_export_output_path(state, index, path),
        Intent::SetExportTemplatePath(index, path) => set_export_template_path(state, index, path),
        Intent::SetEditorProjectPath(path) => set_editor_project_path(state, path),
        Intent::SetEditorAutoLoadProject(enabled) => set_editor_auto_load_project(state, enabled),
        Intent::SetEditorAutoSaveOnExport(enabled) => {
            set_editor_auto_save_on_export(state, enabled)
        }
        Intent::SetEditorStartupFocus(focus) => set_editor_startup_focus(state, focus),
        Intent::AppendTextInput(ch) => {
            append_text_input(state, ch);
            Vec::new()
        }
        Intent::BackspaceTextInput => {
            if let Some(input) = &mut state.ui.text_input {
                input.buffer.pop();
            }
            Vec::new()
        }
        Intent::ClearTextInput => {
            if let Some(input) = &mut state.ui.text_input {
                input.buffer.clear();
            }
            Vec::new()
        }
        Intent::CommitTextInput => commit_text_input(state),
        Intent::CancelTextInput => {
            state.ui.text_input = None;
            state.ui.status = "Input cancelled.".to_string();
            Vec::new()
        }
        Intent::AppendSourcePickerFilter(ch) => {
            if let Some(picker) = &mut state.ui.source_picker {
                picker.filter.push(ch);
                picker.selected = 0;
            }
            Vec::new()
        }
        Intent::BackspaceSourcePickerFilter => {
            if let Some(picker) = &mut state.ui.source_picker {
                picker.filter.pop();
                picker.selected = 0;
            }
            Vec::new()
        }
        Intent::ClearSourcePickerFilter => {
            if let Some(picker) = &mut state.ui.source_picker {
                picker.filter.clear();
                picker.selected = 0;
            }
            Vec::new()
        }
        Intent::MoveSourcePickerSelection(delta) => {
            move_source_picker_selection(state, delta);
            Vec::new()
        }
        Intent::ApplySourcePickerSelection => {
            apply_source_picker_selection(state);
            Vec::new()
        }
        Intent::CloseSourcePicker => {
            state.ui.source_picker = None;
            state.ui.status = "Source picker closed.".to_string();
            Vec::new()
        }
        Intent::OpenConfigRequested => {
            open_config_modal(state);
            Vec::new()
        }
        Intent::CloseConfigRequested => {
            close_config_modal(state);
            Vec::new()
        }
        Intent::MoveConfigSelection(delta) => {
            move_config_selection(state, delta);
            Vec::new()
        }
        Intent::ActivateConfigField => activate_config_field(state),
        Intent::SaveProjectRequested => vec![save_project_effect(state)],
        Intent::LoadProjectRequested => vec![Effect::LoadProject {
            path: state.editor.project_path.clone(),
        }],
        Intent::ExportThemeRequested => {
            let mut effects = Vec::new();
            if state.editor.auto_save_project_on_export {
                effects.push(save_project_effect(state));
            }
            effects.push(Effect::ExportTheme {
                profiles: state.project.export_profiles.clone(),
                theme: state.domain.resolved.clone(),
            });
            effects
        }
        Intent::ResetRequested => {
            reset_state(state);
            Vec::new()
        }
        Intent::ProjectSaved(result) => {
            state.ui.status = match result {
                Ok(path) => format!("Saved project to {}", path.display()),
                Err(err) => format!("Save failed: {err}"),
            };
            Vec::new()
        }
        Intent::ProjectLoaded(result) => {
            match result {
                Ok(project) => match state.apply_project_data(project) {
                    Ok(()) => {
                        state.ui.status = format!(
                            "Loaded project from {}",
                            state.editor.project_path.display()
                        );
                    }
                    Err(err) => {
                        state.ui.status = format!("Load recompute failed: {err}");
                    }
                },
                Err(err) => {
                    state.ui.status = format!("Load failed: {err}");
                }
            }
            Vec::new()
        }
        Intent::ThemeExported(result) => {
            state.ui.status = match result {
                Ok(artifacts) if artifacts.is_empty() => {
                    "Export completed with no output.".to_string()
                }
                Ok(artifacts) if artifacts.len() == 1 => format!(
                    "Exported {} to {}",
                    artifacts[0].profile_name,
                    artifacts[0].output_path.display()
                ),
                Ok(artifacts) => format!("Exported {} targets.", artifacts.len()),
                Err(err) => format!("Export failed: {err}"),
            };
            Vec::new()
        }
        Intent::EditorConfigSaved(result) => {
            if let Err(err) = result {
                state.ui.status = format!("Editor config save failed: {err}");
            }
            Vec::new()
        }
    }
}

fn move_selection(state: &mut AppState, delta: i32) {
    match state.ui.focus {
        FocusPane::Tokens => {
            state.ui.selected_token =
                cycle_index(state.ui.selected_token, TokenRole::ALL.len(), delta);
            state.ui.inspector_field = state
                .ui
                .inspector_field
                .min(state.inspector_field_count().saturating_sub(1));
            state.ui.source_picker = None;
            state.ui.status = format!("Selected token {}", state.selected_role().label());
        }
        FocusPane::Params => {
            state.ui.selected_param =
                cycle_index(state.ui.selected_param, ParamKey::ALL.len(), delta);
            state.ui.source_picker = None;
            state.ui.status = format!("Selected param {}", state.selected_param_key().label());
        }
        FocusPane::Inspector => {
            state.ui.inspector_field = cycle_index(
                state.ui.inspector_field,
                state.inspector_field_count(),
                delta,
            );
            state.ui.source_picker = None;
        }
    }
}

fn select_token(state: &mut AppState, index: usize) {
    state.ui.selected_token = index.min(TokenRole::ALL.len().saturating_sub(1));
    state.ui.inspector_field = state
        .ui
        .inspector_field
        .min(state.inspector_field_count().saturating_sub(1));
    state.ui.source_picker = None;
    state.ui.text_input = None;
    state.ui.status = format!("Selected token {}", state.selected_role().label());
}

fn adjust_control_by_step(state: &mut AppState, control: ControlId, delta: i32) {
    match control {
        ControlId::Param(key) => adjust_param_by_step(state, key, delta),
        ControlId::RuleKind(role) => cycle_rule_kind_for_role(state, role, delta),
        ControlId::Reference(role, _) => cycle_reference_source(state, control, role, delta),
        ControlId::MixRatio(role) => set_mix_ratio_by_step(state, role, delta),
        ControlId::AdjustOp(role) => cycle_adjust_op_for_role(state, role, delta),
        ControlId::AdjustAmount(role) => set_adjust_amount_by_step(state, role, delta),
        ControlId::FixedColor(role) => cycle_fixed_color_for_role(state, role, delta),
    }
}

fn activate_control(state: &mut AppState, control: ControlId) {
    if control.supports_source_picker() {
        open_source_picker(state, control);
    } else if control.supports_text_input() {
        open_text_input(state, TextInputTarget::Control(control));
    } else {
        state.ui.status = format!("{} does not support activation.", control.label());
    }
}

fn set_param_value(state: &mut AppState, key: ParamKey, value: f32) {
    let mut params = state.domain.params.clone();
    key.set(&mut params, value);
    state.domain.params = params;
    if let Err(err) = state.recompute() {
        state.ui.status = format!("Failed to update {}: {err}", key.label());
    } else {
        state.ui.status = format!(
            "{} -> {}",
            key.label(),
            key.format_value(&state.domain.params)
        );
    }
}

fn adjust_param_by_step(state: &mut AppState, key: ParamKey, delta: i32) {
    let mut params = state.domain.params.clone();
    key.adjust(&mut params, delta);
    state.domain.params = params;
    if let Err(err) = state.recompute() {
        state.ui.status = format!("Failed to update {}: {err}", key.label());
    } else {
        state.ui.status = format!(
            "{} -> {}",
            key.label(),
            key.format_value(&state.domain.params)
        );
    }
}

fn set_rule_kind_for_role(state: &mut AppState, role: TokenRole, kind: RuleKind) {
    let current_color = state
        .domain
        .resolved
        .token(role)
        .expect("resolved theme should contain every token");
    let selection_mix = state.domain.params.selection_mix;
    let result = try_mutate_rules(state, |rules| {
        let rule = rules
            .get_mut(role)
            .expect("selected token should have rule");
        *rule = starter_rule(kind, role, current_color, selection_mix);
    });

    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn cycle_rule_kind_for_role(state: &mut AppState, role: TokenRole, delta: i32) {
    let current_color = state
        .domain
        .resolved
        .token(role)
        .expect("resolved theme should contain every token");
    let selection_mix = state.domain.params.selection_mix;
    let result = try_mutate_rules(state, |rules| {
        let rule = rules
            .get_mut(role)
            .expect("selected token should have rule");
        let current = rule.kind();
        let next = cycle_rule_kind(current, delta);
        *rule = starter_rule(next, role, current_color, selection_mix);
    });

    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn set_reference_source(state: &mut AppState, control: ControlId, source: SourceRef) {
    let result = try_mutate_rules(state, |rules| {
        let role =
            role_for_reference_control(control).expect("control should be reference control");
        let rule = rules
            .get_mut(role)
            .expect("selected token should have a rule");
        apply_source_to_control(control, rule, source.clone());
    });

    state.ui.status = match result {
        Ok(()) => format!("Updated {}", control.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn cycle_reference_source(state: &mut AppState, control: ControlId, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        let rule = rules
            .get_mut(role)
            .expect("selected token should have rule");
        let source = current_source_for_control(control, rule).expect("control should match rule");
        let next = cycle_source(&source, role, delta);
        apply_source_to_control(control, rule, next);
    });

    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn set_mix_ratio(state: &mut AppState, role: TokenRole, ratio: f32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio: current, .. }) = rules.get_mut(role) {
            *current = ratio.clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn set_mix_ratio_by_step(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio, .. }) = rules.get_mut(role) {
            *ratio = (*ratio + delta as f32 * 0.05).clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn set_adjust_op(state: &mut AppState, role: TokenRole, next: AdjustOp) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { op, .. }) = rules.get_mut(role) {
            *op = next;
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn cycle_adjust_op_for_role(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { op, .. }) = rules.get_mut(role) {
            *op = cycle_adjust_op(*op, delta);
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn set_adjust_amount(state: &mut AppState, role: TokenRole, amount: f32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust {
            amount: current, ..
        }) = rules.get_mut(role)
        {
            *current = amount.clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn set_adjust_amount_by_step(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { amount, .. }) = rules.get_mut(role) {
            *amount = (*amount + delta as f32 * 0.02).clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn set_fixed_color(state: &mut AppState, role: TokenRole, next: Color) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color }) = rules.get_mut(role) {
            *color = next;
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn cycle_fixed_color_for_role(state: &mut AppState, role: TokenRole, delta: i32) {
    let options = state.fixed_color_options();
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color }) = rules.get_mut(role) {
            *color = cycle_fixed_color(*color, &options, delta);
        }
    });
    state.ui.status = match result {
        Ok(()) => format!("Updated {}", role.label()),
        Err(err) => format!("Rule change rejected: {err}"),
    };
}

fn open_text_input(state: &mut AppState, target: TextInputTarget) {
    let buffer = default_input_buffer(state, target);
    state.ui.text_input = Some(TextInputState { target, buffer });
    state.ui.status = format!(
        "Editing {}. Press Enter to apply or Esc to cancel.",
        input_target_label(target)
    );
}

fn open_source_picker(state: &mut AppState, control: ControlId) {
    let role = role_for_reference_control(control).expect("control should be reference control");
    let current = state
        .domain
        .rules
        .get(role)
        .and_then(|rule| current_source_for_control(control, rule));
    let options = available_source_options(role);
    let selected = current
        .as_ref()
        .and_then(|source| options.iter().position(|option| option.source == *source))
        .unwrap_or_default();

    state.ui.source_picker = Some(SourcePickerState {
        control,
        filter: String::new(),
        selected,
    });
    state.ui.status = format!("Selecting source for {}. Type to filter.", control.label());
}

fn append_text_input(state: &mut AppState, ch: char) {
    let label = state
        .ui
        .text_input
        .as_ref()
        .map(|input| input_target_label(input.target))
        .unwrap_or_else(|| "input field".to_string());

    if let Some(input) = &mut state.ui.text_input {
        if accepts_input_char(input.target, ch, &input.buffer) {
            let ch = match input.target {
                TextInputTarget::Control(ControlId::FixedColor(_)) => ch.to_ascii_uppercase(),
                _ => ch,
            };
            input.buffer.push(ch);
        } else {
            state.ui.status = format!("Character '{}' is not valid for {}.", ch, label);
        }
    }
}

fn commit_text_input(state: &mut AppState) -> Vec<Effect> {
    let Some(input) = state.ui.text_input.clone() else {
        return Vec::new();
    };

    let result = match input.target {
        TextInputTarget::Control(control) => match control {
            ControlId::Param(key) => apply_param_input(state, key, &input.buffer),
            ControlId::MixRatio(role) => apply_mix_ratio_input(state, role, &input.buffer),
            ControlId::AdjustAmount(role) => apply_adjust_amount_input(state, role, &input.buffer),
            ControlId::FixedColor(role) => apply_fixed_color_input(state, role, &input.buffer),
            _ => Err(format!("{} does not support text input.", control.label())),
        },
        TextInputTarget::Config(field) => apply_config_input(state, field, &input.buffer),
    };

    match result {
        Ok(status) => {
            state.ui.text_input = None;
            state.ui.status = status;
            effects_for_text_target(state, input.target)
        }
        Err(err) => {
            state.ui.status = err;
            Vec::new()
        }
    }
}

fn move_source_picker_selection(state: &mut AppState, delta: i32) {
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

fn apply_source_picker_selection(state: &mut AppState) {
    let Some(picker) = state.ui.source_picker.clone() else {
        return;
    };

    let options = filtered_source_options(&picker);
    if options.is_empty() {
        state.ui.status = "No sources match the current filter.".to_string();
        return;
    }

    let selected = options[picker.selected.min(options.len() - 1)].clone();
    let source_label = selected.source.label();
    let result = try_mutate_rules(state, |rules| {
        let role =
            role_for_reference_control(picker.control).expect("picker control should be reference");
        let rule = rules
            .get_mut(role)
            .expect("selected token should have a rule");
        apply_source_to_control(picker.control, rule, selected.source.clone());
    });

    match result {
        Ok(()) => {
            state.ui.source_picker = None;
            state.ui.status = format!("{} -> {}", picker.control.label(), source_label);
        }
        Err(err) => {
            state.ui.status = format!("Source change rejected: {err}");
        }
    }
}

fn apply_param_input(state: &mut AppState, key: ParamKey, buffer: &str) -> Result<String, String> {
    let value = parse_param_input(key, buffer)?;
    let mut params = state.domain.params.clone();
    key.set(&mut params, value);
    state.domain.params = params;
    state.recompute().map_err(|err| err.to_string())?;
    Ok(format!(
        "{} -> {}",
        key.label(),
        key.format_value(&state.domain.params)
    ))
}

fn apply_mix_ratio_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let ratio = parse_fraction_input(buffer)?;
    try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio: current, .. }) = rules.get_mut(role) {
            *current = ratio;
        }
    })?;
    Ok(format!("{} blend -> {:>3.0}%", role.label(), ratio * 100.0))
}

fn apply_adjust_amount_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let amount = parse_fraction_input(buffer)?;
    try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust {
            amount: current, ..
        }) = rules.get_mut(role)
        {
            *current = amount;
        }
    })?;
    Ok(format!(
        "{} amount -> {:>3.0}%",
        role.label(),
        amount * 100.0
    ))
}

fn apply_fixed_color_input(
    state: &mut AppState,
    role: TokenRole,
    buffer: &str,
) -> Result<String, String> {
    let color = Color::from_hex(buffer)
        .map_err(|_| "Invalid hex color. Use #C586C0 or #C586C080.".to_string())?;
    try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color: current }) = rules.get_mut(role) {
            *current = color;
        }
    })?;
    Ok(format!(
        "{} fixed color -> {}",
        role.label(),
        color.to_hex()
    ))
}

fn try_mutate_rules(state: &mut AppState, update: impl FnOnce(&mut RuleSet)) -> Result<(), String> {
    let previous_rules = state.domain.rules.clone();
    update(&mut state.domain.rules);

    if let Err(err) = state.recompute() {
        state.domain.rules = previous_rules;
        let _ = state.recompute();
        Err(err.to_string())
    } else {
        state.ui.inspector_field = state
            .ui
            .inspector_field
            .min(state.inspector_field_count().saturating_sub(1));
        Ok(())
    }
}

fn reset_state(state: &mut AppState) {
    state.domain.params = ThemeParams::default();
    state.domain.rules = RuleSet::default();
    state.project.name = "Untitled Theme".to_string();
    state.project.export_profiles = default_export_profiles();
    state.ui.selected_token = 0;
    state.ui.selected_param = 0;
    state.ui.inspector_field = 0;
    state.ui.text_input = None;
    state.ui.source_picker = None;
    state.ui.config_modal = None;
    match state.recompute() {
        Ok(()) => state.ui.status = "Reset to defaults.".to_string(),
        Err(err) => state.ui.status = format!("Reset failed: {err}"),
    }
}

fn save_project_effect(state: &AppState) -> Effect {
    Effect::SaveProject {
        path: state.editor.project_path.clone(),
        project: ProjectData {
            name: state.project.name.clone(),
            params: state.domain.params.clone(),
            rules: state.domain.rules.clone(),
            export_profiles: state.project.export_profiles.clone(),
        },
    }
}

fn save_editor_config_effects(state: &AppState) -> Vec<Effect> {
    vec![Effect::SaveEditorConfig {
        data: EditorConfigData {
            config: state.editor_config(),
        },
    }]
}

fn set_project_name(state: &mut AppState, name: String) -> Vec<Effect> {
    let value = name.trim();
    if value.is_empty() {
        state.ui.status = "Project name cannot be empty.".to_string();
        return Vec::new();
    }

    state.project.name = value.to_string();
    state.ui.status = format!("Project name -> {}", state.project.name);
    Vec::new()
}

fn set_export_enabled(state: &mut AppState, index: usize, enabled: bool) -> Vec<Effect> {
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => {
            profile.enabled = enabled;
            let status = if enabled { "enabled" } else { "disabled" };
            state.ui.status = format!("{} {status}.", profile.name);
        }
        None => {
            state.ui.status = format!("Missing export target {}.", index + 1);
        }
    }
    Vec::new()
}

fn set_export_output_path(
    state: &mut AppState,
    index: usize,
    path: std::path::PathBuf,
) -> Vec<Effect> {
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => {
            profile.output_path = path;
            state.ui.status = format!(
                "{} output -> {}",
                profile.name,
                profile.output_path.display()
            );
        }
        None => {
            state.ui.status = format!("Missing export target {}.", index + 1);
        }
    }
    Vec::new()
}

fn set_export_template_path(
    state: &mut AppState,
    index: usize,
    path: std::path::PathBuf,
) -> Vec<Effect> {
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => match &mut profile.format {
            ExportFormat::Template { template_path } => {
                *template_path = path;
                state.ui.status =
                    format!("{} template -> {}", profile.name, template_path.display());
            }
            ExportFormat::Alacritty => {
                state.ui.status = format!("{} does not use a template path.", profile.name);
            }
        },
        None => {
            state.ui.status = format!("Missing export target {}.", index + 1);
        }
    }
    Vec::new()
}

fn set_editor_project_path(state: &mut AppState, path: std::path::PathBuf) -> Vec<Effect> {
    state.editor.project_path = path;
    state.ui.status = format!(
        "Project file path -> {}",
        state.editor.project_path.display()
    );
    save_editor_config_effects(state)
}

fn set_editor_auto_load_project(state: &mut AppState, enabled: bool) -> Vec<Effect> {
    state.editor.auto_load_project_on_startup = enabled;
    state.ui.status = if enabled {
        "Auto-load project on startup enabled.".to_string()
    } else {
        "Auto-load project on startup disabled.".to_string()
    };
    save_editor_config_effects(state)
}

fn set_editor_auto_save_on_export(state: &mut AppState, enabled: bool) -> Vec<Effect> {
    state.editor.auto_save_project_on_export = enabled;
    state.ui.status = if enabled {
        "Auto-save project on export enabled.".to_string()
    } else {
        "Auto-save project on export disabled.".to_string()
    };
    save_editor_config_effects(state)
}

fn set_editor_startup_focus(state: &mut AppState, focus: FocusPane) -> Vec<Effect> {
    state.editor.startup_focus = focus;
    state.ui.focus = focus;
    state.ui.status = format!("Startup focus -> {}.", focus.label());
    save_editor_config_effects(state)
}

fn effects_for_text_target(state: &AppState, target: TextInputTarget) -> Vec<Effect> {
    match target {
        TextInputTarget::Config(
            ConfigFieldId::EditorProjectPath
            | ConfigFieldId::EditorAutoLoadProject
            | ConfigFieldId::EditorAutoSaveOnExport
            | ConfigFieldId::EditorStartupFocus,
        ) => save_editor_config_effects(state),
        _ => Vec::new(),
    }
}

fn input_target_label(target: TextInputTarget) -> String {
    match target {
        TextInputTarget::Control(control) => control.label().to_string(),
        TextInputTarget::Config(field) => match field {
            ConfigFieldId::ProjectName => "project name".to_string(),
            ConfigFieldId::ExportEnabled(index) => format!("export target {}", index + 1),
            ConfigFieldId::ExportOutputPath(index) => format!("export {} output path", index + 1),
            ConfigFieldId::ExportTemplatePath(index) => {
                format!("export {} template path", index + 1)
            }
            ConfigFieldId::EditorProjectPath => "project file path".to_string(),
            ConfigFieldId::EditorAutoLoadProject => "auto load project on startup".to_string(),
            ConfigFieldId::EditorAutoSaveOnExport => "auto save project on export".to_string(),
            ConfigFieldId::EditorStartupFocus => "startup focus".to_string(),
        },
    }
}

fn open_config_modal(state: &mut AppState) {
    state.ui.source_picker = None;
    state.ui.text_input = None;
    state.ui.config_modal = Some(ConfigModalState { selected_field: 0 });
    state.ui.status = "Opened configuration panel.".to_string();
}

fn close_config_modal(state: &mut AppState) {
    let was_open = state.ui.config_modal.take().is_some();
    state.ui.text_input = None;
    if was_open {
        state.ui.status = "Configuration panel closed.".to_string();
    }
}

fn move_config_selection(state: &mut AppState, delta: i32) {
    let len = config_fields(state).len();
    if len == 0 {
        return;
    }

    if let Some(modal) = &mut state.ui.config_modal {
        modal.selected_field = cycle_index(modal.selected_field.min(len - 1), len, delta);
    }
}

fn activate_config_field(state: &mut AppState) -> Vec<Effect> {
    let Some(modal) = &state.ui.config_modal else {
        return Vec::new();
    };

    let fields = config_fields(state);
    let Some(field) = fields
        .get(modal.selected_field.min(fields.len().saturating_sub(1)))
        .copied()
    else {
        return Vec::new();
    };

    if field.supports_text_input() {
        open_text_input(state, TextInputTarget::Config(field));
        return Vec::new();
    }

    match field {
        ConfigFieldId::ExportEnabled(index) => {
            if let Some(profile) = state.project.export_profiles.get_mut(index) {
                profile.enabled = !profile.enabled;
                let status = if profile.enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                state.ui.status = format!("{} {status}.", profile.name);
            } else {
                state.ui.status = format!("Missing export target {}.", index + 1);
            }
        }
        ConfigFieldId::EditorAutoLoadProject => {
            return set_editor_auto_load_project(state, !state.editor.auto_load_project_on_startup);
        }
        ConfigFieldId::EditorAutoSaveOnExport => {
            return set_editor_auto_save_on_export(
                state,
                !state.editor.auto_save_project_on_export,
            );
        }
        ConfigFieldId::EditorStartupFocus => {
            return set_editor_startup_focus(state, state.editor.startup_focus.next());
        }
        ConfigFieldId::ProjectName
        | ConfigFieldId::ExportOutputPath(_)
        | ConfigFieldId::ExportTemplatePath(_)
        | ConfigFieldId::EditorProjectPath => {}
    }

    Vec::new()
}

fn apply_config_input(
    state: &mut AppState,
    field: ConfigFieldId,
    buffer: &str,
) -> Result<String, String> {
    let value = buffer.trim();
    if value.is_empty() {
        return Err("Input is empty.".to_string());
    }

    match field {
        ConfigFieldId::ProjectName => {
            state.project.name = value.to_string();
            Ok(format!("Project name -> {}", state.project.name))
        }
        ConfigFieldId::EditorProjectPath => {
            state.editor.project_path = value.into();
            Ok(format!(
                "Project file path -> {}",
                state.editor.project_path.display()
            ))
        }
        ConfigFieldId::EditorAutoLoadProject
        | ConfigFieldId::EditorAutoSaveOnExport
        | ConfigFieldId::EditorStartupFocus => {
            Err("Use Enter or Space to cycle this editor preference.".to_string())
        }
        ConfigFieldId::ExportOutputPath(index) => {
            let profile = state
                .project
                .export_profiles
                .get_mut(index)
                .ok_or_else(|| format!("Missing export target {}.", index + 1))?;
            profile.output_path = value.into();
            Ok(format!(
                "{} output -> {}",
                profile.name,
                profile.output_path.display()
            ))
        }
        ConfigFieldId::ExportTemplatePath(index) => {
            let profile = state
                .project
                .export_profiles
                .get_mut(index)
                .ok_or_else(|| format!("Missing export target {}.", index + 1))?;
            match &mut profile.format {
                ExportFormat::Template { template_path } => {
                    *template_path = value.into();
                    Ok(format!(
                        "{} template -> {}",
                        profile.name,
                        template_path.display()
                    ))
                }
                ExportFormat::Alacritty => {
                    Err(format!("{} does not use a template path.", profile.name))
                }
            }
        }
        ConfigFieldId::ExportEnabled(index) => Err(format!(
            "Use Enter or Space to toggle export target {}.",
            index + 1
        )),
    }
}

pub fn filtered_source_options(picker: &SourcePickerState) -> Vec<SourceOption> {
    let filter = picker.filter.trim().to_ascii_lowercase();
    let role =
        role_for_reference_control(picker.control).expect("picker control should be reference");
    available_source_options(role)
        .into_iter()
        .filter(|option| {
            filter.is_empty()
                || option.source.label().to_ascii_lowercase().contains(&filter)
                || option.group.label().to_ascii_lowercase().contains(&filter)
        })
        .collect()
}

pub fn config_fields(state: &AppState) -> Vec<ConfigFieldId> {
    let mut fields = vec![ConfigFieldId::ProjectName];
    for (index, profile) in state.project.export_profiles.iter().enumerate() {
        fields.push(ConfigFieldId::ExportEnabled(index));
        fields.push(ConfigFieldId::ExportOutputPath(index));
        if matches!(&profile.format, ExportFormat::Template { .. }) {
            fields.push(ConfigFieldId::ExportTemplatePath(index));
        }
    }
    fields.push(ConfigFieldId::EditorProjectPath);
    fields.push(ConfigFieldId::EditorAutoLoadProject);
    fields.push(ConfigFieldId::EditorAutoSaveOnExport);
    fields.push(ConfigFieldId::EditorStartupFocus);
    fields
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
            ConfigFieldId::ProjectName => state.project.name.clone(),
            ConfigFieldId::EditorProjectPath => state.editor.project_path.display().to_string(),
            ConfigFieldId::EditorAutoLoadProject
            | ConfigFieldId::EditorAutoSaveOnExport
            | ConfigFieldId::EditorStartupFocus => String::new(),
            ConfigFieldId::ExportOutputPath(index) => state
                .project
                .export_profiles
                .get(index)
                .map(|profile| profile.output_path.display().to_string())
                .unwrap_or_default(),
            ConfigFieldId::ExportTemplatePath(index) => state
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
            ConfigFieldId::ExportEnabled(_) => String::new(),
        },
    }
}

fn parse_param_input(key: ParamKey, buffer: &str) -> Result<f32, String> {
    match key {
        ParamKey::BackgroundHue | ParamKey::AccentHue => parse_float_input(buffer),
        _ => parse_fraction_input(buffer),
    }
}

fn parse_float_input(buffer: &str) -> Result<f32, String> {
    let trimmed = buffer.trim().trim_end_matches('%').trim();
    if trimmed.is_empty() {
        return Err("Input is empty.".to_string());
    }
    trimmed
        .parse::<f32>()
        .map_err(|_| format!("Invalid number: {buffer}"))
}

fn parse_fraction_input(buffer: &str) -> Result<f32, String> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Err("Input is empty.".to_string());
    }

    let is_percent = trimmed.ends_with('%');
    let number = trimmed
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("Invalid number: {buffer}"))?;

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

pub fn cycle_index(current: usize, len: usize, delta: i32) -> usize {
    let len = len as i32;
    ((current as i32 + delta).rem_euclid(len)) as usize
}

fn cycle_rule_kind(current: RuleKind, delta: i32) -> RuleKind {
    let index = RuleKind::ALL
        .iter()
        .position(|kind| *kind == current)
        .unwrap_or_default();
    RuleKind::ALL[cycle_index(index, RuleKind::ALL.len(), delta)]
}

fn cycle_source(current: &SourceRef, role: TokenRole, delta: i32) -> SourceRef {
    let options = available_sources(role);
    let index = options
        .iter()
        .position(|option| option == current)
        .unwrap_or_default();
    options[cycle_index(index, options.len(), delta)].clone()
}

fn cycle_adjust_op(current: AdjustOp, delta: i32) -> AdjustOp {
    let index = AdjustOp::ALL
        .iter()
        .position(|op| *op == current)
        .unwrap_or_default();
    AdjustOp::ALL[cycle_index(index, AdjustOp::ALL.len(), delta)]
}

fn cycle_fixed_color(current: Color, options: &[Color], delta: i32) -> Color {
    let index = options
        .iter()
        .position(|candidate| candidate.approx_eq(current))
        .unwrap_or_default();
    options[cycle_index(index, options.len(), delta)]
}

fn role_for_reference_control(control: ControlId) -> Option<TokenRole> {
    match control {
        ControlId::Reference(role, _) => Some(role),
        _ => None,
    }
}

pub fn current_source_for_control(control: ControlId, rule: &Rule) -> Option<SourceRef> {
    match (control, rule) {
        (ControlId::Reference(_, ReferenceField::AliasSource), Rule::Alias { source }) => {
            Some(source.clone())
        }
        (ControlId::Reference(_, ReferenceField::MixA), Rule::Mix { a, .. }) => Some(a.clone()),
        (ControlId::Reference(_, ReferenceField::MixB), Rule::Mix { b, .. }) => Some(b.clone()),
        (ControlId::Reference(_, ReferenceField::AdjustSource), Rule::Adjust { source, .. }) => {
            Some(source.clone())
        }
        _ => None,
    }
}

pub fn apply_source_to_control(control: ControlId, rule: &mut Rule, source: SourceRef) {
    match (control, rule) {
        (ControlId::Reference(_, ReferenceField::AliasSource), Rule::Alias { source: current }) => {
            *current = source
        }
        (ControlId::Reference(_, ReferenceField::MixA), Rule::Mix { a, .. }) => *a = source,
        (ControlId::Reference(_, ReferenceField::MixB), Rule::Mix { b, .. }) => *b = source,
        (
            ControlId::Reference(_, ReferenceField::AdjustSource),
            Rule::Adjust {
                source: current, ..
            },
        ) => *current = source,
        _ => {}
    }
}
