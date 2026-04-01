use crate::app::interaction::SurfaceId;
use crate::app::state::AppState;
use crate::domain::preview::{PreviewFrame, PreviewRuntimeEvent};
use crate::i18n::{self, UiText};
use crate::preview::PreviewMode;

use super::{modals, tr, tr1};

pub(super) fn cycle_preview_mode(state: &mut AppState, delta: i32) {
    let next_mode = if delta >= 0 {
        state.preview.active_mode.next()
    } else {
        state.preview.active_mode.previous()
    };
    set_preview_mode(state, next_mode);
}

pub(super) fn set_preview_mode(state: &mut AppState, mode: PreviewMode) {
    if state.preview.active_mode == mode {
        return;
    }

    state.preview.active_mode = mode;
    state.preview.capture_active = false;
    modals::pop_capture_owner(state, SurfaceId::PreviewBody);
    state.preview.runtime_status.clear();
    if state.preview.active_mode.is_runtime_backed() {
        state.preview.runtime_frame = PreviewFrame::placeholder(
            tr(state, UiText::PreviewWaitingTitle),
            tr(state, UiText::PreviewWaitingDetail),
        );
        state.preview.runtime_status = tr(state, UiText::PreviewWaitingDetail);
    }
    state.ui.status = tr1(
        state,
        UiText::StatusPreviewModeChanged,
        "mode",
        i18n::preview_mode_label(state.locale(), state.preview.active_mode),
    );
}

pub(super) fn set_preview_capture(state: &mut AppState, active: bool) {
    if active && !state.preview.active_mode.is_interactive() {
        return;
    }

    state.preview.capture_active = active;
    if active {
        modals::push_capture_owner(state, SurfaceId::PreviewBody);
    } else {
        modals::pop_capture_owner(state, SurfaceId::PreviewBody);
    }
    state.ui.status = if active {
        tr1(
            state,
            UiText::StatusPreviewCaptureActive,
            "mode",
            i18n::preview_mode_label(state.locale(), state.preview.active_mode),
        )
    } else {
        tr(state, UiText::StatusPreviewCaptureReleased)
    };
}

pub(super) fn apply_preview_runtime_event(state: &mut AppState, event: PreviewRuntimeEvent) {
    match event {
        PreviewRuntimeEvent::FrameUpdated(frame) => {
            state.preview.runtime_frame = frame;
            state.preview.runtime_status.clear();
        }
        PreviewRuntimeEvent::StatusUpdated(status) => {
            state.preview.runtime_status = status;
        }
        PreviewRuntimeEvent::Exited { message } => {
            state.preview.capture_active = false;
            modals::pop_capture_owner(state, SurfaceId::PreviewBody);
            state.preview.runtime_status = tr1(
                state,
                UiText::StatusPreviewProcessExited,
                "message",
                &message,
            );
            state.preview.runtime_frame =
                PreviewFrame::error(tr(state, UiText::PreviewExitedTitle), &message);
            state.ui.status = state.preview.runtime_status.clone();
        }
    }
}
