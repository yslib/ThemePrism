use std::collections::BTreeMap;

use crate::color::Color;
use crate::domain::params::{ParamKey, ThemeParams};
use crate::evaluator::ResolvedTheme;
use crate::export::{ExportError, ExportFormat, ExportProfile};
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
    pub exporter: String,
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
    pub fn build(self) -> Result<ExportContext, ExportError> {
        let meta = ExportMeta {
            project_name: self.project_name.to_string(),
            profile_name: self.profile.name.clone(),
            profile_format: match &self.profile.format {
                ExportFormat::Alacritty => "alacritty".to_string(),
                ExportFormat::Template { .. } => "template".to_string(),
            },
            output_path: self.profile.output_path.display().to_string(),
            exporter: self.profile.format_label().to_string(),
        };

        let token = TokenRole::ALL
            .into_iter()
            .map(|role| {
                let color = self.theme.token(role).ok_or_else(|| {
                    ExportError::MissingExportContextValue(format!("token.{}", role.key()))
                })?;
                Ok((role.key().to_string(), ExportValue::Color(color)))
            })
            .collect::<Result<BTreeMap<_, _>, ExportError>>()?;

        let palette = PaletteSlot::ALL
            .into_iter()
            .map(|slot| {
                let color = self.theme.palette.get(slot).ok_or_else(|| {
                    ExportError::MissingExportContextValue(format!("palette.{}", slot.key()))
                })?;
                Ok((slot.key().to_string(), ExportValue::Color(color)))
            })
            .collect::<Result<BTreeMap<_, _>, ExportError>>()?;

        let param = ParamKey::ALL
            .into_iter()
            .map(|key| {
                Ok((key.key().to_string(), ExportValue::Number(key.get(self.params))))
            })
            .collect::<Result<BTreeMap<_, _>, ExportError>>()?;

        Ok(ExportContext {
            meta,
            token,
            palette,
            param,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{ExportContext, ExportValue};
    use crate::domain::params::{ParamKey, ThemeParams};
    use crate::domain::palette::Palette;
    use crate::domain::palette::generate_palette;
    use crate::domain::rules::RuleSet;
    use crate::domain::tokens::{PaletteSlot, TokenRole};
    use crate::evaluator::resolve_theme;
    use crate::export::{ExportError, ExportProfile};
    use crate::evaluator::ResolvedTheme;

    fn build_context() -> ExportContext {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = ExportProfile::template_default();

        ExportContext::builder("Demo Project", &profile, &theme, &params)
            .build()
            .unwrap()
    }

    #[test]
    fn context_exposes_required_meta_fields() {
        let context = build_context();

        assert_eq!(context.meta.project_name, "Demo Project");
        assert_eq!(context.meta.profile_name, "Template");
        assert_eq!(context.meta.profile_format, "template");
        assert_eq!(context.meta.output_path, "exports/theme-template.txt");
        assert_eq!(context.meta.exporter, "Template");
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

    #[test]
    fn context_builder_returns_error_when_theme_data_is_missing() {
        let params = ThemeParams::default();
        let profile = ExportProfile::template_default();
        let theme = ResolvedTheme {
            palette: Palette {
                slots: BTreeMap::new(),
            },
            tokens: BTreeMap::new(),
        };

        let error = ExportContext::builder("Demo Project", &profile, &theme, &params)
            .build()
            .unwrap_err();

        assert!(matches!(
            error,
            ExportError::MissingExportContextValue(value) if value == "token.background"
        ));
    }
}
