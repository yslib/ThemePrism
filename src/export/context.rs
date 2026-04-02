use std::collections::BTreeMap;

use crate::color::Color;
use crate::domain::params::{ParamKey, ThemeParams};
use crate::evaluator::ResolvedTheme;
use crate::export::{ExportFormat, ExportProfile};
use crate::tokens::{PaletteSlot, TokenRole};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ExportValue {
    Color(Color),
    Number(f32),
    Text(String),
}

impl ExportValue {
    pub fn render_text(&self) -> String {
        match self {
            Self::Color(color) => color.to_hex(),
            Self::Number(value) => value.to_string(),
            Self::Text(value) => value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportMeta {
    pub project_name: String,
    pub profile_name: String,
    pub profile_format: String,
    pub output_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportContext {
    pub meta: ExportMeta,
    pub token: BTreeMap<String, ExportValue>,
    pub palette: BTreeMap<String, ExportValue>,
    pub param: BTreeMap<String, ExportValue>,
}

pub struct ExportContextBuilder<'a> {
    project_name: &'a str,
    profile: &'a ExportProfile,
    theme: &'a ResolvedTheme,
    params: &'a ThemeParams,
}

impl ExportContext {
    pub fn builder<'a>(
        project_name: &'a str,
        profile: &'a ExportProfile,
        theme: &'a ResolvedTheme,
        params: &'a ThemeParams,
    ) -> ExportContextBuilder<'a> {
        ExportContextBuilder {
            project_name,
            profile,
            theme,
            params,
        }
    }
}

impl<'a> ExportContextBuilder<'a> {
    pub fn build(self) -> ExportContext {
        let meta = ExportMeta {
            project_name: self.project_name.to_string(),
            profile_name: self.profile.name.clone(),
            profile_format: match &self.profile.format {
                ExportFormat::Alacritty => "alacritty".to_string(),
                ExportFormat::Template { .. } => "template".to_string(),
            },
            output_path: self.profile.output_path.display().to_string(),
        };

        let token = TokenRole::ALL
            .into_iter()
            .map(|role| {
                (
                    role.key().to_string(),
                    ExportValue::Color(
                        self.theme
                            .token(role)
                            .expect("resolved theme must include all token roles"),
                    ),
                )
            })
            .collect();

        let palette = PaletteSlot::ALL
            .into_iter()
            .map(|slot| {
                (
                    slot.key().to_string(),
                    ExportValue::Color(
                        self.theme
                            .palette
                            .get(slot)
                            .expect("resolved palette must include all palette slots"),
                    ),
                )
            })
            .collect();

        let param = ParamKey::ALL
            .into_iter()
            .map(|key| {
                (
                    key.key().to_string(),
                    ExportValue::Number(key.get(self.params)),
                )
            })
            .collect();

        ExportContext {
            meta,
            token,
            palette,
            param,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExportContext, ExportValue};
    use crate::domain::params::{ParamKey, ThemeParams};
    use crate::domain::palette::generate_palette;
    use crate::domain::rules::RuleSet;
    use crate::domain::tokens::{PaletteSlot, TokenRole};
    use crate::evaluator::resolve_theme;
    use crate::export::ExportProfile;

    fn build_context() -> ExportContext {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = ExportProfile::template_default();

        ExportContext::builder("Demo Project", &profile, &theme, &params).build()
    }

    #[test]
    fn context_exposes_required_meta_fields() {
        let context = build_context();

        assert_eq!(context.meta.project_name, "Demo Project");
        assert_eq!(context.meta.profile_name, "Template");
        assert_eq!(context.meta.profile_format, "template");
        assert_eq!(context.meta.output_path, "exports/theme-template.txt");
    }

    #[test]
    fn context_includes_all_token_palette_and_param_keys() {
        let context = build_context();

        for role in TokenRole::ALL {
            assert!(matches!(
                context.token.get(role.key()),
                Some(ExportValue::Color(_))
            ));
        }

        for slot in PaletteSlot::ALL {
            assert!(matches!(
                context.palette.get(slot.key()),
                Some(ExportValue::Color(_))
            ));
        }

        for key in ParamKey::ALL {
            assert!(matches!(
                context.param.get(key.key()),
                Some(ExportValue::Number(_))
            ));
        }
    }
}
