pub mod gui;
pub mod tui;

use std::fmt;
use std::io::{self, Write};

use clap::{ArgAction, Parser, ValueEnum, error::ErrorKind};
use thiserror::Error;

use crate::core::{AppState, CoreSession};
use crate::persistence::editor_config::load_editor_config;
use crate::persistence::project_file::load_project;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
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

impl fmt::Display for PlatformKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchCommand {
    Run(LaunchOptions),
    PrintHelp(String),
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "theme",
    about = "Generate terminal/editor themes from a shared color system.",
    long_about = None,
    disable_version_flag = true,
    after_help = "No arguments: start tui.\n\
--platform tui: explicitly start the TUI runtime.\n\
--platform gui: start the GUI runtime when it becomes available."
)]
struct CliArgs {
    #[arg(long, value_enum, conflicts_with_all = ["tui", "gui", "platform_positional"])]
    platform: Option<PlatformKind>,

    #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gui", "platform", "platform_positional"])]
    tui: bool,

    #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["tui", "platform", "platform_positional"])]
    gui: bool,

    #[arg(value_enum, hide = true, conflicts_with_all = ["platform", "tui", "gui"])]
    platform_positional: Option<PlatformKind>,
}

impl CliArgs {
    fn into_launch_options(self) -> LaunchOptions {
        let platform = self
            .platform
            .or(self.platform_positional)
            .or_else(|| self.tui.then_some(PlatformKind::Tui))
            .or_else(|| self.gui.then_some(PlatformKind::Gui))
            .unwrap_or(PlatformKind::DEFAULT);

        LaunchOptions { platform }
    }
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("{0}")]
    InvalidArgs(String),
    #[error("failed to initialize app state: {0}")]
    StateInit(String),
    #[error("{kind} platform is not enabled: {reason}")]
    Unavailable {
        kind: PlatformKind,
        reason: &'static str,
    },
    #[error("{kind} platform failed: {message}")]
    Runtime { kind: PlatformKind, message: String },
}

impl PlatformError {
    pub fn runtime(kind: PlatformKind, message: impl Into<String>) -> Self {
        Self::Runtime {
            kind,
            message: message.into(),
        }
    }
}

pub trait PlatformRuntime {
    fn kind(&self) -> PlatformKind;
    fn launch(&self, session: CoreSession) -> Result<(), PlatformError>;
}

pub fn run_entrypoint(
    state_factory: impl FnOnce() -> Result<AppState, PlatformError>,
) -> Result<(), PlatformError> {
    run_entrypoint_with_writer(std::env::args().skip(1), io::stdout(), state_factory)
}

fn run_entrypoint_with_writer(
    args: impl IntoIterator<Item = String>,
    mut output: impl Write,
    state_factory: impl FnOnce() -> Result<AppState, PlatformError>,
) -> Result<(), PlatformError> {
    match resolve_launch_command(args)? {
        LaunchCommand::Run(options) => {
            let mut state = state_factory()?;
            match load_editor_config() {
                Ok(config) => {
                    state.editor.project_path = config.project_path;
                    state.editor.auto_load_project_on_startup = config.auto_load_project_on_startup;
                    state.editor.auto_save_project_on_export = config.auto_save_project_on_export;
                    state.editor.startup_focus = config.startup_focus.into();
                    state.ui.focus = state.editor.startup_focus;

                    if state.editor.auto_load_project_on_startup {
                        match load_project(&state.editor.project_path) {
                            Ok(project) => {
                                if let Err(err) = state.apply_project_data(project) {
                                    state.ui.status = format!("Auto-load recompute failed: {err}");
                                } else {
                                    state.ui.status = format!(
                                        "Auto-loaded project from {}",
                                        state.editor.project_path.display()
                                    );
                                }
                            }
                            Err(err) => {
                                state.ui.status = format!("Auto-load failed: {err}");
                            }
                        }
                    }
                }
                Err(err) => {
                    state.ui.status = format!("Editor config load failed: {err}");
                }
            }
            launch(state, options)
        }
        LaunchCommand::PrintHelp(help) => {
            write!(output, "{help}")
                .map_err(|err| PlatformError::runtime(PlatformKind::DEFAULT, err.to_string()))?;
            Ok(())
        }
    }
}

pub fn launch(state: AppState, options: LaunchOptions) -> Result<(), PlatformError> {
    build_runtime(options.platform).launch(CoreSession::new(state))
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

pub fn resolve_launch_command(
    args: impl IntoIterator<Item = String>,
) -> Result<LaunchCommand, PlatformError> {
    let argv = std::iter::once("theme".to_string())
        .chain(args)
        .collect::<Vec<_>>();

    match CliArgs::try_parse_from(argv) {
        Ok(cli) => Ok(LaunchCommand::Run(cli.into_launch_options())),
        Err(error) => match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                Ok(LaunchCommand::PrintHelp(error.to_string()))
            }
            _ => Err(PlatformError::InvalidArgs(error.to_string())),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LaunchCommand, PlatformError, PlatformKind, resolve_launch_command,
        run_entrypoint_with_writer,
    };

    fn parse(args: &[&str]) -> Result<PlatformKind, PlatformError> {
        match resolve_launch_command(args.iter().map(|arg| (*arg).to_string()))? {
            LaunchCommand::Run(options) => Ok(options.platform),
            LaunchCommand::PrintHelp(_) => panic!("expected a run command"),
        }
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
        assert!(err.contains("invalid value"));
        assert!(err.contains("tui"));
        assert!(err.contains("gui"));
    }

    #[test]
    fn rejects_conflicting_platform_options() {
        let err = parse(&["--tui", "--gui"]).unwrap_err().to_string();
        assert!(err.contains("cannot be used with"));
    }

    #[test]
    fn help_is_a_successful_command() {
        match resolve_launch_command(["--help".to_string()]).unwrap() {
            LaunchCommand::PrintHelp(help) => {
                assert!(help.contains("Usage: theme"));
                assert!(help.contains("--platform"));
            }
            LaunchCommand::Run(_) => panic!("expected help output"),
        }
    }

    #[test]
    fn entrypoint_prints_help_without_building_state() {
        let mut output = Vec::new();
        run_entrypoint_with_writer(["--help".to_string()], &mut output, || {
            Err(PlatformError::StateInit("should not run".to_string()))
        })
        .unwrap();

        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Usage: theme"));
        assert!(text.contains("No arguments: start tui."));
    }
}
