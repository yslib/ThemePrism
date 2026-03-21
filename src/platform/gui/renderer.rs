#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GuiRenderer;

use crate::app::view::ViewTree;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiWindowConfig {
    pub title: &'static str,
    pub min_width: u16,
    pub min_height: u16,
}

impl Default for GuiWindowConfig {
    fn default() -> Self {
        Self {
            title: "Theme Generator",
            min_width: 1080,
            min_height: 720,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiScene {
    pub root: &'static str,
    pub overlays: usize,
}

impl GuiRenderer {
    pub fn build_scene(self, tree: &ViewTree) -> GuiScene {
        let _ = tree;
        GuiScene {
            root: "view_tree_root",
            overlays: tree.overlays.len(),
        }
    }
}
