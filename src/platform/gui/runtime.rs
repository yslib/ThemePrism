use crate::app::AppState;
use crate::platform::{PlatformError, PlatformKind, PlatformRuntime};

#[derive(Debug, Default, Clone, Copy)]
pub struct GuiPlatform;

impl PlatformRuntime for GuiPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Gui
    }

    fn launch(&self, _state: AppState) -> Result<(), PlatformError> {
        Err(PlatformError::Unavailable {
            kind: self.kind(),
            reason: "native GUI event adapter and renderer are not implemented yet",
        })
    }
}
