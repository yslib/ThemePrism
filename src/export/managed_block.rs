pub const MANAGED_BLOCK_NAME: &str = "theme-generator";
const COMMENT_PREFIX: &str = "#";

pub fn managed_block_start_marker() -> String {
    format!("{COMMENT_PREFIX} {MANAGED_BLOCK_NAME}:start")
}

pub fn managed_block_end_marker() -> String {
    format!("{COMMENT_PREFIX} {MANAGED_BLOCK_NAME}:end")
}

pub fn patch_managed_block_contents(existing: &str, rendered: &str) -> String {
    let newline = detect_newline(existing);
    let start = managed_block_start_marker();
    let end = managed_block_end_marker();
    let body = rendered.trim_end_matches(['\r', '\n']);
    let block = format!("{start}{newline}{body}{newline}{end}{newline}");

    if existing.is_empty() {
        return block;
    }

    let lines = existing.lines().collect::<Vec<_>>();
    let start_index = lines.iter().position(|line| normalized_line(line) == start);
    let end_index = start_index.and_then(|index| {
        lines[index + 1..]
            .iter()
            .position(|line| normalized_line(line) == end)
            .map(|offset| index + 1 + offset)
    });

    if let (Some(start_index), Some(end_index)) = (start_index, end_index) {
        let before = join_lines(&lines[..start_index], newline);
        let after = join_lines(&lines[end_index + 1..], newline);
        return format!("{before}{block}{after}");
    }

    let existing = existing.trim_end_matches(['\r', '\n']);
    format!("{existing}{newline}{newline}{block}")
}

fn detect_newline(existing: &str) -> &'static str {
    if existing.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn normalized_line(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

fn join_lines(lines: &[&str], newline: &str) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        format!(
            "{}{}",
            lines
                .iter()
                .map(|line| normalized_line(line))
                .collect::<Vec<_>>()
                .join(newline),
            newline
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        managed_block_end_marker, managed_block_start_marker, patch_managed_block_contents,
    };

    #[test]
    fn patch_managed_block_replaces_existing_managed_section_only() {
        let existing = format!(
            "live = true\n{}\nold = \"value\"\n{}\nother = 42\n",
            managed_block_start_marker(),
            managed_block_end_marker()
        );

        let patched = patch_managed_block_contents(&existing, "[colors]\nbackground = \"#000000\"");

        assert_eq!(
            patched,
            format!(
                "live = true\n{}\n[colors]\nbackground = \"#000000\"\n{}\nother = 42\n",
                managed_block_start_marker(),
                managed_block_end_marker()
            )
        );
    }

    #[test]
    fn patch_managed_block_appends_section_when_markers_are_missing() {
        let patched =
            patch_managed_block_contents("live = true\n", "[colors]\nforeground = \"#ffffff\"");

        assert_eq!(
            patched,
            format!(
                "live = true\n\n{}\n[colors]\nforeground = \"#ffffff\"\n{}\n",
                managed_block_start_marker(),
                managed_block_end_marker()
            )
        );
    }

    #[test]
    fn patch_managed_block_initializes_missing_files_with_managed_section() {
        let patched = patch_managed_block_contents("", "[colors]\nforeground = \"#ffffff\"");

        assert_eq!(
            patched,
            format!(
                "{}\n[colors]\nforeground = \"#ffffff\"\n{}\n",
                managed_block_start_marker(),
                managed_block_end_marker()
            )
        );
    }
}
