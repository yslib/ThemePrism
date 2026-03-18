use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::color::Color;
use crate::palette::Palette;
use crate::rules::{AdjustOp, Rule, RuleSet, SourceRef};
use crate::tokens::{PaletteSlot, TokenRole};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTheme {
    pub palette: Palette,
    pub tokens: BTreeMap<TokenRole, Color>,
}

impl ResolvedTheme {
    pub fn token(&self, role: TokenRole) -> Option<Color> {
        self.tokens.get(&role).copied()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum EvalError {
    CycleDetected(Vec<TokenRole>),
    MissingRule(TokenRole),
    MissingPaletteSlot(PaletteSlot),
    InvalidRatio(f32),
    InvalidAmount(f32),
    InvalidSource(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleDetected(path) => {
                let labels = path
                    .iter()
                    .map(|role| role.label())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(f, "rule cycle detected: {labels}")
            }
            Self::MissingRule(role) => write!(f, "missing rule for {}", role.label()),
            Self::MissingPaletteSlot(slot) => write!(f, "missing palette slot {}", slot.label()),
            Self::InvalidRatio(value) => write!(f, "invalid mix ratio {value:.2}"),
            Self::InvalidAmount(value) => write!(f, "invalid adjust amount {value:.2}"),
            Self::InvalidSource(message) => f.write_str(message),
        }
    }
}

impl Error for EvalError {}

pub fn resolve_theme(palette: Palette, rules: &RuleSet) -> Result<ResolvedTheme, EvalError> {
    let mut resolver = Resolver {
        palette: &palette,
        rules,
        cache: BTreeMap::new(),
        stack: Vec::new(),
    };

    for role in TokenRole::ALL {
        resolver.resolve_token(role)?;
    }

    let tokens = resolver.cache.clone();
    drop(resolver);

    Ok(ResolvedTheme { palette, tokens })
}

struct Resolver<'a> {
    palette: &'a Palette,
    rules: &'a RuleSet,
    cache: BTreeMap<TokenRole, Color>,
    stack: Vec<TokenRole>,
}

impl Resolver<'_> {
    fn resolve_token(&mut self, role: TokenRole) -> Result<Color, EvalError> {
        if let Some(color) = self.cache.get(&role).copied() {
            return Ok(color);
        }
        if let Some(index) = self.stack.iter().position(|candidate| *candidate == role) {
            let mut cycle = self.stack[index..].to_vec();
            cycle.push(role);
            return Err(EvalError::CycleDetected(cycle));
        }

        let rule = self.rules.get(role).ok_or(EvalError::MissingRule(role))?;
        self.stack.push(role);
        let color = self.evaluate_rule(rule)?;
        self.stack.pop();
        self.cache.insert(role, color);
        Ok(color)
    }

    fn evaluate_rule(&mut self, rule: &Rule) -> Result<Color, EvalError> {
        match rule {
            Rule::Alias { source } => self.resolve_source(source),
            Rule::Mix { a, b, ratio } => {
                if !(0.0..=1.0).contains(ratio) {
                    return Err(EvalError::InvalidRatio(*ratio));
                }
                let color_a = self.resolve_source(a)?;
                let color_b = self.resolve_source(b)?;
                Ok(color_a.mix(color_b, *ratio))
            }
            Rule::Adjust { source, op, amount } => {
                if !(0.0..=1.0).contains(amount) {
                    return Err(EvalError::InvalidAmount(*amount));
                }
                let color = self.resolve_source(source)?;
                Ok(match op {
                    AdjustOp::Lighten => color.lighten(*amount),
                    AdjustOp::Darken => color.darken(*amount),
                    AdjustOp::Saturate => color.saturate(*amount),
                    AdjustOp::Desaturate => color.desaturate(*amount),
                })
            }
            Rule::Fixed { color } => Ok(*color),
        }
    }

    fn resolve_source(&mut self, source: &SourceRef) -> Result<Color, EvalError> {
        match source {
            SourceRef::Token(role) => self.resolve_token(*role),
            SourceRef::Palette(slot) => self
                .palette
                .get(*slot)
                .ok_or(EvalError::MissingPaletteSlot(*slot)),
            SourceRef::Literal(color) => Ok(*color),
        }
    }
}
