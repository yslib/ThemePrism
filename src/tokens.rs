use crate::enum_meta::define_labeled_key_enum;

define_labeled_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum PaletteSlot {
        Bg0 => { key: "bg_0", label: "bg_0" },
        Bg1 => { key: "bg_1", label: "bg_1" },
        Bg2 => { key: "bg_2", label: "bg_2" },
        Fg0 => { key: "fg_0", label: "fg_0" },
        Fg1 => { key: "fg_1", label: "fg_1" },
        Fg2 => { key: "fg_2", label: "fg_2" },
        Accent0 => { key: "accent_0", label: "accent_0" },
        Accent1 => { key: "accent_1", label: "accent_1" },
        Accent2 => { key: "accent_2", label: "accent_2" },
        Accent3 => { key: "accent_3", label: "accent_3" },
        Accent4 => { key: "accent_4", label: "accent_4" },
        Accent5 => { key: "accent_5", label: "accent_5" },
    }
}

define_labeled_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TokenCategory {
        Ui => { key: "ui", label: "UI" },
        Syntax => { key: "syntax", label: "Syntax" },
        State => { key: "state", label: "State" },
    }
}

define_labeled_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum TokenRole {
        Background => { key: "background", label: "Background" },
        Surface => { key: "surface", label: "Surface" },
        SurfaceAlt => { key: "surface_alt", label: "SurfaceAlt" },
        Text => { key: "text", label: "Text" },
        TextMuted => { key: "text_muted", label: "TextMuted" },
        Border => { key: "border", label: "Border" },
        Selection => { key: "selection", label: "Selection" },
        Cursor => { key: "cursor", label: "Cursor" },
        Comment => { key: "comment", label: "Comment" },
        Keyword => { key: "keyword", label: "Keyword" },
        String => { key: "string", label: "String" },
        Number => { key: "number", label: "Number" },
        Type => { key: "type", label: "Type" },
        Function => { key: "function", label: "Function" },
        Variable => { key: "variable", label: "Variable" },
        Error => { key: "error", label: "Error" },
        Warning => { key: "warning", label: "Warning" },
        Info => { key: "info", label: "Info" },
        Hint => { key: "hint", label: "Hint" },
        Success => { key: "success", label: "Success" },
    }
}

impl TokenRole {
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

#[cfg(test)]
mod tests {
    use super::{PaletteSlot, TokenRole};

    #[test]
    fn token_role_keys_round_trip_with_stable_domain_ids() {
        for role in TokenRole::ALL {
            assert_eq!(TokenRole::from_key(role.key()), Some(role));
            assert_eq!(TokenRole::from_label(role.label()), Some(role));
        }

        assert_eq!(TokenRole::SurfaceAlt.key(), "surface_alt");
        assert_eq!(TokenRole::TextMuted.key(), "text_muted");
    }

    #[test]
    fn palette_slot_keys_round_trip() {
        for slot in PaletteSlot::ALL {
            assert_eq!(PaletteSlot::from_key(slot.key()), Some(slot));
            assert_eq!(PaletteSlot::from_label(slot.label()), Some(slot));
        }
    }
}
