use crate::app::AppState;
use crate::app::build_view;
use crate::platform::gui::bootstrap::GuiBootstrap;
use crate::platform::gui::host::{GuiHost, PendingGuiHost};
use crate::platform::{PlatformError, PlatformKind, PlatformRuntime};

#[derive(Debug, Default, Clone, Copy)]
pub struct GuiPlatform;

impl PlatformRuntime for GuiPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Gui
    }

    fn launch(&self, state: AppState) -> Result<(), PlatformError> {
        let view = build_view(&state);
        let bootstrap = GuiBootstrap::new(state, &view);
        self.host().run(bootstrap)
    }
}

impl GuiPlatform {
    fn host(&self) -> impl GuiHost {
        PendingGuiHost
    }
}
