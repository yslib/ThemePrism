use std::path::PathBuf;

pub const BUNDLED_TEMPLATE_PATH: &str = "templates/alacritty.toml";

pub fn bundled_template_path() -> PathBuf {
    PathBuf::from(BUNDLED_TEMPLATE_PATH)
}
