use crate::AppState;
use crate::app::workspace::PanelId;
use crate::i18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    SaveProject,
    LoadProject,
    ExportTheme,
    Reset,
    OpenConfig,
    OpenHelp,
    ToggleFullscreen,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItem {
    pub id: CommandId,
    pub title: String,
    pub keywords: Vec<String>,
    pub context_score: u16,
    pub enabled: bool,
}

pub fn command_items(state: &AppState) -> Vec<CommandItem> {
    const COMMAND_IDS: [CommandId; 8] = [
        CommandId::SaveProject,
        CommandId::LoadProject,
        CommandId::ExportTheme,
        CommandId::Reset,
        CommandId::OpenConfig,
        CommandId::OpenHelp,
        CommandId::ToggleFullscreen,
        CommandId::Quit,
    ];

    COMMAND_IDS
        .into_iter()
        .map(|id| build_command_item(state, id))
        .collect()
}

pub fn filter_commands(state: &AppState, query: &str) -> Vec<CommandItem> {
    let query = normalize_for_match(query.trim());
    let items = command_items(state);
    let mut ranked: Vec<(usize, u8, u16, CommandItem)> = items
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let match_rank = match_command(&item, &query)?;
            Some((index, match_rank, item.context_score, item))
        })
        .collect();

    ranked.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then_with(|| right.2.cmp(&left.2))
            .then_with(|| left.0.cmp(&right.0))
    });

    ranked.into_iter().map(|(_, _, _, item)| item).collect()
}

fn match_command(item: &CommandItem, query: &str) -> Option<u8> {
    if query.is_empty() {
        return Some(0);
    }

    let title = normalize_for_match(&item.title);
    if title.contains(query) {
        return Some(0);
    }

    let keywords = item
        .keywords
        .iter()
        .map(|keyword| normalize_for_match(keyword))
        .collect::<Vec<_>>();
    if keywords.iter().any(|keyword| keyword.contains(query)) {
        return Some(1);
    }

    if is_subsequence(&title, query) {
        return Some(2);
    }

    if keywords
        .iter()
        .any(|keyword| is_subsequence(keyword, query))
    {
        return Some(3);
    }

    None
}

fn build_command_item(state: &AppState, id: CommandId) -> CommandItem {
    let text = i18n::command_text(state.locale(), id);
    CommandItem {
        id,
        title: text.label,
        keywords: text.keywords,
        context_score: context_score(state.active_panel(), id),
        enabled: true,
    }
}

fn context_score(active_panel: PanelId, command: CommandId) -> u16 {
    match command {
        CommandId::OpenConfig
            if matches!(
                active_panel,
                PanelId::ProjectConfig | PanelId::ExportTargets | PanelId::EditorPreferences
            ) =>
        {
            10
        }
        CommandId::ToggleFullscreen if active_panel == PanelId::Preview => 20,
        _ => 0,
    }
}

fn normalize_for_match(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}

fn is_subsequence(haystack: &str, needle: &str) -> bool {
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next();

    for haystack_char in haystack.chars() {
        if current == Some(haystack_char) {
            current = needle_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }

    current.is_none()
}

#[cfg(test)]
mod tests {
    use super::{CommandId, command_items, filter_commands};
    use crate::AppState;
    use crate::app::workspace::PanelId;
    use crate::persistence::editor_config::EditorLocale;

    #[test]
    fn command_provider_exposes_expected_core_commands() {
        let state = AppState::new().unwrap();
        let items = command_items(&state);

        assert!(items.iter().any(|item| item.id == CommandId::SaveProject));
        assert!(items.iter().any(|item| item.id == CommandId::OpenConfig));
        assert!(items.iter().any(|item| item.id == CommandId::Quit));
    }

    #[test]
    fn empty_query_keeps_context_commands_ahead_of_global_commands() {
        let mut state = AppState::new().unwrap();
        state.set_active_panel(PanelId::Preview);

        let ranked = filter_commands(&state, "");
        let preview_index = ranked
            .iter()
            .position(|item| item.id == CommandId::ToggleFullscreen)
            .unwrap();
        let help_index = ranked
            .iter()
            .position(|item| item.id == CommandId::OpenHelp)
            .unwrap();

        assert!(preview_index < help_index);
    }

    #[test]
    fn query_matching_is_case_insensitive_and_substring_friendly() {
        let state = AppState::new().unwrap();
        let ranked = filter_commands(&state, "expo");

        assert_eq!(
            ranked.first().map(|item| item.id),
            Some(CommandId::ExportTheme)
        );
    }

    #[test]
    fn zh_cn_command_labels_and_keywords_are_used_for_matching() {
        let mut state = AppState::new().unwrap();
        state.editor.locale = EditorLocale::ZhCn;

        let items = command_items(&state);
        assert_eq!(
            items
                .iter()
                .find(|item| item.id == CommandId::SaveProject)
                .map(|item| item.title.as_str()),
            Some("保存工程")
        );

        assert_eq!(
            filter_commands(&state, "保存").first().map(|item| item.id),
            Some(CommandId::SaveProject)
        );
        assert_eq!(
            filter_commands(&state, "设置").first().map(|item| item.id),
            Some(CommandId::OpenConfig)
        );
    }

    #[test]
    fn query_matching_has_a_subsequence_fallback() {
        let state = AppState::new().unwrap();

        assert_eq!(
            filter_commands(&state, "svpr").first().map(|item| item.id),
            Some(CommandId::SaveProject)
        );
    }
}
