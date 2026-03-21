use std::fs;
use std::path::Path;

use crate::evaluator::ResolvedTheme;
use crate::export::{ExportError, Exporter};
use crate::tokens::TokenRole;

#[derive(Debug, Clone)]
pub struct TemplateExporter {
    profile_name: String,
    template: String,
}

impl TemplateExporter {
    pub fn from_path(profile_name: &str, path: &Path) -> Result<Self, ExportError> {
        let template = fs::read_to_string(path)
            .map_err(|err| ExportError::Io(format!("failed to read template {}: {err}", path.display())))?;
        Ok(Self {
            profile_name: profile_name.to_string(),
            template,
        })
    }
}

impl Exporter for TemplateExporter {
    fn name(&self) -> &'static str {
        "Template"
    }

    fn export(&self, theme: &ResolvedTheme) -> Result<String, ExportError> {
        let mut rendered = self.template.clone();
        rendered = rendered.replace("{{meta.profile_name}}", &self.profile_name);
        rendered = rendered.replace("{{meta.exporter}}", self.name());

        for role in TokenRole::ALL {
            let color = theme
                .token(role)
                .map(|value| value.to_hex())
                .ok_or_else(|| ExportError::MissingToken(role.label().to_string()))?;
            let key = format!("{{{{token.{}}}}}", encode_role(role));
            rendered = rendered.replace(&key, &color);
        }

        if let Some(start) = rendered.find("{{token.") {
            let tail = &rendered[start..];
            let end = tail.find("}}").map(|index| start + index + 2).unwrap_or(rendered.len());
            let unknown = &rendered[start..end];
            return Err(ExportError::InvalidTemplate(format!(
                "unknown template placeholder {unknown}"
            )));
        }

        Ok(rendered)
    }
}

fn encode_role(role: TokenRole) -> &'static str {
    match role {
        TokenRole::Background => "background",
        TokenRole::Surface => "surface",
        TokenRole::SurfaceAlt => "surface_alt",
        TokenRole::Text => "text",
        TokenRole::TextMuted => "text_muted",
        TokenRole::Border => "border",
        TokenRole::Selection => "selection",
        TokenRole::Cursor => "cursor",
        TokenRole::Comment => "comment",
        TokenRole::Keyword => "keyword",
        TokenRole::String => "string",
        TokenRole::Number => "number",
        TokenRole::Type => "type",
        TokenRole::Function => "function",
        TokenRole::Variable => "variable",
        TokenRole::Error => "error",
        TokenRole::Warning => "warning",
        TokenRole::Info => "info",
        TokenRole::Hint => "hint",
        TokenRole::Success => "success",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::export::template::TemplateExporter;
    use crate::export::Exporter;
    use crate::params::ThemeParams;
    use crate::palette::generate_palette;
    use crate::rules::RuleSet;
    use crate::evaluator::resolve_theme;

    #[test]
    fn template_exporter_replaces_token_placeholders() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("theme-template-{unique}.txt"));
        fs::write(
            &path,
            "profile={{meta.profile_name}}\nbackground={{token.background}}\nkeyword={{token.keyword}}\n",
        )
        .unwrap();

        let params = ThemeParams::default();
        let theme = resolve_theme(generate_palette(&params), &RuleSet::default()).unwrap();
        let exporter = TemplateExporter::from_path("Test Profile", &path).unwrap();
        let output = exporter.export(&theme).unwrap();

        assert!(output.contains("profile=Test Profile"));
        assert!(output.contains("background=#"));
        assert!(output.contains("keyword=#"));

        let _ = fs::remove_file(path);
    }
}
