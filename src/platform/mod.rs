pub mod gui;
pub mod tui;

use std::env;
use std::error::Error;
use std::fmt;

use crate::app::AppState;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Tui,
    Gui,
}

impl PlatformKind {
    pub const DEFAULT: Self = Self::Tui;

    pub const fn label(self) -> &'static str {
        match self {
            Self::Tui => "TUI",
            Self::Gui => "GUI",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PlatformDescriptor {
    pub kind: PlatformKind,
    pub enabled: bool,
    pub summary: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchOptions {
    pub platform: PlatformKind,
}

#[derive(Debug)]
pub enum PlatformError {
    InvalidArgs(String),
    Unavailable {
        kind: PlatformKind,
        reason: &'static str,
    },
    Runtime {
        kind: PlatformKind,
        message: String,
    },
}

impl PlatformError {
    pub fn runtime(kind: PlatformKind, message: impl Into<String>) -> Self {
        Self::Runtime {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgs(message) => f.write_str(message),
            Self::Unavailable { kind, reason } => {
                write!(f, "{} platform is not enabled: {reason}", kind.label())
            }
            Self::Runtime { kind, message } => {
                write!(f, "{} platform failed: {message}", kind.label())
            }
        }
    }
}

impl Error for PlatformError {}

pub trait PlatformRuntime {
    fn kind(&self) -> PlatformKind;
    fn launch(&self, state: AppState) -> Result<(), PlatformError>;
}

pub fn launch_from_env(state: AppState) -> Result<(), PlatformError> {
    let options = parse_launch_options(env::args().skip(1))?;
    launch(state, options)
}

pub fn launch(state: AppState, options: LaunchOptions) -> Result<(), PlatformError> {
    build_runtime(options.platform).launch(state)
}

#[allow(dead_code)]
pub fn launch_default(state: AppState) -> Result<(), PlatformError> {
    launch(
        state,
        LaunchOptions {
            platform: PlatformKind::DEFAULT,
        },
    )
}

#[allow(dead_code)]
pub fn build_runtime(kind: PlatformKind) -> Box<dyn PlatformRuntime> {
    match kind {
        PlatformKind::Tui => Box::new(tui::runtime::TuiPlatform),
        PlatformKind::Gui => Box::new(gui::runtime::GuiPlatform),
    }
}

#[allow(dead_code)]
pub fn registered_platforms() -> &'static [PlatformDescriptor] {
    &[
        PlatformDescriptor {
            kind: PlatformKind::Tui,
            enabled: true,
            summary: "Crossterm + ratatui runtime backed by the shared app core.",
        },
        PlatformDescriptor {
            kind: PlatformKind::Gui,
            enabled: false,
            summary: "Reserved native GUI platform slot with matching runtime shape.",
        },
    ]
}

pub fn parse_launch_options(
    args: impl IntoIterator<Item = String>,
) -> Result<LaunchOptions, PlatformError> {
    let mut selected = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(PlatformError::InvalidArgs(usage_text())),
            "--platform" => {
                let value = args.next().ok_or_else(|| {
                    PlatformError::InvalidArgs(
                        "missing value for --platform\n\n".to_string() + &usage_text(),
                    )
                })?;
                let kind = parse_platform_kind(&value)?;
                set_platform(&mut selected, kind)?;
            }
            "--tui" | "tui" => set_platform(&mut selected, PlatformKind::Tui)?,
            "--gui" | "gui" => set_platform(&mut selected, PlatformKind::Gui)?,
            other if other.starts_with('-') => {
                return Err(PlatformError::InvalidArgs(format!(
                    "unknown option: {other}\n\n{}",
                    usage_text()
                )));
            }
            other => {
                return Err(PlatformError::InvalidArgs(format!(
                    "unknown argument: {other}\n\n{}",
                    usage_text()
                )));
            }
        }
    }

    Ok(LaunchOptions {
        platform: selected.unwrap_or(PlatformKind::DEFAULT),
    })
}

fn parse_platform_kind(value: &str) -> Result<PlatformKind, PlatformError> {
    match value {
        "tui" | "TUI" => Ok(PlatformKind::Tui),
        "gui" | "GUI" => Ok(PlatformKind::Gui),
        _ => Err(PlatformError::InvalidArgs(format!(
            "unsupported platform: {value}\n\n{}",
            usage_text()
        ))),
    }
}

fn set_platform(
    selected: &mut Option<PlatformKind>,
    next: PlatformKind,
) -> Result<(), PlatformError> {
    match selected {
        Some(current) if *current != next => Err(PlatformError::InvalidArgs(format!(
            "conflicting platform options: {} and {}\n\n{}",
            current.label(),
            next.label(),
            usage_text()
        ))),
        Some(_) => Ok(()),
        None => {
            *selected = Some(next);
            Ok(())
        }
    }
}

fn usage_text() -> String {
    let default = PlatformKind::DEFAULT.label().to_ascii_lowercase();
    format!(
        "Usage: theme [--platform <tui|gui>] [--tui|--gui]\n\
         \n\
         No arguments: start {default}\n\
         --platform tui: explicitly start the TUI runtime\n\
         --platform gui: start the GUI runtime when it becomes available"
    )
}

#[cfg(test)]
mod tests {
    use super::{PlatformError, PlatformKind, parse_launch_options};

    fn parse(args: &[&str]) -> Result<PlatformKind, PlatformError> {
        parse_launch_options(args.iter().map(|arg| (*arg).to_string()))
            .map(|options| options.platform)
    }

    #[test]
    fn defaults_to_tui_without_args() {
        assert_eq!(parse(&[]).unwrap(), PlatformKind::Tui);
    }

    #[test]
    fn parses_explicit_platform_flag() {
        assert_eq!(parse(&["--platform", "gui"]).unwrap(), PlatformKind::Gui);
        assert_eq!(parse(&["--platform", "tui"]).unwrap(), PlatformKind::Tui);
    }

    #[test]
    fn parses_shortcut_flags_and_positionals() {
        assert_eq!(parse(&["--gui"]).unwrap(), PlatformKind::Gui);
        assert_eq!(parse(&["tui"]).unwrap(), PlatformKind::Tui);
    }

    #[test]
    fn rejects_unknown_platform() {
        let err = parse(&["--platform", "web"]).unwrap_err().to_string();
        assert!(err.contains("unsupported platform"));
    }

    #[test]
    fn rejects_conflicting_platform_options() {
        let err = parse(&["--tui", "--gui"]).unwrap_err().to_string();
        assert!(err.contains("conflicting platform options"));
    }
}
