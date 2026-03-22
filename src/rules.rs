use std::collections::BTreeMap;

use crate::color::Color;
use crate::enum_meta::define_labeled_key_enum;
use crate::tokens::{PaletteSlot, TokenRole};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SourceRef {
    Token(TokenRole),
    Palette(PaletteSlot),
    Literal(Color),
}

impl SourceRef {
    pub fn label(&self) -> String {
        match self {
            Self::Token(role) => role.label().to_string(),
            Self::Palette(slot) => slot.label().to_string(),
            Self::Literal(color) => color.to_hex(),
        }
    }
}

define_labeled_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SourceGroup {
        Common => { key: "common", label: "Common Sources" },
        Advanced => { key: "advanced", label: "Advanced Palette" },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceOption {
    pub group: SourceGroup,
    pub source: SourceRef,
}

define_labeled_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AdjustOp {
        Lighten => { key: "lighten", label: "Lighten" },
        Darken => { key: "darken", label: "Darken" },
        Saturate => { key: "saturate", label: "Saturate" },
        Desaturate => { key: "desaturate", label: "Desaturate" },
    }
}

define_labeled_key_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RuleKind {
        Alias => { key: "alias", label: "Alias" },
        Mix => { key: "mix", label: "Mix" },
        Adjust => { key: "adjust", label: "Adjust" },
        Fixed => { key: "fixed", label: "Fixed" },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    Alias {
        source: SourceRef,
    },
    Mix {
        a: SourceRef,
        b: SourceRef,
        ratio: f32,
    },
    Adjust {
        source: SourceRef,
        op: AdjustOp,
        amount: f32,
    },
    Fixed {
        color: Color,
    },
}

