use crate::AppState;
use crate::app::workspace::PanelId;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandItem {
    pub id: CommandId,
    pub title: &'static str,
    pub keywords: &'static [&'static str],
    pub context_score: u16,
    pub enabled: bool,
}

pub fn command_items(state: &AppState) -> Vec<CommandItem> {
    let active_panel = state.active_panel();
    vec![
        CommandItem {
            id: CommandId::SaveProject,
            title: "Save Project",
            keywords: &["save", "project"],
            context_score: 0,
            enabled: true,
        },
        CommandItem {
            id: CommandId::LoadProject,
            title: "Load Project",
            keywords: &["load", "project", "open"],
            context_score: 0,
            enabled: true,
        },
        CommandItem {
            id: CommandId::ExportTheme,
            title: "Export Theme",
            keywords: &["export", "theme"],
            context_score: 0,
            enabled: true,
        },
        CommandItem {
            id: CommandId::Reset,
            title: "Reset",
            keywords: &["defaults", "restore"],
            context_score: 0,
            enabled: true,
        },
        CommandItem {
            id: CommandId::OpenConfig,
            title: "Open Config",
            keywords: &["config", "settings"],
            context_score: match active_panel {
                PanelId::ProjectConfig | PanelId::ExportTargets | PanelId::EditorPreferences => 10,
                _ => 0,
            },
            enabled: true,
        },
        CommandItem {
            id: CommandId::OpenHelp,
            title: "Open Help",
            keywords: &["help", "shortcuts"],
            context_score: 0,
            enabled: true,
        },
        CommandItem {
            id: CommandId::ToggleFullscreen,
            title: "Toggle Fullscreen",
            keywords: &["fullscreen", "preview"],
            context_score: match active_panel {
                PanelId::Preview => 20,
                _ => 0,
            },
            enabled: true,
        },
        CommandItem {
            id: CommandId::Quit,
            title: "Quit",
            keywords: &["exit", "close"],
            context_score: 0,
            enabled: true,
        },
    ]
}

pub fn filter_commands(state: &AppState, query: &str) -> Vec<CommandItem> {
    let query = query.trim().to_ascii_lowercase();
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

    let title = item.title.to_ascii_lowercase();
    if title.contains(query) {
        return Some(0);
    }

    if item
        .keywords
        .iter()
        .any(|keyword| keyword.to_ascii_lowercase().contains(query))
    {
        return Some(1);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{CommandId, command_items, filter_commands};
    use crate::AppState;
    use crate::app::workspace::PanelId;

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
        let quit_index = ranked
            .iter()
            .position(|item| item.id == CommandId::Quit)
            .unwrap();

        assert!(preview_index < quit_index);
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
}
