use crate::app::controls::{ControlId, ReferenceField};
use crate::app::state::AppState;
use crate::domain::color::Color;
use crate::domain::params::ParamKey;
use crate::domain::rules::{
    AdjustOp, Rule, RuleKind, RuleSet, SourceRef, available_sources, starter_rule,
};
use crate::domain::tokens::TokenRole;
use crate::i18n::UiText;

use super::{text_input, tr1, tr2};

pub(super) fn adjust_control_by_step(state: &mut AppState, control: ControlId, delta: i32) {
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

pub(super) fn activate_control(state: &mut AppState, control: ControlId) {
    if control.supports_source_picker() {
        text_input::open_source_picker(state, control);
    } else if control.supports_text_input() {
        text_input::open_text_input(state, crate::app::state::TextInputTarget::Control(control));
    } else {
        state.ui.status = tr1(
            state,
            UiText::StatusControlNoActivation,
            "control",
            control.label(),
        );
    }
}

pub(super) fn set_param_value(state: &mut AppState, key: ParamKey, value: f32) {
    let mut params = state.domain.params.clone();
    key.set(&mut params, value);
    state.domain.params = params;
    if let Err(err) = state.recompute() {
        state.ui.status = tr2(
            state,
            UiText::StatusFailedToUpdateField,
            "field",
            key.label(),
            "error",
            err,
        );
    } else {
        state.ui.status = tr2(
            state,
            UiText::StatusUpdatedFieldValue,
            "field",
            key.label(),
            "value",
            key.format_value(&state.domain.params),
        );
    }
}

fn adjust_param_by_step(state: &mut AppState, key: ParamKey, delta: i32) {
    let mut params = state.domain.params.clone();
    key.adjust(&mut params, delta);
    state.domain.params = params;
    if let Err(err) = state.recompute() {
        state.ui.status = tr2(
            state,
            UiText::StatusFailedToUpdateField,
            "field",
            key.label(),
            "error",
            err,
        );
    } else {
        state.ui.status = tr2(
            state,
            UiText::StatusUpdatedFieldValue,
            "field",
            key.label(),
            "value",
            key.format_value(&state.domain.params),
        );
    }
}

pub(super) fn set_rule_kind_for_role(state: &mut AppState, role: TokenRole, kind: RuleKind) {
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
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
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
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

pub(super) fn set_reference_source(state: &mut AppState, control: ControlId, source: SourceRef) {
    let result = try_mutate_rules(state, |rules| {
        let role =
            role_for_reference_control(control).expect("control should be reference control");
        let rule = rules
            .get_mut(role)
            .expect("selected token should have a rule");
        apply_source_to_control(control, rule, source.clone());
    });

    state.ui.status = match result {
        Ok(()) => tr1(
            state,
            UiText::StatusUpdatedEntity,
            "entity",
            control.label(),
        ),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
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
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

pub(super) fn set_mix_ratio(state: &mut AppState, role: TokenRole, ratio: f32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio: current, .. }) = rules.get_mut(role) {
            *current = ratio.clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_mix_ratio_by_step(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Mix { ratio, .. }) = rules.get_mut(role) {
            *ratio = (*ratio + delta as f32 * 0.05).clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

pub(super) fn set_adjust_op(state: &mut AppState, role: TokenRole, next: AdjustOp) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { op, .. }) = rules.get_mut(role) {
            *op = next;
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn cycle_adjust_op_for_role(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { op, .. }) = rules.get_mut(role) {
            *op = cycle_adjust_op(*op, delta);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

pub(super) fn set_adjust_amount(state: &mut AppState, role: TokenRole, amount: f32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust {
            amount: current, ..
        }) = rules.get_mut(role)
        {
            *current = amount.clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

fn set_adjust_amount_by_step(state: &mut AppState, role: TokenRole, delta: i32) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Adjust { amount, .. }) = rules.get_mut(role) {
            *amount = (*amount + delta as f32 * 0.02).clamp(0.0, 1.0);
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

pub(super) fn set_fixed_color(state: &mut AppState, role: TokenRole, next: Color) {
    let result = try_mutate_rules(state, |rules| {
        if let Some(Rule::Fixed { color }) = rules.get_mut(role) {
            *color = next;
        }
    });
    state.ui.status = match result {
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
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
        Ok(()) => tr1(state, UiText::StatusUpdatedEntity, "entity", role.label()),
        Err(err) => tr1(state, UiText::StatusRuleChangeRejected, "error", err),
    };
}

pub(super) fn try_mutate_rules(
    state: &mut AppState,
    update: impl FnOnce(&mut RuleSet),
) -> Result<(), String> {
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

fn cycle_rule_kind(current: RuleKind, delta: i32) -> RuleKind {
    let index = RuleKind::ALL
        .iter()
        .position(|kind| *kind == current)
        .unwrap_or_default();
    RuleKind::ALL[super::cycle_index(index, RuleKind::ALL.len(), delta)]
}

fn cycle_source(current: &SourceRef, role: TokenRole, delta: i32) -> SourceRef {
    let options = available_sources(role);
    let index = options
        .iter()
        .position(|option| option == current)
        .unwrap_or_default();
    options[super::cycle_index(index, options.len(), delta)].clone()
}

fn cycle_adjust_op(current: AdjustOp, delta: i32) -> AdjustOp {
    let index = AdjustOp::ALL
        .iter()
        .position(|op| *op == current)
        .unwrap_or_default();
    AdjustOp::ALL[super::cycle_index(index, AdjustOp::ALL.len(), delta)]
}

fn cycle_fixed_color(current: Color, options: &[Color], delta: i32) -> Color {
    let index = options
        .iter()
        .position(|candidate| candidate.approx_eq(current))
        .unwrap_or_default();
    options[super::cycle_index(index, options.len(), delta)]
}

pub(super) fn role_for_reference_control(control: ControlId) -> Option<TokenRole> {
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

pub(super) fn apply_source_to_control(control: ControlId, rule: &mut Rule, source: SourceRef) {
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
