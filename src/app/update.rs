use crate::app::controls::{ControlId, ReferenceField};
use crate::app::effect::{Effect, ProjectData};
use crate::app::intent::Intent;
use crate::app::state::{AppState, FocusPane, SourcePickerState, TextInputState};
use crate::domain::color::Color;
use crate::domain::params::{ParamKey, ThemeParams};
use crate::domain::rules::{
    AdjustOp, Rule, RuleKind, RuleSet, SourceOption, SourceRef, available_source_options,
    available_sources, starter_rule,
};
use crate::domain::tokens::TokenRole;

pub fn update(state: &mut AppState, intent: Intent) -> Vec<Effect> {
    match intent {
        Intent::QuitRequested => {
            state.ui.should_quit = true;
            Vec::new()
        }
        Intent::MoveFocus(delta) => {
            if state.ui.source_picker.is_some() || state.ui.text_input.is_some() {
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
            if state.ui.source_picker.is_some() || state.ui.text_input.is_some() {
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
            if state.ui.source_picker.is_some() || state.ui.text_input.is_some() {
                return Vec::new();
            }
            adjust_control_by_step(state, control, delta);
            Vec::new()
        }
        Intent::ActivateControl(control) => {
            if state.ui.source_picker.is_some() || state.ui.text_input.is_some() {
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
        Intent::CommitTextInput => {
            commit_text_input(state);
            Vec::new()
        }
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
        Intent::SaveProjectRequested => vec![Effect::SaveProject {
            path: state.project.project_path.clone(),
            project: ProjectData {
                name: state.project.name.clone(),
                params: state.domain.params.clone(),
                rules: state.domain.rules.clone(),
                export_profile: state.project.export_profile.clone(),
            },
        }],
        Intent::LoadProjectRequested => vec![Effect::LoadProject {
            path: state.project.project_path.clone(),
        }],
        Intent::ExportThemeRequested => vec![Effect::ExportTheme {
            profile: state.project.export_profile.clone(),
            theme: state.domain.resolved.clone(),
        }],
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
                Ok(project) => {
                    state.project.name = project.name;
                    state.domain.params = project.params;
                    state.domain.rules = project.rules;
                    state.project.export_profile = project.export_profile;
                    state.ui.text_input = None;
                    state.ui.source_picker = None;
                    match state.recompute() {
                        Ok(()) => {
                            state.ui.inspector_field = state
                                .ui
                                .inspector_field
                                .min(state.inspector_field_count().saturating_sub(1));
                            state.ui.status = format!(
                                "Loaded project from {}",
                                state.project.project_path.display()
                            );
                        }
                        Err(err) => {
                            state.ui.status = format!("Load recompute failed: {err}");
                        }
                    }
                }
                Err(err) => {
                    state.ui.status = format!("Load failed: {err}");
                }
            }
            Vec::new()
        }
        Intent::ThemeExported(result) => {
            state.ui.status = match result {
                Ok(artifact) => format!(
                    "Exported {} to {}",
                    artifact.profile_name,
                    artifact.output_path.display()
                ),
                Err(err) => format!("Export failed: {err}"),
            };
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
        open_text_input(state, control);
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

fn open_text_input(state: &mut AppState, control: ControlId) {
    let buffer = default_input_buffer(state, control);
    state.ui.text_input = Some(TextInputState { control, buffer });
    state.ui.status = format!(
        "Editing {}. Press Enter to apply or Esc to cancel.",
        control.label()
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
        .map(|input| input.control.label())
        .unwrap_or_else(|| "input field".to_string());

    if let Some(input) = &mut state.ui.text_input {
        if accepts_input_char(input.control, ch, &input.buffer) {
            let ch = match input.control {
                ControlId::FixedColor(_) => ch.to_ascii_uppercase(),
                _ => ch,
            };
            input.buffer.push(ch);
        } else {
            state.ui.status = format!("Character '{}' is not valid for {}.", ch, label);
        }
    }
}

fn commit_text_input(state: &mut AppState) {
    let Some(input) = state.ui.text_input.clone() else {
        return;
    };

    let result = match input.control {
        ControlId::Param(key) => apply_param_input(state, key, &input.buffer),
        ControlId::MixRatio(role) => apply_mix_ratio_input(state, role, &input.buffer),
        ControlId::AdjustAmount(role) => apply_adjust_amount_input(state, role, &input.buffer),
        ControlId::FixedColor(role) => apply_fixed_color_input(state, role, &input.buffer),
        _ => Err(format!(
            "{} does not support text input.",
            input.control.label()
        )),
    };

    match result {
        Ok(status) => {
            state.ui.text_input = None;
            state.ui.status = status;
        }
        Err(err) => {
            state.ui.status = err;
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
        .map_err(|_| "Invalid hex color. Use a 6-digit value like #C586C0.".to_string())?;
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
    state.ui.selected_token = 0;
    state.ui.selected_param = 0;
    state.ui.inspector_field = 0;
    state.ui.text_input = None;
    state.ui.source_picker = None;
    match state.recompute() {
        Ok(()) => state.ui.status = "Reset to defaults.".to_string(),
        Err(err) => state.ui.status = format!("Reset failed: {err}"),
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

pub fn default_input_buffer(state: &AppState, control: ControlId) -> String {
    match control {
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

fn accepts_input_char(control: ControlId, ch: char, existing: &str) -> bool {
    match control {
        ControlId::Param(_) | ControlId::MixRatio(_) | ControlId::AdjustAmount(_) => {
            ch.is_ascii_digit() || ch == '.' || ch == '%'
        }
        ControlId::FixedColor(_) => {
            ch.is_ascii_hexdigit() || (ch == '#' && !existing.contains('#') && existing.is_empty())
        }
        _ => false,
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
