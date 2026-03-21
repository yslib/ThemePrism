use std::collections::BTreeMap;

use crate::color::Color;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceGroup {
    Common,
    Advanced,
}

impl SourceGroup {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Common => "Common Sources",
            Self::Advanced => "Advanced Palette",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceOption {
    pub group: SourceGroup,
    pub source: SourceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustOp {
    Lighten,
    Darken,
    Saturate,
    Desaturate,
}

impl AdjustOp {
    pub const ALL: [Self; 4] = [
        Self::Lighten,
        Self::Darken,
        Self::Saturate,
        Self::Desaturate,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Lighten => "Lighten",
            Self::Darken => "Darken",
            Self::Saturate => "Saturate",
            Self::Desaturate => "Desaturate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    Alias,
    Mix,
    Adjust,
    Fixed,
}

impl RuleKind {
    pub const ALL: [Self; 4] = [Self::Alias, Self::Mix, Self::Adjust, Self::Fixed];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Alias => "Alias",
            Self::Mix => "Mix",
            Self::Adjust => "Adjust",
            Self::Fixed => "Fixed",
        }
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
