pub mod alacritty;

use std::error::Error;
use std::fmt;

use crate::evaluator::ResolvedTheme;

#[allow(dead_code)]
pub trait Exporter {
    fn name(&self) -> &'static str;
    fn export(&self, theme: &ResolvedTheme) -> Result<String, ExportError>;
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ExportError {
    MissingToken(String),
    SerializeError(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingToken(token) => write!(f, "missing token {token}"),
            Self::SerializeError(message) => f.write_str(message),
        }
    }
}

impl Error for ExportError {}
