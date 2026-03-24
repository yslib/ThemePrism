use std::env;
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::layout::Rect;

use crate::app::Intent;
use crate::app::actions::{BoundAction, matches_bound_action};
use crate::app::view::ViewTree;
use crate::app::workspace::PanelId;
use crate::core::CoreSession;
use crate::domain::color::Color;
use crate::domain::preview::{
    PreviewDocument, PreviewFrame, PreviewLine, PreviewMode, PreviewRuntimeEvent, PreviewSpan,
    PreviewSpanStyle,
};
use crate::tokens::TokenRole;
use crate::platform::tui::view_metrics::locate_panel_body;

#[derive(Default)]
pub struct PreviewRuntimeController {
    active_mode: Option<PreviewMode>,
    session: Option<HostedPreviewSession>,
    last_size: Option<PreviewSize>,
}

impl PreviewRuntimeController {
    pub fn sync(
        &mut self,
        session: &mut CoreSession,
        tree: &ViewTree,
        area: Rect,
    ) -> Result<(), String> {
        let Some(size) = preview_body_size(tree, area) else {
            self.stop_current();
            return Ok(());
        };

        let mode = session.state().preview.active_mode;
        if !mode.is_runtime_backed() {
            self.stop_current();
            return Ok(());
        }

        if self.active_mode != Some(mode) {
            self.stop_current();
            match HostedPreviewSession::start(mode, size) {
                Ok(hosted) => {
                    self.session = Some(hosted);
                    self.active_mode = Some(mode);
                    self.last_size = Some(size);
                }
                Err(err) => {
                    self.active_mode = Some(mode);
                    self.last_size = Some(size);
                    session.dispatch(Intent::PreviewRuntimeEvent(PreviewRuntimeEvent::Exited {
                        message: err,
                    }));
                    return Ok(());
                }
            }
        }

        if self.last_size != Some(size) {
            if let Some(hosted) = self.session.as_mut() {
                hosted.resize(size)?;
            }
            self.last_size = Some(size);
        }

        let mut should_stop = false;
        if let Some(hosted) = self.session.as_mut() {
            while let Some(event) = hosted.poll(session.state()) {
                if matches!(event, PreviewRuntimeEvent::Exited { .. }) {
                    should_stop = true;
                }
                session.dispatch(Intent::PreviewRuntimeEvent(event));
            }
        }
        if should_stop {
            self.stop_current();
        }

        Ok(())
    }

    pub fn handle_capture_key(
        &mut self,
        session: &mut CoreSession,
        key: KeyEvent,
    ) -> Result<bool, String> {
        if !session.state().preview.capture_active {
            return Ok(false);
        }

        if matches_bound_action(
            session.state().editor.keymap_preset,
            BoundAction::ReleasePreviewCapture,
            &key,
        ) {
            session.dispatch(Intent::SetPreviewCapture(false));
            return Ok(true);
        }

        let Some(hosted) = self.session.as_mut() else {
            session.dispatch(Intent::SetPreviewCapture(false));
            return Ok(false);
        };

        if let Some(bytes) = encode_key_event(key) {
            hosted.send_input(&bytes)?;
        }
        Ok(true)
    }

    fn stop_current(&mut self) {
        self.session = None;
        self.active_mode = None;
        self.last_size = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreviewSize {
    cols: u16,
    rows: u16,
}

struct HostedPreviewSession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    parser: vt100::Parser,
    events: Receiver<HostedPreviewEvent>,
    exited: bool,
}

enum HostedPreviewEvent {
    Bytes(Vec<u8>),
}

impl HostedPreviewSession {
    fn start(mode: PreviewMode, size: PreviewSize) -> Result<Self, String> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(to_pty_size(size))
            .map_err(|err| err.to_string())?;

        let command = command_for_mode(mode);
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|err| err.to_string())?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|err| err.to_string())?;
        let writer = pair.master.take_writer().map_err(|err| err.to_string())?;

        let (tx, rx) = mpsc::channel();
        let tx_output = tx.clone();
        thread::spawn(move || {
            let mut buffer = [0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => {
                        if tx_output
                            .send(HostedPreviewEvent::Bytes(buffer[..read].to_vec()))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            master: pair.master,
            writer,
            child,
            parser: vt100::Parser::new(size.rows, size.cols, 0),
            events: rx,
            exited: false,
        })
    }

    fn resize(&mut self, size: PreviewSize) -> Result<(), String> {
        self.master
            .resize(to_pty_size(size))
            .map_err(|err| err.to_string())?;
        self.parser.set_size(size.rows, size.cols);
        Ok(())
    }

    fn send_input(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.writer
            .write_all(bytes)
            .map_err(|err| err.to_string())?;
        self.writer.flush().map_err(|err| err.to_string())
    }

    fn poll(&mut self, session: &crate::app::AppState) -> Option<PreviewRuntimeEvent> {
        if let Some(event) = self.events.try_recv().ok() {
            match event {
                HostedPreviewEvent::Bytes(bytes) => {
                    self.parser.process(&bytes);
                    return Some(PreviewRuntimeEvent::FrameUpdated(PreviewFrame::Document(
                        document_from_parser(&self.parser, session),
                    )));
                }
            }
        }

        if self.exited {
            return None;
        }

        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.exited = true;
                Some(PreviewRuntimeEvent::Exited {
                    message: format!("preview exited with status {status:?}"),
                })
            }
            Ok(None) => None,
            Err(err) => {
                self.exited = true;
                Some(PreviewRuntimeEvent::Exited {
                    message: format!("preview exited: {err}"),
                })
            }
        }
    }
}

