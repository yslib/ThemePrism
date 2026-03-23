use crate::app::workspace::PanelId;

use super::SurfaceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Normal,
    NavigateChildren(SurfaceId),
    Capture { owner: SurfaceId },
    Modal { owner: SurfaceId },
}

impl Default for InteractionMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionState {
    pub focus_path: Vec<SurfaceId>,
    mode_stack: Vec<InteractionMode>,
}

impl InteractionState {
    pub fn new(initial_surface: impl Into<SurfaceId>) -> Self {
        let initial_surface = initial_surface.into();
        let initial_panel = initial_surface.panel_id().unwrap_or(PanelId::Tokens);
        Self {
            focus_path: vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::workspace_surface(initial_panel),
            ],
            mode_stack: vec![InteractionMode::Normal],
        }
    }

    pub fn focused_surface(&self) -> SurfaceId {
        self.focus_path
            .last()
            .copied()
            .unwrap_or(SurfaceId::MainWindow)
    }

    pub fn focused_workspace_surface(&self) -> SurfaceId {
        self.focused_surface()
    }

    pub fn current_mode(&self) -> InteractionMode {
        self.mode_stack
            .last()
            .copied()
            .unwrap_or(InteractionMode::Normal)
    }

    pub fn current_owner(&self) -> Option<SurfaceId> {
        owner_for_mode(self.current_mode())
    }

    pub fn push_mode(&mut self, mode: InteractionMode) {
        self.mode_stack.push(mode);
    }

    pub fn pop_mode(&mut self) -> Option<InteractionMode> {
        if self.mode_stack.len() <= 1 {
            return None;
        }

        let popped = self.mode_stack.pop();
        if owner_for_mode(popped.unwrap_or(InteractionMode::Normal))
            .is_some_and(|owner| self.focus_path.last().copied() == Some(owner))
            && self.focus_path.len() > 2
        {
            self.focus_path.pop();
        }

        popped
    }

    pub fn remove_mode(&mut self, mode: InteractionMode) -> bool {
        if mode == InteractionMode::Normal {
            return false;
        }

        let Some(index) = self.mode_stack.iter().rposition(|entry| *entry == mode) else {
            return false;
        };

        let removed = self.mode_stack.remove(index);
        if index == self.mode_stack.len()
            && owner_for_mode(removed)
                .is_some_and(|owner| self.focus_path.last().copied() == Some(owner))
            && self.focus_path.len() > 2
        {
            self.focus_path.pop();
        }

        true
    }

    pub fn set_mode(&mut self, mode: InteractionMode) {
        self.mode_stack.clear();
        self.mode_stack.push(InteractionMode::Normal);
        if mode != InteractionMode::Normal {
            self.mode_stack.push(mode);
        }
    }

    pub fn has_mode_for(&self, surface: SurfaceId) -> bool {
        self.mode_stack
            .iter()
            .any(|mode| matches!(mode, InteractionMode::NavigateChildren(target) if *target == surface)
                || matches!(mode, InteractionMode::Capture { owner } if *owner == surface)
                || matches!(mode, InteractionMode::Modal { owner } if *owner == surface))
    }

    pub fn focus_root(&mut self) {
        self.set_focus_root_path();
        self.set_mode(InteractionMode::Normal);
    }

    pub fn set_focus_root_path(&mut self) {
        self.focus_path.clear();
        self.focus_path.push(SurfaceId::AppRoot);
        self.focus_path.push(SurfaceId::MainWindow);
    }

    pub fn focus_panel(&mut self, panel: PanelId) {
        self.set_focus_panel_path(panel);
        self.set_mode(InteractionMode::Normal);
    }

    pub fn set_focus_panel_path(&mut self, panel: PanelId) {
        self.focus_path.clear();
        self.focus_path.push(SurfaceId::AppRoot);
        self.focus_path.push(SurfaceId::MainWindow);
        self.focus_path.push(SurfaceId::workspace_surface(panel));
    }
}

fn owner_for_mode(mode: InteractionMode) -> Option<SurfaceId> {
    match mode {
        InteractionMode::Capture { owner } | InteractionMode::Modal { owner } => Some(owner),
        InteractionMode::Normal | InteractionMode::NavigateChildren(_) => None,
    }
}

impl From<PanelId> for SurfaceId {
    fn from(panel: PanelId) -> Self {
        SurfaceId::workspace_surface(panel)
    }
}

#[cfg(test)]
mod tests {
    use super::{InteractionMode, InteractionState, SurfaceId};

    #[test]
    fn interaction_state_shape_stores_modes_only_in_the_stack() {
        let state = InteractionState {
            focus_path: vec![SurfaceId::AppRoot, SurfaceId::MainWindow],
            mode_stack: vec![InteractionMode::Normal],
        };

        assert_eq!(state.current_mode(), InteractionMode::Normal);
    }

    #[test]
    fn remove_mode_can_remove_non_top_owner_without_changing_focus() {
        let mut state = InteractionState {
            focus_path: vec![SurfaceId::AppRoot, SurfaceId::MainWindow],
            mode_stack: vec![
                InteractionMode::Normal,
                InteractionMode::Capture {
                    owner: SurfaceId::PreviewBody,
                },
                InteractionMode::Modal {
                    owner: SurfaceId::ConfigDialog,
                },
            ],
        };

        assert!(state.remove_mode(InteractionMode::Capture {
            owner: SurfaceId::PreviewBody,
        }));
        assert_eq!(
            state.current_mode(),
            InteractionMode::Modal {
                owner: SurfaceId::ConfigDialog,
            }
        );
        assert_eq!(state.focus_path, vec![SurfaceId::AppRoot, SurfaceId::MainWindow]);
    }
}
