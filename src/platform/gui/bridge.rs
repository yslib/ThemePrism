use crate::app::controls::{ControlId, ReferenceField};
use crate::app::snapshot::{
    AppSnapshot, ChoiceFieldSnapshot, ChoiceOptionSnapshot, ColorFieldSnapshot,
    EditorFieldSnapshot, PreviewLineSnapshot, PreviewSegmentSnapshot, ProjectSnapshot,
    ScalarFieldSnapshot, SwatchSnapshot, ThemeSnapshot,
};
#[cfg(test)]
use crate::core::AppState;
use crate::core::{CoreSession, Intent};
use crate::domain::color::Color;
use crate::domain::params::ParamKey;
use crate::domain::rules::{AdjustOp, RuleKind, SourceRef};
use crate::domain::tokens::{PaletteSlot, TokenRole};

#[derive(Debug)]
pub struct GuiBridgeSession {
    core: CoreSession,
}

impl GuiBridgeSession {
    #[cfg(test)]
    pub fn new(state: AppState) -> Self {
        Self::from_core(CoreSession::new(state))
    }

    pub fn from_core(core: CoreSession) -> Self {
        Self { core }
    }

    pub fn snapshot_json(&self) -> String {
        snapshot_to_json(&self.core.snapshot())
    }

    pub fn dispatch(&mut self, command: &str) {
        match parse_command(command) {
            Ok(intent) => self.core.dispatch(intent),
            Err(message) => self.core.set_status(format!("GUI command rejected: {message}")),
        }
    }

    #[cfg(test)]
    pub fn state(&self) -> &AppState {
        self.core.state()
    }
}

fn parse_command(command: &str) -> Result<Intent, String> {
    let mut parts = command.split('|');
    let action = parts
        .next()
        .ok_or_else(|| "command is empty".to_string())?
        .trim();

    match action {
        "select-token" => {
            let index = parts
                .next()
                .ok_or_else(|| "missing token index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid token index".to_string())?;
            Ok(Intent::SelectToken(index))
        }
        "set-scalar" => {
            let control = parse_control_id(
                parts
                    .next()
                    .ok_or_else(|| "missing control id".to_string())?,
            )?;
            let value = parts
                .next()
                .ok_or_else(|| "missing scalar value".to_string())?
                .parse::<f32>()
                .map_err(|_| "invalid scalar value".to_string())?;

            match control {
                ControlId::Param(key) => Ok(Intent::SetParamValue(key, value)),
                ControlId::MixRatio(role) => Ok(Intent::SetMixRatio(role, value)),
                ControlId::AdjustAmount(role) => Ok(Intent::SetAdjustAmount(role, value)),
                _ => Err(format!("{command} does not target a scalar control")),
            }
        }
        "set-choice" => {
            let control = parse_control_id(
                parts
                    .next()
                    .ok_or_else(|| "missing control id".to_string())?,
            )?;
            let selected = parts
                .next()
                .ok_or_else(|| "missing selected key".to_string())?;

            match control {
                ControlId::RuleKind(role) => {
                    Ok(Intent::SetRuleKind(role, parse_rule_kind(selected)?))
                }
                ControlId::Reference(_, _) => Ok(Intent::SetReferenceSource(
                    control,
                    parse_source_ref(selected)?,
                )),
                ControlId::AdjustOp(role) => {
                    Ok(Intent::SetAdjustOp(role, parse_adjust_op(selected)?))
                }
                _ => Err(format!("{command} does not target a choice control")),
            }
        }
        "set-text" => {
            let control = parse_control_id(
                parts
                    .next()
                    .ok_or_else(|| "missing control id".to_string())?,
            )?;
            let value = parts
                .next()
                .ok_or_else(|| "missing text value".to_string())?;

            match control {
                ControlId::FixedColor(role) => Ok(Intent::SetFixedColor(
                    role,
                    Color::from_hex(value).map_err(|err| err.to_string())?,
                )),
                _ => Err(format!("{command} does not target a text control")),
            }
        }
        "save" => Ok(Intent::SaveProjectRequested),
        "load" => Ok(Intent::LoadProjectRequested),
        "export" => Ok(Intent::ExportThemeRequested),
        "reset" => Ok(Intent::ResetRequested),
        "quit" => Ok(Intent::QuitRequested),
        _ => Err(format!("unknown GUI command: {action}")),
    }
}

