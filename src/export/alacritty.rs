use std::path::{Path, PathBuf};

pub const BUNDLED_TEMPLATE_PATH: &str = "@bundled/alacritty";
pub const GENERIC_TEMPLATE_PATH: &str = "@bundled/generic-theme";

const BUNDLED_TEMPLATE_CONTENTS: &str = include_str!("../../templates/alacritty.toml");
const GENERIC_TEMPLATE_CONTENTS: &str = include_str!("../../templates/generic-theme.txt");

pub fn bundled_template_path() -> PathBuf {
    PathBuf::from(BUNDLED_TEMPLATE_PATH)
}

pub fn generic_template_path() -> PathBuf {
    PathBuf::from(GENERIC_TEMPLATE_PATH)
}

pub fn resolve_bundled_template_path(path: &Path) -> Option<PathBuf> {
    bundled_template_contents(path).map(|_| path.to_path_buf())
}

pub fn bundled_template_contents(path: &Path) -> Option<&'static str> {
    if path == Path::new(BUNDLED_TEMPLATE_PATH) {
        Some(BUNDLED_TEMPLATE_CONTENTS)
    } else if path == Path::new(GENERIC_TEMPLATE_PATH) {
        Some(GENERIC_TEMPLATE_CONTENTS)
    } else {
        None
    }
}
