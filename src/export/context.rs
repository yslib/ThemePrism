use std::collections::BTreeMap;

use crate::color::Color;
use crate::domain::params::{ParamKey, ThemeParams};
use crate::evaluator::ResolvedTheme;
use crate::export::{ExportError, ExportProfile};
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
    pub exporter_key: String,
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
            profile_format: self.profile.format.key().to_string(),
            output_path: self.profile.output_path.display().to_string(),
            exporter: self.profile.format_label().to_string(),
            exporter_key: self.profile.format_key().to_string(),
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
                Ok((
                    key.key().to_string(),
                    ExportValue::Number(key.get(self.params)),
                ))
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
    use std::collections::{BTreeMap, BTreeSet};

    use super::{ExportContext, ExportValue};
    use crate::domain::palette::Palette;
    use crate::domain::palette::generate_palette;
    use crate::domain::params::{ParamKey, ThemeParams};
    use crate::domain::rules::RuleSet;
    use crate::domain::tokens::{PaletteSlot, TokenRole};
    use crate::evaluator::ResolvedTheme;
    use crate::evaluator::resolve_theme;
    use crate::export::{ExportError, ExportFormat, ExportProfile, ExportWriteMode};

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
        assert_eq!(
            context.meta.output_path,
            std::path::Path::new("exports/theme-template.txt")
                .display()
                .to_string()
        );
        assert_eq!(context.meta.exporter, "Template");
        assert_eq!(context.meta.exporter_key, "template");
    }

    #[test]
    fn legacy_alacritty_profile_normalizes_profile_format_in_context() {
        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let profile = ExportProfile {
            name: "Alacritty".to_string(),
            enabled: true,
            output_path: std::path::PathBuf::from("exports/alacritty-theme.toml"),
            write_mode: ExportWriteMode::ReplaceFile,
            format: ExportFormat::Alacritty,
        }
        .normalize();

        let context = ExportContext::builder("Demo Project", &profile, &theme, &params)
            .build()
            .unwrap();

        assert_eq!(context.meta.profile_format, "template");
    }

    #[test]
    fn context_includes_all_token_palette_and_param_keys() {
        let context = build_context();

        assert_eq!(context.token.len(), TokenRole::ALL.len());
        assert_eq!(context.palette.len(), PaletteSlot::ALL.len());
        assert_eq!(context.param.len(), ParamKey::ALL.len());

        let token_keys: BTreeSet<_> = context.token.keys().cloned().collect();
        let expected_token_keys: BTreeSet<_> = TokenRole::ALL
            .into_iter()
            .map(|role| role.key().to_string())
            .collect();
        assert_eq!(token_keys, expected_token_keys);

        let palette_keys: BTreeSet<_> = context.palette.keys().cloned().collect();
        let expected_palette_keys: BTreeSet<_> = PaletteSlot::ALL
            .into_iter()
            .map(|slot| slot.key().to_string())
            .collect();
        assert_eq!(palette_keys, expected_palette_keys);

        let param_keys: BTreeSet<_> = context.param.keys().cloned().collect();
        let expected_param_keys: BTreeSet<_> = ParamKey::ALL
            .into_iter()
            .map(|key| key.key().to_string())
            .collect();
        assert_eq!(param_keys, expected_param_keys);

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