fn parse_control_id(raw: &str) -> Result<ControlId, String> {
    let mut parts = raw.split(':');
    let prefix = parts
        .next()
        .ok_or_else(|| "missing control prefix".to_string())?;

    match prefix {
        "param" => Ok(ControlId::Param(parse_param_key(
            parts
                .next()
                .ok_or_else(|| "missing param key".to_string())?,
        )?)),
        "rule_kind" => Ok(ControlId::RuleKind(parse_token_role(
            parts.next().ok_or_else(|| "missing role".to_string())?,
        )?)),
        "reference" => Ok(ControlId::Reference(
            parse_token_role(parts.next().ok_or_else(|| "missing role".to_string())?)?,
            parse_reference_field(parts.next().ok_or_else(|| "missing field".to_string())?)?,
        )),
        "mix_ratio" => Ok(ControlId::MixRatio(parse_token_role(
            parts.next().ok_or_else(|| "missing role".to_string())?,
        )?)),
        "adjust_op" => Ok(ControlId::AdjustOp(parse_token_role(
            parts.next().ok_or_else(|| "missing role".to_string())?,
        )?)),
        "adjust_amount" => Ok(ControlId::AdjustAmount(parse_token_role(
            parts.next().ok_or_else(|| "missing role".to_string())?,
        )?)),
        "fixed_color" => Ok(ControlId::FixedColor(parse_token_role(
            parts.next().ok_or_else(|| "missing role".to_string())?,
        )?)),
        _ => Err(format!("unknown control id: {raw}")),
    }
}

fn parse_param_key(raw: &str) -> Result<ParamKey, String> {
    match raw {
        "background_hue" => Ok(ParamKey::BackgroundHue),
        "background_lightness" => Ok(ParamKey::BackgroundLightness),
        "background_saturation" => Ok(ParamKey::BackgroundSaturation),
        "contrast" => Ok(ParamKey::Contrast),
        "accent_hue" => Ok(ParamKey::AccentHue),
        "accent_saturation" => Ok(ParamKey::AccentSaturation),
        "accent_lightness" => Ok(ParamKey::AccentLightness),
        "selection_mix" => Ok(ParamKey::SelectionMix),
        "vibrancy" => Ok(ParamKey::Vibrancy),
        _ => Err(format!("unknown param key: {raw}")),
    }
}

fn parse_token_role(raw: &str) -> Result<TokenRole, String> {
    match raw {
        "background" => Ok(TokenRole::Background),
        "surface" => Ok(TokenRole::Surface),
        "surfacealt" => Ok(TokenRole::SurfaceAlt),
        "text" => Ok(TokenRole::Text),
        "textmuted" => Ok(TokenRole::TextMuted),
        "border" => Ok(TokenRole::Border),
        "selection" => Ok(TokenRole::Selection),
        "cursor" => Ok(TokenRole::Cursor),
        "comment" => Ok(TokenRole::Comment),
        "keyword" => Ok(TokenRole::Keyword),
        "string" => Ok(TokenRole::String),
        "number" => Ok(TokenRole::Number),
        "type" => Ok(TokenRole::Type),
        "function" => Ok(TokenRole::Function),
        "variable" => Ok(TokenRole::Variable),
        "error" => Ok(TokenRole::Error),
        "warning" => Ok(TokenRole::Warning),
        "info" => Ok(TokenRole::Info),
        "hint" => Ok(TokenRole::Hint),
        "success" => Ok(TokenRole::Success),
        _ => Err(format!("unknown token role: {raw}")),
    }
}

fn parse_reference_field(raw: &str) -> Result<ReferenceField, String> {
    match raw {
        "alias_source" => Ok(ReferenceField::AliasSource),
        "mix_a" => Ok(ReferenceField::MixA),
        "mix_b" => Ok(ReferenceField::MixB),
        "adjust_source" => Ok(ReferenceField::AdjustSource),
        _ => Err(format!("unknown reference field: {raw}")),
    }
}

fn parse_rule_kind(raw: &str) -> Result<RuleKind, String> {
    match raw {
        "alias" => Ok(RuleKind::Alias),
        "mix" => Ok(RuleKind::Mix),
        "adjust" => Ok(RuleKind::Adjust),
        "fixed" => Ok(RuleKind::Fixed),
        _ => Err(format!("unknown rule kind: {raw}")),
    }
}

fn parse_adjust_op(raw: &str) -> Result<AdjustOp, String> {
    match raw {
        "lighten" => Ok(AdjustOp::Lighten),
        "darken" => Ok(AdjustOp::Darken),
        "saturate" => Ok(AdjustOp::Saturate),
        "desaturate" => Ok(AdjustOp::Desaturate),
        _ => Err(format!("unknown adjust op: {raw}")),
    }
}

