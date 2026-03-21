mod app;
mod domain;
mod export;
mod persistence;
mod platform;

pub use domain::{color, evaluator, palette, params, preview, rules, tokens};

use std::error::Error;

use crate::app::AppState;
use crate::platform::launch_from_env;

fn main() -> Result<(), Box<dyn Error>> {
    let state = AppState::new()?;
    launch_from_env(state)?;
    Ok(())
}
