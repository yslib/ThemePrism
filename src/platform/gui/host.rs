use crate::platform::gui::bootstrap::GuiBootstrap;
use crate::platform::{PlatformError, PlatformKind};

#[allow(dead_code)]
pub trait GuiHost {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Gui
    }

    fn run(&self, bootstrap: GuiBootstrap) -> Result<(), PlatformError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PendingGuiHost;

impl GuiHost for PendingGuiHost {
    fn run(&self, _bootstrap: GuiBootstrap) -> Result<(), PlatformError> {
        Err(PlatformError::Unavailable {
            kind: self.kind(),
            reason: "native GUI event loop is not implemented yet",
        })
    }
}
