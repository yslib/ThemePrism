use crate::evaluator::ResolvedTheme;
use crate::export::{ExportError, Exporter};
use crate::tokens::TokenRole;

#[derive(Debug, Default)]
pub struct AlacrittyExporter;

impl Exporter for AlacrittyExporter {
    fn name(&self) -> &'static str {
        "Alacritty"
    }

    fn export(&self, theme: &ResolvedTheme) -> Result<String, ExportError> {
        let token = |role: TokenRole| {
            theme
                .token(role)
                .map(|color| color.to_hex())
                .ok_or_else(|| ExportError::MissingToken(role.label().to_string()))
        };

        Ok(format!(
            "[colors.primary]\nbackground = \"{background}\"\nforeground = \"{foreground}\"\n\n\
             [colors.cursor]\ntext = \"{cursor_text}\"\ncursor = \"{cursor}\"\n\n\
             [colors.selection]\ntext = \"{selection_text}\"\nbackground = \"{selection_bg}\"\n\n\
             [colors.normal]\nblack = \"{black}\"\nred = \"{red}\"\ngreen = \"{green}\"\nyellow = \"{yellow}\"\nblue = \"{blue}\"\nmagenta = \"{magenta}\"\ncyan = \"{cyan}\"\nwhite = \"{white}\"\n\n\
             [colors.bright]\nblack = \"{bright_black}\"\nred = \"{bright_red}\"\ngreen = \"{bright_green}\"\nyellow = \"{bright_yellow}\"\nblue = \"{bright_blue}\"\nmagenta = \"{bright_magenta}\"\ncyan = \"{bright_cyan}\"\nwhite = \"{bright_white}\"\n",
            background = token(TokenRole::Background)?,
            foreground = token(TokenRole::Text)?,
            cursor_text = token(TokenRole::Background)?,
            cursor = token(TokenRole::Cursor)?,
            selection_text = token(TokenRole::Text)?,
            selection_bg = token(TokenRole::Selection)?,
            black = token(TokenRole::Background)?,
            red = token(TokenRole::Error)?,
            green = token(TokenRole::Success)?,
            yellow = token(TokenRole::Warning)?,
            blue = token(TokenRole::Info)?,
            magenta = token(TokenRole::Keyword)?,
            cyan = token(TokenRole::Hint)?,
            white = token(TokenRole::TextMuted)?,
            bright_black = token(TokenRole::SurfaceAlt)?,
            bright_red = token(TokenRole::Error)?,
            bright_green = token(TokenRole::Success)?,
            bright_yellow = token(TokenRole::Warning)?,
            bright_blue = token(TokenRole::Function)?,
            bright_magenta = token(TokenRole::Keyword)?,
            bright_cyan = token(TokenRole::Hint)?,
            bright_white = token(TokenRole::Cursor)?,
        ))
    }
}
