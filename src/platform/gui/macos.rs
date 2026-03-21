use std::ffi::{CStr, CString, c_char, c_void};

use crate::platform::gui::bootstrap::GuiBootstrap;
use crate::platform::gui::bridge::GuiBridgeSession;
use crate::platform::gui::host::GuiHost;
use crate::platform::{PlatformError, PlatformKind};

#[derive(Debug, Default, Clone, Copy)]
pub struct MacOsAppKitHost;

impl GuiHost for MacOsAppKitHost {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Gui
    }

    fn run(&self, bootstrap: GuiBootstrap) -> Result<(), PlatformError> {
        let mut session = Box::new(GuiBridgeSession::from_core(bootstrap.session));
        let result = unsafe {
            theme_gui_host_run(
                (&mut *session) as *mut GuiBridgeSession as *mut c_void,
                bridge_copy_snapshot,
                bridge_dispatch_command,
                bridge_free_string,
            )
        };
        drop(session);

        if result == 0 {
            Ok(())
        } else {
            Err(PlatformError::Unavailable {
                kind: self.kind(),
                reason: "AppKit host failed to launch",
            })
        }
    }
}

unsafe extern "C" {
    fn theme_gui_host_run(
        context: *mut c_void,
        copy_snapshot: extern "C" fn(*mut c_void) -> *mut c_char,
        dispatch_command: extern "C" fn(*mut c_void, *const c_char),
        free_string: extern "C" fn(*mut c_char),
    ) -> i32;
}

extern "C" fn bridge_copy_snapshot(context: *mut c_void) -> *mut c_char {
    let session = unsafe { &mut *(context as *mut GuiBridgeSession) };
    CString::new(session.snapshot_json())
        .expect("snapshot JSON should not contain interior nul bytes")
        .into_raw()
}

extern "C" fn bridge_dispatch_command(context: *mut c_void, command: *const c_char) {
    if command.is_null() {
        return;
    }

    let session = unsafe { &mut *(context as *mut GuiBridgeSession) };
    let command = unsafe { CStr::from_ptr(command) };
    if let Ok(command) = command.to_str() {
        session.dispatch(command);
    }
}

extern "C" fn bridge_free_string(value: *mut c_char) {
    if value.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(value);
    }
}
