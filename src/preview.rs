use crate::color::Color;
use crate::enum_meta::define_key_enum;
use crate::tokens::TokenRole;

define_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum PreviewMode {
        Code => "code",
        Shell => "shell",
        Lazygit => "lazygit",
    }
}

impl PreviewMode {
    pub const fn next(self) -> Self {
        match self {
            Self::Code => Self::Shell,
            Self::Shell => Self::Lazygit,
            Self::Lazygit => Self::Code,
        }
    }

    pub const fn previous(self) -> Self {
        match self {
            Self::Code => Self::Lazygit,
            Self::Shell => Self::Code,
            Self::Lazygit => Self::Shell,
        }
    }

    pub const fn is_runtime_backed(self) -> bool {
        !matches!(self, Self::Code)
    }

    pub const fn is_interactive(self) -> bool {
        matches!(self, Self::Shell | Self::Lazygit)
    }
}

#[derive(Debug, Clone)]
pub struct PreviewState {
    pub active_mode: PreviewMode,
    pub capture_active: bool,
    pub runtime_frame: PreviewFrame,
    pub runtime_status: String,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            active_mode: PreviewMode::Code,
            capture_active: false,
            runtime_frame: PreviewFrame::placeholder("", ""),
            runtime_status: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PreviewFrame {
    Document(PreviewDocument),
    Placeholder(PreviewMessage),
    Error(PreviewMessage),
}

impl PreviewFrame {
    pub fn placeholder(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Placeholder(PreviewMessage {
            title: title.into(),
            detail: detail.into(),
        })
    }

    pub fn error(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Error(PreviewMessage {
            title: title.into(),
            detail: detail.into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PreviewMessage {
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default)]
pub struct PreviewDocument {
    pub lines: Vec<PreviewLine>,
}

#[derive(Debug, Clone, Default)]
pub struct PreviewLine {
    pub spans: Vec<PreviewSpan>,
}

#[derive(Debug, Clone)]
pub struct PreviewSpan {
    pub text: String,
    pub style: PreviewSpanStyle,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewSpanStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone)]
pub enum PreviewRuntimeEvent {
    FrameUpdated(PreviewFrame),
    StatusUpdated(String),
    Exited { message: String },
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewTemplateSegment {
    pub text: &'static str,
    pub role: Option<TokenRole>,
}

pub fn sample_code_template() -> Vec<Vec<PreviewTemplateSegment>> {
    use TokenRole::*;

    vec![
        vec![
            seg("fn", Some(Keyword)),
            seg(" ", None),
            seg("build_theme", Some(Function)),
            seg("()", None),
            seg(" -> ", None),
            seg("ResolvedTheme", Some(Type)),
            seg(" {", None),
        ],
        vec![
            seg("    let ", Some(Keyword)),
            seg("background", Some(Variable)),
            seg(" = ", None),
            seg("\"#0D1017\"", Some(String)),
            seg(";", None),
        ],
        vec![
            seg("    let ", Some(Keyword)),
            seg("selection_mix", Some(Variable)),
            seg(" = ", None),
            seg("0.35", Some(Number)),
            seg(";", None),
        ],
        vec![seg("    // Blend accent with the surface", Some(Comment))],
        vec![
            seg("    ", None),
            seg("theme", Some(Variable)),
            seg(".", None),
            seg("resolve", Some(Function)),
            seg("(", None),
            seg("selection_mix", Some(Variable)),
            seg(")", None),
        ],
        vec![seg("}", None)],
    ]
}

pub fn sample_document(resolve: impl Fn(TokenRole) -> Color) -> PreviewDocument {
    let background = resolve(TokenRole::Background);
    let default_text = resolve(TokenRole::Text);

    PreviewDocument {
        lines: sample_code_template()
            .into_iter()
            .map(|segments| PreviewLine {
                spans: segments
                    .into_iter()
                    .map(|segment| PreviewSpan {
                        text: segment.text.to_string(),
                        style: PreviewSpanStyle {
                            fg: Some(segment.role.map(&resolve).unwrap_or(default_text)),
                            bg: Some(background),
                            bold: false,
                            italic: false,
                        },
                    })
                    .collect(),
            })
            .collect(),
    }
}

const fn seg(text: &'static str, role: Option<TokenRole>) -> PreviewTemplateSegment {
    PreviewTemplateSegment { text, role }
}
