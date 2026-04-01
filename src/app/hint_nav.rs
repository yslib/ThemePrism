use crate::app::interaction::{InteractionMode, SurfaceId};
use crate::app::state::AppState;
use crate::app::ui_meta::{preview_mode_spec, workspace_tab_spec};
use crate::app::view::{panel_order, workspace_layout_for_tab};
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::preview::PreviewMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintTarget {
    Panel { label: char, panel: PanelId },
    WorkspaceTab { label: char, tab: WorkspaceTab },
    PreviewTab { label: char, mode: PreviewMode },
}

impl HintTarget {
    pub const fn label(self) -> char {
        match self {
            Self::Panel { label, .. }
            | Self::WorkspaceTab { label, .. }
            | Self::PreviewTab { label, .. } => label,
        }
    }
}

pub fn main_window_hint_targets(state: &AppState) -> Vec<HintTarget> {
    let mut targets = Vec::new();
    let layout = workspace_layout_for_tab(state.ui.active_tab);
    let panels = panel_order(&layout);

    for (index, panel) in panels.iter().copied().enumerate() {
        let Ok(number) = u8::try_from(index + 1) else {
            continue;
        };
        if !(1..=9).contains(&number) {
            continue;
        }
        targets.push(HintTarget::Panel {
            label: char::from(b'0' + number),
            panel,
        });
    }

    let mut next_letter = b'a';

    for tab in WorkspaceTab::ALL.iter().copied() {
        let Some(spec) = workspace_tab_spec(tab) else {
            continue;
        };
        if !spec.hint_navigation {
            continue;
        }
        targets.push(HintTarget::WorkspaceTab {
            label: char::from(next_letter),
            tab: spec.id,
        });
        next_letter = next_letter.saturating_add(1);
    }

    if panels.contains(&PanelId::Preview) {
        for mode in PreviewMode::ALL.iter().copied() {
            let Some(spec) = preview_mode_spec(mode) else {
                continue;
            };
            if !spec.hint_navigation {
                continue;
            }
            targets.push(HintTarget::PreviewTab {
                label: char::from(next_letter),
                mode: spec.id,
            });
            next_letter = next_letter.saturating_add(1);
        }
    }

    targets
}

pub fn workspace_tab_hint_label(state: &AppState, tab: WorkspaceTab) -> Option<char> {
    if !is_main_window_hint_mode(state) {
        return None;
    }

    main_window_hint_targets(state)
        .into_iter()
        .find_map(|target| match target {
            HintTarget::WorkspaceTab {
                label,
                tab: candidate,
            } if candidate == tab => Some(label),
            _ => None,
        })
}

pub fn preview_tab_hint_label(state: &AppState, mode: PreviewMode) -> Option<char> {
    if !is_main_window_hint_mode(state) {
        return None;
    }

    main_window_hint_targets(state)
        .into_iter()
        .find_map(|target| match target {
            HintTarget::PreviewTab {
                label,
                mode: candidate,
            } if candidate == mode => Some(label),
            _ => None,
        })
}

fn is_main_window_hint_mode(state: &AppState) -> bool {
    matches!(
        state.ui.interaction.current_mode(),
        InteractionMode::NavigateScope(SurfaceId::MainWindow)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        HintTarget, main_window_hint_targets, preview_tab_hint_label, workspace_tab_hint_label,
    };
    use crate::app::AppState;
    use crate::app::interaction::{InteractionMode, SurfaceId};
    use crate::app::ui_meta::{preview_mode_spec, workspace_tab_spec};
    use crate::app::workspace::{PanelId, WorkspaceTab};
    use crate::preview::PreviewMode;

    #[test]
    fn main_window_hint_targets_flatten_visible_panels_and_tabs() {
        let state = AppState::new().expect("state");
        let targets = main_window_hint_targets(&state);

        assert!(targets.contains(&HintTarget::Panel {
            label: '1',
            panel: PanelId::Tokens,
        }));
        assert!(
            targets.contains(&HintTarget::WorkspaceTab {
                label: 'a',
                tab: workspace_tab_spec(WorkspaceTab::Theme)
                    .expect("theme workspace tab spec")
                    .id,
            })
        );
        assert!(
            targets.contains(&HintTarget::WorkspaceTab {
                label: 'b',
                tab: workspace_tab_spec(WorkspaceTab::Project)
                    .expect("project workspace tab spec")
                    .id,
            })
        );
        assert!(
            targets.contains(&HintTarget::PreviewTab {
                label: 'c',
                mode: preview_mode_spec(PreviewMode::Code)
                    .expect("code preview mode spec")
                    .id,
            })
        );
        assert!(
            targets.contains(&HintTarget::PreviewTab {
                label: 'd',
                mode: preview_mode_spec(PreviewMode::Shell)
                    .expect("shell preview mode spec")
                    .id,
            })
        );
        assert!(
            targets.contains(&HintTarget::PreviewTab {
                label: 'e',
                mode: preview_mode_spec(PreviewMode::Lazygit)
                    .expect("lazygit preview mode spec")
                    .id,
            })
        );
    }

    #[test]
    fn project_tab_hints_do_not_include_preview_tabs() {
        let mut state = AppState::new().expect("state");
        state.ui.active_tab = WorkspaceTab::Project;
        state
            .ui
            .interaction
            .set_mode(InteractionMode::NavigateScope(SurfaceId::MainWindow));

        assert_eq!(
            workspace_tab_hint_label(&state, WorkspaceTab::Theme),
            Some('a')
        );
        assert_eq!(
            workspace_tab_hint_label(&state, WorkspaceTab::Project),
            Some('b')
        );
        assert_eq!(preview_tab_hint_label(&state, PreviewMode::Code), None);
    }
}
