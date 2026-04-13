mod app;
mod branding;
mod core;
mod domain;
mod enum_meta;
mod export;
mod i18n;
mod persistence;
mod platform;

pub use domain::{color, evaluator, palette, params, preview, rules, tokens};

use std::error::Error;

use crate::core::AppState;
use crate::platform::{PlatformError, run_entrypoint};

fn main() -> Result<(), Box<dyn Error>> {
    run_entrypoint(|| AppState::new().map_err(|err| PlatformError::StateInit(err.to_string())))?;
    Ok(())
}