fn parse_palette_slot(raw: &str) -> Result<PaletteSlot, String> {
    match raw {
        "bg_0" => Ok(PaletteSlot::Bg0),
        "bg_1" => Ok(PaletteSlot::Bg1),
        "bg_2" => Ok(PaletteSlot::Bg2),
        "fg_0" => Ok(PaletteSlot::Fg0),
        "fg_1" => Ok(PaletteSlot::Fg1),
        "fg_2" => Ok(PaletteSlot::Fg2),
        "accent_0" => Ok(PaletteSlot::Accent0),
        "accent_1" => Ok(PaletteSlot::Accent1),
        "accent_2" => Ok(PaletteSlot::Accent2),
        "accent_3" => Ok(PaletteSlot::Accent3),
        "accent_4" => Ok(PaletteSlot::Accent4),
        "accent_5" => Ok(PaletteSlot::Accent5),
        _ => Err(format!("unknown palette slot: {raw}")),
    }
}

fn parse_source_ref(raw: &str) -> Result<SourceRef, String> {
    let (kind, value) = raw
        .split_once(':')
        .ok_or_else(|| "invalid source reference".to_string())?;
    match kind {
        "token" => Ok(SourceRef::Token(parse_token_role(value)?)),
        "palette" => Ok(SourceRef::Palette(parse_palette_slot(value)?)),
        "literal" => Ok(SourceRef::Literal(
            Color::from_hex(value).map_err(|err| err.to_string())?,
        )),
        _ => Err(format!("unknown source reference: {raw}")),
    }
}

pub fn snapshot_to_json(snapshot: &AppSnapshot) -> String {
    let tokens = join_json(snapshot.tokens.iter().map(token_to_json));
    let params = join_json(snapshot.params.iter().map(scalar_to_json));
    let fields = join_json(snapshot.inspector.fields.iter().map(editor_field_to_json));
    let palette = join_json(snapshot.palette.iter().map(swatch_to_json));
    let resolved = join_json(snapshot.resolved_tokens.iter().map(swatch_to_json));
    let preview = join_json(snapshot.preview.iter().map(preview_line_to_json));

    format!(
        "{{\"window_title\":{},\"status\":{},\"project\":{},\"theme\":{},\"tokens\":[{}],\"params\":[{}],\"inspector\":{{\"token_id\":{},\"token_label\":{},\"token_color_hex\":{},\"rule_summary\":{},\"fields\":[{}]}} ,\"palette\":[{}],\"resolved_tokens\":[{}],\"preview\":[{}]}}",
        json_string(&snapshot.window_title),
        json_string(&snapshot.status),
        project_to_json(&snapshot.project),
        theme_to_json(&snapshot.theme),
        tokens,
        params,
        json_string(&snapshot.inspector.token_id),
        json_string(&snapshot.inspector.token_label),
        json_string(&snapshot.inspector.token_color_hex),
        json_string(&snapshot.inspector.rule_summary),
        fields,
        palette,
        resolved,
        preview,
    )
}

fn project_to_json(project: &ProjectSnapshot) -> String {
    format!(
        "{{\"name\":{},\"project_path\":{},\"export_profile_name\":{},\"export_format\":{},\"export_output_path\":{}}}",
        json_string(&project.name),
        json_string(&project.project_path),
        json_string(&project.export_profile_name),
        json_string(&project.export_format),
        json_string(&project.export_output_path),
    )
}

fn token_to_json(token: &crate::app::snapshot::TokenItemSnapshot) -> String {
    format!(
        "{{\"index\":{},\"id\":{},\"label\":{},\"category\":{},\"color_hex\":{},\"selected\":{}}}",
        token.index,
        json_string(&token.id),
        json_string(&token.label),
        json_string(&token.category),
        json_string(&token.color_hex),
        json_bool(token.selected),
    )
}

fn theme_to_json(theme: &ThemeSnapshot) -> String {
    format!(
        "{{\"background_hex\":{},\"surface_hex\":{},\"border_hex\":{},\"selection_hex\":{},\"text_hex\":{},\"muted_hex\":{}}}",
        json_string(&theme.background_hex),
        json_string(&theme.surface_hex),
        json_string(&theme.border_hex),
        json_string(&theme.selection_hex),
        json_string(&theme.text_hex),
        json_string(&theme.muted_hex),
    )
}

fn editor_field_to_json(field: &EditorFieldSnapshot) -> String {
    match field {
        EditorFieldSnapshot::Choice(choice) => choice_to_json(choice),
        EditorFieldSnapshot::Scalar(scalar) => {
            format!("{{\"kind\":\"scalar\",{}}}", scalar_json_body(scalar))
        }
        EditorFieldSnapshot::Color(color) => color_field_to_json(color),
    }
}