impl Rule {
    pub fn kind(&self) -> RuleKind {
        match self {
            Self::Alias { .. } => RuleKind::Alias,
            Self::Mix { .. } => RuleKind::Mix,
            Self::Adjust { .. } => RuleKind::Adjust,
            Self::Fixed { .. } => RuleKind::Fixed,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Alias { source } => format!("Alias({})", source.label()),
            Self::Mix { a, b, ratio } => {
                format!("Mix({}, {}, {:>3.0}%)", a.label(), b.label(), ratio * 100.0)
            }
            Self::Adjust { source, op, amount } => {
                format!(
                    "{}({}, {:>3.0}%)",
                    op.label(),
                    source.label(),
                    amount * 100.0
                )
            }
            Self::Fixed { color } => format!("Fixed({})", color.to_hex()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSet {
    pub rules: BTreeMap<TokenRole, Rule>,
}

impl RuleSet {
    pub fn get(&self, role: TokenRole) -> Option<&Rule> {
        self.rules.get(&role)
    }

    pub fn get_mut(&mut self, role: TokenRole) -> Option<&mut Rule> {
        self.rules.get_mut(&role)
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        let rules = TokenRole::ALL
            .into_iter()
            .map(|role| (role, default_rule_for(role, 0.35)))
            .collect();
        Self { rules }
    }
}

pub fn default_rule_for(role: TokenRole, selection_mix: f32) -> Rule {
    use PaletteSlot::*;
    use SourceRef::{Palette, Token};
    use TokenRole::*;

    match role {
        Background => Rule::Alias {
            source: Palette(Bg0),
        },
        Surface => Rule::Alias {
            source: Palette(Bg1),
        },
        SurfaceAlt => Rule::Alias {
            source: Palette(Bg2),
        },
        Text => Rule::Alias {
            source: Palette(Fg1),
        },
        TextMuted => Rule::Alias {
            source: Palette(Fg0),
        },
        Border => Rule::Adjust {
            source: Token(Surface),
            op: AdjustOp::Lighten,
            amount: 0.08,
        },
        Selection => Rule::Mix {
            a: Palette(Accent3),
            b: Token(Surface),
            ratio: selection_mix,
        },
        Cursor => Rule::Alias {
            source: Palette(Fg2),
        },
        Comment => Rule::Alias {
            source: Token(TextMuted),
        },
        Keyword => Rule::Alias {
            source: Palette(Accent4),
        },
        String => Rule::Alias {
            source: Palette(Accent1),
        },
        Number => Rule::Alias {
            source: Palette(Accent2),
        },
        Type => Rule::Alias {
            source: Palette(Accent3),
        },
        Function => Rule::Alias {
            source: Palette(Accent5),
        },
        Variable => Rule::Alias {
            source: Token(Text),
        },
        Error => Rule::Alias {
            source: Palette(Accent0),
        },
        Warning => Rule::Alias {
            source: Palette(Accent2),
        },
        Info => Rule::Alias {
            source: Palette(Accent3),
        },
        Hint => Rule::Alias {
            source: Palette(Accent5),
        },
        Success => Rule::Alias {
            source: Palette(Accent1),
        },
    }
}

pub fn starter_rule(
    kind: RuleKind,
    role: TokenRole,
    current_color: Color,
    selection_mix: f32,
) -> Rule {
    use PaletteSlot::*;
    use SourceRef::{Palette, Token};
    use TokenRole::*;

    match kind {
        RuleKind::Alias => match default_rule_for(role, selection_mix) {
            Rule::Alias { source } => Rule::Alias { source },
            _ => Rule::Alias {
                source: Token(Text),
            },
        },
        RuleKind::Mix => match role {
            Selection => default_rule_for(role, selection_mix),
            _ => Rule::Mix {
                a: Palette(Accent3),
                b: Token(Surface),
                ratio: 0.50,
            },
        },
        RuleKind::Adjust => match role {
            Border => default_rule_for(role, selection_mix),
            Comment => Rule::Adjust {
                source: Token(Text),
                op: AdjustOp::Desaturate,
                amount: 0.25,
            },
            _ => Rule::Adjust {
                source: Token(Text),
                op: AdjustOp::Lighten,
                amount: 0.10,
            },
        },
        RuleKind::Fixed => Rule::Fixed {
            color: current_color,
        },
    }
}

pub fn available_source_options(current_role: TokenRole) -> Vec<SourceOption> {
    let common_tokens = [
        TokenRole::Background,
        TokenRole::Surface,
        TokenRole::SurfaceAlt,
        TokenRole::Text,
        TokenRole::TextMuted,
        TokenRole::Border,
        TokenRole::Selection,
        TokenRole::Cursor,
        TokenRole::Comment,
        TokenRole::Keyword,
        TokenRole::String,
        TokenRole::Number,
        TokenRole::Type,
        TokenRole::Function,
        TokenRole::Variable,
        TokenRole::Error,
        TokenRole::Warning,
        TokenRole::Info,
        TokenRole::Hint,
        TokenRole::Success,
    ];

    let mut sources = Vec::new();
    for role in common_tokens {
        if role != current_role {
            sources.push(SourceOption {
                group: SourceGroup::Common,
                source: SourceRef::Token(role),
            });
        }
    }
    for slot in PaletteSlot::ALL {
        sources.push(SourceOption {
            group: SourceGroup::Advanced,
            source: SourceRef::Palette(slot),
        });
    }
    sources
}

pub fn available_sources(current_role: TokenRole) -> Vec<SourceRef> {
    available_source_options(current_role)
        .into_iter()
        .map(|option| option.source)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{AdjustOp, RuleKind, SourceGroup};

    #[test]
    fn rule_enums_round_trip_through_keys_and_labels() {
        for kind in RuleKind::ALL {
            assert_eq!(RuleKind::from_key(kind.key()), Some(kind));
            assert_eq!(RuleKind::from_label(kind.label()), Some(kind));
        }

        for op in AdjustOp::ALL {
            assert_eq!(AdjustOp::from_key(op.key()), Some(op));
            assert_eq!(AdjustOp::from_label(op.label()), Some(op));
        }
    }

    #[test]
    fn source_group_labels_are_defined_once() {
        assert_eq!(SourceGroup::from_key("common"), Some(SourceGroup::Common));
        assert_eq!(
            SourceGroup::from_label("Advanced Palette"),
            Some(SourceGroup::Advanced)
        );
    }
}
