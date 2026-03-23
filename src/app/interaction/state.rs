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
    pub mode: InteractionMode,
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
            mode: InteractionMode::Normal,
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
        self.mode
    }

    pub fn push_mode(&mut self, mode: InteractionMode) {
        self.mode_stack.push(mode);
        self.mode = mode;
    }

    pub fn pop_mode(&mut self) -> Option<InteractionMode> {
        if self.mode_stack.len() <= 1 {
            return None;
        }

        let popped = self.mode_stack.pop();
        if matches!(
            popped,
            Some(InteractionMode::Capture { .. } | InteractionMode::Modal { .. })
        ) && self.focus_path.len() > 2
        {
            self.focus_path.pop();
        }
        self.mode = self.current_mode();

        popped
    }

    pub fn set_mode(&mut self, mode: InteractionMode) {
        self.mode_stack.clear();
        self.mode_stack.push(InteractionMode::Normal);
        if mode != InteractionMode::Normal {
            self.mode_stack.push(mode);
        }
        self.mode = mode;
    }

    pub fn has_mode_for(&self, surface: SurfaceId) -> bool {
        self.mode_stack
            .iter()
            .any(|mode| matches!(mode, InteractionMode::NavigateChildren(target) if *target == surface)
                || matches!(mode, InteractionMode::Capture { owner } if *owner == surface)
                || matches!(mode, InteractionMode::Modal { owner } if *owner == surface))
    }

    pub fn focus_root(&mut self) {
        self.focus_path.clear();
        self.focus_path.push(SurfaceId::AppRoot);
        self.focus_path.push(SurfaceId::MainWindow);
        self.set_mode(InteractionMode::Normal);
    }

    pub fn focus_panel(&mut self, panel: PanelId) {
        self.focus_path.clear();
        self.focus_path.push(SurfaceId::AppRoot);
        self.focus_path.push(SurfaceId::MainWindow);
        self.focus_path.push(SurfaceId::workspace_surface(panel));
        self.set_mode(InteractionMode::Normal);
    }
}

impl From<PanelId> for SurfaceId {
    fn from(panel: PanelId) -> Self {
        SurfaceId::workspace_surface(panel)
    }
}
