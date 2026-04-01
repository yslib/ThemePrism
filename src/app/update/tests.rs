use super::update;
use crate::app::intent::Intent;
use crate::app::state::{
    AppState, ConfigFieldId, ConfigModalState, TextInputState, TextInputTarget,
};
use crate::app::workspace::PanelId;
use crate::domain::rules::{Rule, RuleKind};
use crate::domain::tokens::TokenRole;
use crate::preview::PreviewMode;

#[test]
fn navigation_intent_routes_through_public_update_entrypoint() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::FocusPanelByNumber(2));

    assert_eq!(state.active_panel(), PanelId::Params);
}

#[test]
fn preview_intent_updates_active_preview_mode() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::SetPreviewMode(PreviewMode::Shell));

    assert_eq!(state.preview.active_mode, PreviewMode::Shell);
}

#[test]
fn modal_intent_toggles_shortcut_help_state() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::ToggleShortcutHelpRequested);

    assert!(state.ui.shortcut_help_open);
}

#[test]
fn config_intents_open_modal_and_move_selection() {
    let mut state = AppState::new().expect("state should build");

    update(&mut state, Intent::OpenConfigRequested);
    update(&mut state, Intent::MoveConfigSelection(1));

    assert_eq!(
        state.ui.config_modal,
        Some(ConfigModalState { selected_field: 1 })
    );
}

#[test]
fn project_intent_updates_project_name() {
    let mut state = AppState::new().expect("state should build");

    update(
        &mut state,
        Intent::SetProjectName("Aurora Theme".to_string()),
    );

    assert_eq!(state.project.name, "Aurora Theme");
}

#[test]
fn inspector_intent_updates_rule_kind_for_selected_role() {
    let mut state = AppState::new().expect("state should build");

    update(
        &mut state,
        Intent::SetRuleKind(TokenRole::Background, RuleKind::Fixed),
    );

    assert!(matches!(
        state.domain.rules.get(TokenRole::Background),
        Some(Rule::Fixed { .. })
    ));
}

#[test]
fn text_input_intent_appends_to_seeded_buffer() {
    let mut state = AppState::new().expect("state should build");
    state.ui.text_input = Some(TextInputState {
        target: TextInputTarget::Config(ConfigFieldId::ProjectName),
        buffer: String::new(),
    });

    update(&mut state, Intent::AppendTextInput('a'));

    assert_eq!(
        state
            .ui
            .text_input
            .as_ref()
            .map(|input| input.buffer.as_str()),
        Some("a")
    );
}
