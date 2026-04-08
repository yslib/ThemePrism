use std::path::{Component, Path, PathBuf};

pub const BUNDLED_TEMPLATE_PATH: &str = "templates/alacritty.toml";
pub const GENERIC_TEMPLATE_PATH: &str = "templates/generic-theme.txt";

pub fn bundled_template_path() -> PathBuf {
    PathBuf::from(BUNDLED_TEMPLATE_PATH)
}

pub fn resolve_bundled_template_path(path: &Path) -> Option<PathBuf> {
    match () {
        _ if matches_bundled_template_marker(path, BUNDLED_TEMPLATE_PATH) => {
            Some(bundled_template_absolute_path())
        }
        _ if matches_bundled_template_marker(path, GENERIC_TEMPLATE_PATH) => {
            Some(generic_template_absolute_path())
        }
        _ => None,
    }
}

fn matches_bundled_template_marker(path: &Path, marker: &str) -> bool {
    normalized_components(path).eq(normalized_components(Path::new(marker)))
}

fn normalized_components(path: &Path) -> impl Iterator<Item = Component<'_>> {
    path.components()
        .filter(|component| !matches!(component, Component::CurDir))
}

fn bundled_template_absolute_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(BUNDLED_TEMPLATE_PATH)
}

fn generic_template_absolute_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(GENERIC_TEMPLATE_PATH)
}
