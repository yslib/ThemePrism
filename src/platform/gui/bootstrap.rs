use crate::app::AppState;
use crate::app::view::ViewTree;
use crate::platform::PlatformKind;
use crate::platform::gui::event_adapter::GuiEventAdapter;
use crate::platform::gui::renderer::{GuiRenderer, GuiScene, GuiWindowConfig};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GuiBootstrap {
    pub kind: PlatformKind,
    pub state: AppState,
    pub window: GuiWindowConfig,
    pub initial_scene: GuiScene,
    pub event_adapter: GuiEventAdapter,
    pub renderer: GuiRenderer,
}

impl GuiBootstrap {
    pub fn new(state: AppState, view: &ViewTree) -> Self {
        let renderer = GuiRenderer;
        let initial_scene = renderer.build_scene(view);

        Self {
            kind: PlatformKind::Gui,
            state,
            window: GuiWindowConfig::default(),
            initial_scene,
            event_adapter: GuiEventAdapter,
            renderer,
        }
    }
}
