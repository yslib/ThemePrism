use crate::app::workspace::PanelId;

use super::SurfaceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Normal,
    NavigateChildren(SurfaceId),
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
}

impl InteractionState {
    pub fn new(initial_panel: PanelId) -> Self {
        Self {
            focus_path: vec![
                SurfaceId::AppRoot,
                SurfaceId::MainWindow,
                SurfaceId::workspace_surface(initial_panel),
            ],
            mode: InteractionMode::Normal,
        }
    }

    pub fn focused_workspace_surface(&self) -> SurfaceId {
        self.focus_path
            .last()
            .copied()
            .unwrap_or(SurfaceId::MainWindow)
    }

    pub fn focus_root(&mut self) {
        self.focus_path.clear();
        self.focus_path.push(SurfaceId::AppRoot);
        self.focus_path.push(SurfaceId::MainWindow);
        self.mode = InteractionMode::Normal;
    }

    pub fn focus_panel(&mut self, panel: PanelId) {
        self.focus_path.clear();
        self.focus_path.push(SurfaceId::AppRoot);
        self.focus_path.push(SurfaceId::MainWindow);
        self.focus_path.push(SurfaceId::workspace_surface(panel));
        self.mode = InteractionMode::Normal;
    }
}
