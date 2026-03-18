use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PaletteSlot {
    Bg0,
    Bg1,
    Bg2,
    Fg0,
    Fg1,
    Fg2,
    Accent0,
    Accent1,
    Accent2,
    Accent3,
    Accent4,
    Accent5,
}

impl PaletteSlot {
    pub const ALL: [Self; 12] = [
        Self::Bg0,
        Self::Bg1,
        Self::Bg2,
        Self::Fg0,
        Self::Fg1,
        Self::Fg2,
        Self::Accent0,
        Self::Accent1,
        Self::Accent2,
        Self::Accent3,
        Self::Accent4,
        Self::Accent5,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Bg0 => "bg_0",
            Self::Bg1 => "bg_1",
            Self::Bg2 => "bg_2",
            Self::Fg0 => "fg_0",
            Self::Fg1 => "fg_1",
            Self::Fg2 => "fg_2",
            Self::Accent0 => "accent_0",
            Self::Accent1 => "accent_1",
            Self::Accent2 => "accent_2",
            Self::Accent3 => "accent_3",
            Self::Accent4 => "accent_4",
            Self::Accent5 => "accent_5",
        }
    }
}

impl fmt::Display for PaletteSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenCategory {
    Ui,
    Syntax,
    State,
}

impl TokenCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ui => "UI",
            Self::Syntax => "Syntax",
            Self::State => "State",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenRole {
    Background,
    Surface,
    SurfaceAlt,
    Text,
    TextMuted,
    Border,
    Selection,
    Cursor,
    Comment,
    Keyword,
    String,
    Number,
    Type,
    Function,
    Variable,
    Error,
    Warning,
    Info,
    Hint,
    Success,
}

impl TokenRole {
    pub const ALL: [Self; 20] = [
        Self::Background,
        Self::Surface,
        Self::SurfaceAlt,
        Self::Text,
        Self::TextMuted,
        Self::Border,
        Self::Selection,
        Self::Cursor,
        Self::Comment,
        Self::Keyword,
        Self::String,
        Self::Number,
        Self::Type,
        Self::Function,
        Self::Variable,
        Self::Error,
        Self::Warning,
        Self::Info,
        Self::Hint,
        Self::Success,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Background => "Background",
            Self::Surface => "Surface",
            Self::SurfaceAlt => "SurfaceAlt",
            Self::Text => "Text",
            Self::TextMuted => "TextMuted",
            Self::Border => "Border",
            Self::Selection => "Selection",
            Self::Cursor => "Cursor",
            Self::Comment => "Comment",
            Self::Keyword => "Keyword",
            Self::String => "String",
            Self::Number => "Number",
            Self::Type => "Type",
            Self::Function => "Function",
            Self::Variable => "Variable",
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
            Self::Hint => "Hint",
            Self::Success => "Success",
        }
    }

    pub const fn category(self) -> TokenCategory {
        match self {
            Self::Background
            | Self::Surface
            | Self::SurfaceAlt
            | Self::Text
            | Self::TextMuted
            | Self::Border
            | Self::Selection
            | Self::Cursor => TokenCategory::Ui,
            Self::Comment
            | Self::Keyword
            | Self::String
            | Self::Number
            | Self::Type
            | Self::Function
            | Self::Variable => TokenCategory::Syntax,
            Self::Error | Self::Warning | Self::Info | Self::Hint | Self::Success => {
                TokenCategory::State
            }
        }
    }
}

impl fmt::Display for TokenRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
