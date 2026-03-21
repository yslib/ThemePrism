use crate::core::CoreSession;
use crate::platform::gui::bootstrap::GuiBootstrap;
use crate::platform::gui::host::GuiHost;
#[cfg(not(target_os = "macos"))]
use crate::platform::gui::host::PendingGuiHost;
#[cfg(target_os = "macos")]
use crate::platform::gui::macos::MacOsAppKitHost;
use crate::platform::{PlatformError, PlatformKind, PlatformRuntime};

#[derive(Debug, Default, Clone, Copy)]
pub struct GuiPlatform;

impl PlatformRuntime for GuiPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Gui
    }

    fn launch(&self, session: CoreSession) -> Result<(), PlatformError> {
        let view = session.view_tree();
        let bootstrap = GuiBootstrap::new(session, &view);
        self.host().run(bootstrap)
    }
}

impl GuiPlatform {
    fn host(&self) -> Box<dyn GuiHost> {
        #[cfg(target_os = "macos")]
        {
            Box::new(MacOsAppKitHost)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Box::new(PendingGuiHost)
        }
    }
}
