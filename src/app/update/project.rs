use std::path::PathBuf;

use crate::app::effect::{Effect, ProjectData};
use crate::app::interaction::SurfaceId;
use crate::app::state::AppState;
use crate::domain::params::ThemeParams;
use crate::domain::rules::RuleSet;
use crate::export::default_export_profiles;
use crate::i18n::{self, UiText};

use super::{modals, tr, tr1};

pub(super) fn reset_state(state: &mut AppState) {
    state.domain.params = ThemeParams::default();
    state.domain.rules = RuleSet::default();
    state.project.name = "Untitled Theme".to_string();
    state.project.export_profiles = default_export_profiles();
    state.ui.selected_token = 0;
    state.ui.selected_param = 0;
    state.ui.inspector_field = 0;
    modals::close_text_input_surface(state);
    modals::close_source_picker_surface(state);
    modals::close_config_surface(state);
    modals::close_shortcut_help_surface(state);
    state.ui.fullscreen_surface = None;
    state.preview.capture_active = false;
    modals::pop_capture_owner(state, SurfaceId::PreviewBody);
    match state.recompute() {
        Ok(()) => state.ui.status = tr(state, UiText::StatusResetDefaults),
        Err(err) => state.ui.status = tr1(state, UiText::StatusResetFailed, "error", err),
    }
}

pub(super) fn save_project_effect(state: &AppState) -> Effect {
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

pub(super) fn set_project_name(state: &mut AppState, name: String) -> Vec<Effect> {
    let value = name.trim();
    if value.is_empty() {
        state.ui.status = tr(state, UiText::ErrorProjectNameEmpty);
        return Vec::new();
    }

    state.project.name = value.to_string();
    state.ui.status = tr1(
        state,
        UiText::StatusProjectNameUpdated,
        "name",
        &state.project.name,
    );
    Vec::new()
}

pub(super) fn set_export_enabled(state: &mut AppState, index: usize, enabled: bool) -> Vec<Effect> {
    let locale = state.locale();
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => {
            profile.enabled = enabled;
            state.ui.status = if enabled {
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
        }
        None => {
            state.ui.status = i18n::format1(
                locale,
                UiText::StatusMissingExportTarget,
                "index",
                index + 1,
            );
        }
    }
    Vec::new()
}

pub(super) fn set_export_output_path(
    state: &mut AppState,
    index: usize,
    path: PathBuf,
) -> Vec<Effect> {
    let locale = state.locale();
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => {
            profile.output_path = path;
            state.ui.status = i18n::format2(
                locale,
                UiText::StatusExportOutputUpdated,
                "name",
                &profile.name,
                "path",
                profile.output_path.display(),
            );
        }
        None => {
            state.ui.status = i18n::format1(
                locale,
                UiText::StatusMissingExportTarget,
                "index",
                index + 1,
            );
        }
    }
    Vec::new()
}

pub(super) fn set_export_template_path(
    state: &mut AppState,
    index: usize,
    path: PathBuf,
) -> Vec<Effect> {
    let locale = state.locale();
    match state.project.export_profiles.get_mut(index) {
        Some(profile) => {
            profile.set_template_path(path);
            state.ui.status = i18n::format2(
                locale,
                UiText::StatusExportTemplateUpdated,
                "name",
                &profile.name,
                "path",
                profile.configured_template_path().display(),
            );
        }
        None => {
            state.ui.status = i18n::format1(
                locale,
                UiText::StatusMissingExportTarget,
                "index",
                index + 1,
            );
        }
    }
    Vec::new()
}
