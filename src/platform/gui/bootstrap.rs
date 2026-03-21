use crate::app::view::ViewTree;
use crate::core::CoreSession;
use crate::platform::PlatformKind;
use crate::platform::gui::event_adapter::GuiEventAdapter;
use crate::platform::gui::renderer::{GuiRenderer, GuiScene, GuiWindowConfig};

#[allow(dead_code)]
#[derive(Debug)]
pub struct GuiBootstrap {
    pub kind: PlatformKind,
    pub session: CoreSession,
    pub window: GuiWindowConfig,
    pub initial_scene: GuiScene,
    pub event_adapter: GuiEventAdapter,
    pub renderer: GuiRenderer,
}

impl GuiBootstrap {
    pub fn new(session: CoreSession, view: &ViewTree) -> Self {
        let renderer = GuiRenderer;
        let initial_scene = renderer.build_scene(view);

        Self {
            kind: PlatformKind::Gui,
            session,
            window: GuiWindowConfig::default(),
            initial_scene,
            event_adapter: GuiEventAdapter,
            renderer,
        }
    }
}