impl Drop for HostedPreviewSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn command_for_mode(mode: PreviewMode) -> CommandBuilder {
    match mode {
        PreviewMode::Code => CommandBuilder::new("/usr/bin/true"),
        PreviewMode::Shell => {
            let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
            let mut command = CommandBuilder::new(shell);
            command.arg("-i");
            command.env("TERM", "xterm-256color");
            command
        }
        PreviewMode::Lazygit => {
            let mut command = CommandBuilder::new("lazygit");
            command.env("TERM", "xterm-256color");
            command
        }
    }
}

fn to_pty_size(size: PreviewSize) -> PtySize {
    PtySize {
        rows: size.rows,
        cols: size.cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn encode_key_event(key: KeyEvent) -> Option<Vec<u8>> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(ch) = key.code {
            let lower = ch.to_ascii_lowercase() as u8;
            return Some(vec![lower & 0x1f]);
        }
    }

    Some(match key.code {
        KeyCode::Char(ch) => ch.to_string().into_bytes(),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        _ => return None,
    })
}

fn document_from_parser(parser: &vt100::Parser, session: &crate::app::AppState) -> PreviewDocument {
    let screen = parser.screen();
    let (rows, cols) = screen.size();
    let mut lines = Vec::new();

    for row in 0..rows {
        let mut spans = Vec::new();
        let mut current_style = PreviewSpanStyle::default();
        let mut current_text = String::new();

        for col in 0..cols {
            let Some(cell) = screen.cell(row, col) else {
                current_text.push(' ');
                continue;
            };
            if cell.is_wide_continuation() {
                continue;
            }

            let mut fg = map_foreground_color(cell.fgcolor(), session);
            let mut bg = map_background_color(cell.bgcolor(), session);
            if cell.inverse() {
                std::mem::swap(&mut fg, &mut bg);
            }
            let style = PreviewSpanStyle {
                fg: Some(fg),
                bg: Some(bg),
                bold: cell.bold(),
                italic: cell.italic(),
            };
            let ch = if cell.has_contents() {
                cell.contents()
            } else {
                " ".to_string()
            };
            if !current_text.is_empty() && styles_equal(current_style, style) {
                current_text.push_str(&ch);
            } else {
                if !current_text.is_empty() {
                    spans.push(PreviewSpan {
                        text: std::mem::take(&mut current_text),
                        style: current_style,
                    });
                }
                current_style = style;
                current_text.push_str(&ch);
            }
        }

        if !current_text.is_empty() {
            spans.push(PreviewSpan {
                text: current_text,
                style: current_style,
            });
        }

        lines.push(PreviewLine { spans });
    }

    PreviewDocument { lines }
}

fn styles_equal(a: PreviewSpanStyle, b: PreviewSpanStyle) -> bool {
    a.fg == b.fg && a.bg == b.bg && a.bold == b.bold && a.italic == b.italic
}

fn map_foreground_color(color: vt100::Color, session: &crate::app::AppState) -> Color {
    match color {
        vt100::Color::Default => session.theme_color(TokenRole::Text),
        vt100::Color::Idx(index) => map_ansi_color(index, session),
        vt100::Color::Rgb(r, g, b) => Color::from_rgba_u8(r, g, b, 255),
    }
}

fn map_background_color(color: vt100::Color, session: &crate::app::AppState) -> Color {
    match color {
        vt100::Color::Default => session.theme_color(TokenRole::Background),
        vt100::Color::Idx(index) => map_ansi_color(index, session),
        vt100::Color::Rgb(r, g, b) => Color::from_rgba_u8(r, g, b, 255),
    }
}

fn map_ansi_color(index: u8, session: &crate::app::AppState) -> Color {
    match index {
        0 => session.theme_color(TokenRole::Background),
        1 | 9 => session.theme_color(TokenRole::Error),
        2 | 10 => session.theme_color(TokenRole::Success),
        3 | 11 => session.theme_color(TokenRole::Warning),
        4 | 12 => session.theme_color(TokenRole::Info),
        5 | 13 => session.theme_color(TokenRole::Keyword),
        6 | 14 => session.theme_color(TokenRole::Hint),
        7 | 15 => session.theme_color(TokenRole::Text),
        8 => session.theme_color(TokenRole::SurfaceAlt),
        _ => session.theme_color(TokenRole::TextMuted),
    }
}

fn preview_body_size(tree: &ViewTree, area: Rect) -> Option<PreviewSize> {
    locate_panel_body(tree, area, PanelId::Preview).map(|(_, body)| PreviewSize {
        cols: body.width.max(1),
        rows: body.height.max(1),
    })
}
