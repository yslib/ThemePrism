use std::path::{Path, PathBuf};

pub const BUNDLED_TEMPLATE_PATH: &str = "@bundled/alacritty";
pub const GENERIC_TEMPLATE_PATH: &str = "@bundled/generic-theme";

const BUNDLED_TEMPLATE_ASSET_PATH: &str = "templates/alacritty.toml";
const GENERIC_TEMPLATE_ASSET_PATH: &str = "templates/generic-theme.txt";

pub fn bundled_template_path() -> PathBuf {
    PathBuf::from(BUNDLED_TEMPLATE_PATH)
}

pub fn generic_template_path() -> PathBuf {
    PathBuf::from(GENERIC_TEMPLATE_PATH)
}

pub fn resolve_bundled_template_path(path: &Path) -> Option<PathBuf> {
    if path == Path::new(BUNDLED_TEMPLATE_PATH) {
        Some(bundled_template_absolute_path())
    } else if path == Path::new(GENERIC_TEMPLATE_PATH) {
        Some(generic_template_absolute_path())
    } else {
        None
    }
}

fn bundled_template_absolute_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(BUNDLED_TEMPLATE_ASSET_PATH)
}

fn generic_template_absolute_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(GENERIC_TEMPLATE_ASSET_PATH)
}