fn scalar_to_json(field: &ScalarFieldSnapshot) -> String {
    format!("{{\"kind\":\"scalar\",{}}}", scalar_json_body(field))
}

fn scalar_json_body(field: &ScalarFieldSnapshot) -> String {
    format!(
        "\"id\":{},\"label\":{},\"value_text\":{},\"current\":{},\"min\":{},\"max\":{},\"step\":{}",
        json_string(&field.id),
        json_string(&field.label),
        json_string(&field.value_text),
        field.current,
        field.min,
        field.max,
        field.step,
    )
}

fn choice_to_json(field: &ChoiceFieldSnapshot) -> String {
    let options = join_json(field.options.iter().map(choice_option_to_json));
    format!(
        "{{\"kind\":\"choice\",\"id\":{},\"label\":{},\"value_text\":{},\"selected_key\":{},\"options\":[{}]}}",
        json_string(&field.id),
        json_string(&field.label),
        json_string(&field.value_text),
        json_string(&field.selected_key),
        options,
    )
}

fn choice_option_to_json(option: &ChoiceOptionSnapshot) -> String {
    format!(
        "{{\"key\":{},\"label\":{}}}",
        json_string(&option.key),
        json_string(&option.label),
    )
}

fn color_field_to_json(field: &ColorFieldSnapshot) -> String {
    format!(
        "{{\"kind\":\"color\",\"id\":{},\"label\":{},\"value_text\":{},\"color_hex\":{}}}",
        json_string(&field.id),
        json_string(&field.label),
        json_string(&field.value_text),
        json_string(&field.color_hex),
    )
}

fn swatch_to_json(swatch: &SwatchSnapshot) -> String {
    format!(
        "{{\"label\":{},\"color_hex\":{}}}",
        json_string(&swatch.label),
        json_string(&swatch.color_hex),
    )
}

fn preview_line_to_json(line: &PreviewLineSnapshot) -> String {
    let segments = join_json(line.segments.iter().map(preview_segment_to_json));
    format!("{{\"segments\":[{}]}}", segments)
}

fn preview_segment_to_json(segment: &PreviewSegmentSnapshot) -> String {
    format!(
        "{{\"text\":{},\"foreground_hex\":{},\"background_hex\":{}}}",
        json_string(&segment.text),
        json_string(&segment.foreground_hex),
        json_string(&segment.background_hex),
    )
}

fn join_json(items: impl IntoIterator<Item = String>) -> String {
    items.into_iter().collect::<Vec<_>>().join(",")
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", escape_json(value))
}

fn json_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn escape_json(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04X}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::GuiBridgeSession;
    use crate::app::AppState;
    use crate::domain::color::Color;
    use crate::domain::params::ParamKey;
    use crate::domain::rules::Rule;
    use crate::domain::tokens::TokenRole;

    #[test]
    fn gui_bridge_updates_scalar_params() {
        let mut session = GuiBridgeSession::new(AppState::new().unwrap());
        session.dispatch("set-scalar|param:contrast|0.420000");

        assert!((session.state().domain.params.contrast - 0.42).abs() < 0.001);
        assert_eq!(
            ParamKey::Contrast.format_value(&session.state().domain.params),
            "    42%"
        );
    }

    #[test]
    fn gui_bridge_can_switch_to_fixed_and_set_hex() {
        let mut session = GuiBridgeSession::new(AppState::new().unwrap());
        session.dispatch("set-choice|rule_kind:background|fixed");
        session.dispatch("set-text|fixed_color:background|#224466");

        match session.state().domain.rules.get(TokenRole::Background) {
            Some(Rule::Fixed { color }) => {
                assert!(color.approx_eq(Color::from_hex("#224466").unwrap()));
            }
            other => panic!("expected fixed background rule, got {other:?}"),
        }

        assert_eq!(
            session.state().theme_color(TokenRole::Background).to_hex(),
            "#224466"
        );
    }

    #[test]
    fn gui_bridge_updates_reference_choices() {
        let mut session = GuiBridgeSession::new(AppState::new().unwrap());
        session.dispatch("set-choice|reference:background:alias_source|palette:accent_3");

        match session.state().domain.rules.get(TokenRole::Background) {
            Some(Rule::Alias { source }) => {
                assert_eq!(source.label(), "accent_3");
            }
            other => panic!("expected alias rule, got {other:?}"),
        }
    }
}
