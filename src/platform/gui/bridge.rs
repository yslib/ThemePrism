use crate::app::controls::{ControlId, ReferenceField};
use crate::app::snapshot::AppSnapshot;
use crate::app::state::FocusPane;
#[cfg(test)]
use crate::core::AppState;
use crate::core::{CoreSession, Intent};
use crate::domain::color::Color;
use crate::domain::params::ParamKey;
use crate::domain::rules::{AdjustOp, RuleKind, SourceRef};
use crate::domain::tokens::{PaletteSlot, TokenRole};
use crate::persistence::editor_config::{EditorKeymapPreset, EditorLocale};

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
            Err(message) => self
                .core
                .set_status(format!("GUI command rejected: {message}")),
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
        "set-project-name" => {
            let value = parts
                .next()
                .ok_or_else(|| "missing project name".to_string())?;
            Ok(Intent::SetProjectName(value.to_string()))
        }
        "set-export-enabled" => {
            let index = parts
                .next()
                .ok_or_else(|| "missing export index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid export index".to_string())?;
            let enabled = parse_bool(
                parts
                    .next()
                    .ok_or_else(|| "missing export enabled value".to_string())?,
            )?;
            Ok(Intent::SetExportEnabled(index, enabled))
        }
        "set-export-output" => {
            let index = parts
                .next()
                .ok_or_else(|| "missing export index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid export index".to_string())?;
            let value = parts
                .next()
                .ok_or_else(|| "missing export output path".to_string())?;
            Ok(Intent::SetExportOutputPath(index, value.into()))
        }
        "set-export-template" => {
            let index = parts
                .next()
                .ok_or_else(|| "missing export index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid export index".to_string())?;
            let value = parts
                .next()
                .ok_or_else(|| "missing export template path".to_string())?;
            Ok(Intent::SetExportTemplatePath(index, value.into()))
        }
        "set-editor-text" => {
            let field = parts
                .next()
                .ok_or_else(|| "missing editor field id".to_string())?;
            let value = parts
                .next()
                .ok_or_else(|| "missing editor text value".to_string())?;

            match field {
                "project_path" => Ok(Intent::SetEditorProjectPath(value.into())),
                _ => Err(format!("unknown editor text field: {field}")),
            }
        }
        "set-editor-toggle" => {
            let field = parts
                .next()
                .ok_or_else(|| "missing editor field id".to_string())?;
            let enabled = parse_bool(
                parts
                    .next()
                    .ok_or_else(|| "missing toggle value".to_string())?,
            )?;

            match field {
                "auto_load_project_on_startup" => Ok(Intent::SetEditorAutoLoadProject(enabled)),
                "auto_save_project_on_export" => Ok(Intent::SetEditorAutoSaveOnExport(enabled)),
                _ => Err(format!("unknown editor toggle field: {field}")),
            }
        }
        "set-editor-choice" => {
            let field = parts
                .next()
                .ok_or_else(|| "missing editor field id".to_string())?;
            let selected = parts
                .next()
                .ok_or_else(|| "missing editor choice value".to_string())?;

            match field {
                "startup_focus" => Ok(Intent::SetEditorStartupFocus(parse_focus_pane(selected)?)),
                "keymap_preset" => Ok(Intent::SetEditorKeymapPreset(parse_keymap_preset(
                    selected,
                )?)),
                "locale" => Ok(Intent::SetEditorLocale(parse_editor_locale(selected)?)),
                _ => Err(format!("unknown editor choice field: {field}")),
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

fn parse_focus_pane(raw: &str) -> Result<FocusPane, String> {
    match raw {
        "tokens" => Ok(FocusPane::Tokens),
        "params" => Ok(FocusPane::Params),
        "inspector" => Ok(FocusPane::Inspector),
        _ => Err(format!("unknown focus pane: {raw}")),
    }
}

fn parse_keymap_preset(raw: &str) -> Result<EditorKeymapPreset, String> {
    match raw {
        "standard" => Ok(EditorKeymapPreset::Standard),
        "vim" => Ok(EditorKeymapPreset::Vim),
        _ => Err(format!("unknown keymap preset: {raw}")),
    }
}

fn parse_editor_locale(raw: &str) -> Result<EditorLocale, String> {
    match raw {
        "en_us" => Ok(EditorLocale::EnUs),
        "zh_cn" => Ok(EditorLocale::ZhCn),
        _ => Err(format!("unknown editor locale: {raw}")),
    }
}

fn parse_bool(raw: &str) -> Result<bool, String> {
    match raw {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid boolean value: {raw}")),
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
    serde_json::to_string(snapshot).expect("app snapshot should serialize to JSON")
}

#[cfg(test)]
mod tests {
    use super::{GuiBridgeSession, parse_command};
    use crate::app::AppState;
    use crate::app::Intent;
    use crate::app::state::FocusPane;
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

    #[test]
    fn gui_bridge_parses_editor_preference_commands() {
        assert!(matches!(
            parse_command("set-project-name|Aurora Theme"),
            Ok(Intent::SetProjectName(name)) if name == "Aurora Theme"
        ));
        assert!(matches!(
            parse_command("set-export-enabled|1|true"),
            Ok(Intent::SetExportEnabled(1, true))
        ));
        assert!(matches!(
            parse_command("set-export-output|0|exports/demo.toml"),
            Ok(Intent::SetExportOutputPath(0, path)) if path.display().to_string() == "exports/demo.toml"
        ));
        assert!(matches!(
            parse_command("set-export-template|1|templates/demo.txt"),
            Ok(Intent::SetExportTemplatePath(1, path)) if path.display().to_string() == "templates/demo.txt"
        ));

        let project_path = match parse_command("set-editor-text|project_path|projects/demo.toml") {
            Ok(Intent::SetEditorProjectPath(path)) => path,
            other => panic!("expected editor project path intent, got {other:?}"),
        };
        assert_eq!(project_path.display().to_string(), "projects/demo.toml");

        assert!(matches!(
            parse_command("set-editor-toggle|auto_load_project_on_startup|true"),
            Ok(Intent::SetEditorAutoLoadProject(true))
        ));
        assert!(matches!(
            parse_command("set-editor-toggle|auto_save_project_on_export|false"),
            Ok(Intent::SetEditorAutoSaveOnExport(false))
        ));
        assert!(matches!(
            parse_command("set-editor-choice|startup_focus|inspector"),
            Ok(Intent::SetEditorStartupFocus(FocusPane::Inspector))
        ));
    }
}
