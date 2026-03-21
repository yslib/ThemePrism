use crate::domain::color::Color;
use crate::domain::params::ParamKey;
use crate::domain::tokens::TokenRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceField {
    AliasSource,
    MixA,
    MixB,
    AdjustSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlId {
    Param(ParamKey),
    RuleKind(TokenRole),
    Reference(TokenRole, ReferenceField),
    MixRatio(TokenRole),
    AdjustOp(TokenRole),
    AdjustAmount(TokenRole),
    FixedColor(TokenRole),
}

impl ControlId {
    pub fn supports_text_input(self) -> bool {
        matches!(
            self,
            Self::Param(_) | Self::MixRatio(_) | Self::AdjustAmount(_) | Self::FixedColor(_)
        )
    }

    pub fn supports_source_picker(self) -> bool {
        matches!(self, Self::Reference(_, _))
    }

    pub fn label(self) -> String {
        match self {
            Self::Param(key) => key.label().to_string(),
            Self::RuleKind(role) => format!("{} rule type", role.label()),
            Self::Reference(role, field) => format!("{} {}", role.label(), field.label()),
            Self::MixRatio(role) => format!("{} blend ratio", role.label()),
            Self::AdjustOp(role) => format!("{} operation", role.label()),
            Self::AdjustAmount(role) => format!("{} adjust amount", role.label()),
            Self::FixedColor(role) => format!("{} fixed color", role.label()),
        }
    }
}

impl ReferenceField {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AliasSource | Self::AdjustSource => "source",
            Self::MixA => "color A",
            Self::MixB => "color B",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScalarControlSpec {
    pub id: ControlId,
    pub label: String,
    pub value_text: String,
    pub current: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChoiceControlSpec {
    pub id: ControlId,
    pub label: String,
    pub value_text: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReferencePickerControlSpec {
    pub id: ControlId,
    pub label: String,
    pub value_text: String,
    pub picker_open: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ColorControlSpec {
    pub id: ControlId,
    pub label: String,
    pub value_text: String,
    pub color: Color,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DisplayFieldSpec {
    pub label: String,
    pub value_text: String,
    pub swatch: Option<Color>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ControlSpec {
    Scalar(ScalarControlSpec),
    Choice(ChoiceControlSpec),
    ReferencePicker(ReferencePickerControlSpec),
    Color(ColorControlSpec),
    Display(DisplayFieldSpec),
}

impl ControlSpec {
    pub fn label(&self) -> &str {
        match self {
            Self::Scalar(spec) => &spec.label,
            Self::Choice(spec) => &spec.label,
            Self::ReferencePicker(spec) => &spec.label,
            Self::Color(spec) => &spec.label,
            Self::Display(spec) => &spec.label,
        }
    }

    pub fn value_text(&self) -> &str {
        match self {
            Self::Scalar(spec) => &spec.value_text,
            Self::Choice(spec) => &spec.value_text,
            Self::ReferencePicker(spec) => &spec.value_text,
            Self::Color(spec) => &spec.value_text,
            Self::Display(spec) => &spec.value_text,
        }
    }

    pub fn swatch(&self) -> Option<Color> {
        match self {
            Self::Color(spec) => Some(spec.color),
            Self::Display(spec) => spec.swatch,
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> Option<ControlId> {
        match self {
            Self::Scalar(spec) => Some(spec.id),
            Self::Choice(spec) => Some(spec.id),
            Self::ReferencePicker(spec) => Some(spec.id),
            Self::Color(spec) => Some(spec.id),
            Self::Display(_) => None,
        }
    }
}
