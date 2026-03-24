use crate::app::interaction::{
    BubblePolicy, ChildNavigation, DefaultAction, InteractionMode, InteractionTree, SurfaceId,
    SurfaceNode, TabScope, build_interaction_tree, effective_focus_path,
};
use crate::app::state::AppState;
use crate::app::workspace::PanelId;
use crate::domain::tokens::TokenRole;
use crate::i18n::{self, UiText};

use super::styled::{colored_span, plain_span};
use super::{DocumentView, PanelBody, PanelView, SpanStyle, StyledLine, StyledSpan};

pub(crate) fn build_interaction_panel(state: &AppState) -> PanelView {
    let tree = build_interaction_tree(state);
    let focus_path = effective_focus_path(state);
    let focused = focus_path.last().copied();
    let locale = state.locale();
    let mut lines = Vec::new();

    lines.push(section_header(
        state,
        &i18n::text(locale, UiText::InteractionInspectorFocusPath),
    ));
    for surface in &focus_path {
        let is_current = Some(*surface) == focused;
        lines.push(text_line(
            state,
            if is_current { "* " } else { "- " },
            &format!("{surface:?}"),
            is_current,
        ));
    }

    lines.push(StyledLine { spans: Vec::new() });
    lines.push(section_header(
        state,
        &i18n::text(locale, UiText::InteractionInspectorModeStack),
    ));
    for mode in state.ui.interaction.mode_stack() {
        let is_current = *mode == state.ui.interaction.current_mode();
        lines.push(text_line(
            state,
            if is_current { "* " } else { "- " },
            &format_mode(*mode),
            is_current,
        ));
    }

    lines.push(StyledLine { spans: Vec::new() });
    lines.push(section_header(
        state,
        &i18n::text(locale, UiText::InteractionInspectorTree),
    ));
    append_tree_lines(
        state,
        &tree,
        SurfaceId::AppRoot,
        0,
        &focus_path,
        focused,
        &mut lines,
    );

    PanelView {
        id: PanelId::InteractionInspector,
        title: i18n::text(locale, UiText::PanelInteractionInspector),
        active: false,
        shortcut: None,
        tabs: Vec::new(),
        header_lines: Vec::new(),
        body: PanelBody::Document(DocumentView { lines }),
    }
}

fn append_tree_lines(
    state: &AppState,
    tree: &InteractionTree,
    surface: SurfaceId,
    depth: usize,
    focus_path: &[SurfaceId],
    focused: Option<SurfaceId>,
    lines: &mut Vec<StyledLine>,
) {
    let Some(node) = tree.node(surface) else {
        return;
    };

    let in_focus_path = focus_path.contains(&surface);
    let is_current = Some(surface) == focused;
    let indent = "  ".repeat(depth);
    let marker = if is_current {
        "* "
    } else if in_focus_path {
        "> "
    } else {
        "  "
    };
    let mut spans = vec![tree_label_span(
        state,
        &format!("{indent}{marker}{surface:?}"),
        in_focus_path,
        is_current,
    )];
    for badge in capability_badges(node) {
        spans.push(plain_span(" "));
        spans.push(badge_span(state, &badge, in_focus_path));
    }
    lines.push(StyledLine { spans });

    for child in &node.children {
        append_tree_lines(state, tree, *child, depth + 1, focus_path, focused, lines);
    }
}

fn capability_badges(node: &SurfaceNode) -> Vec<String> {
    let mut badges = Vec::new();

    if node.focusable {
        badges.push("focus".to_string());
    }
    badges.push(match node.tab_scope {
        TabScope::Global => "tabs:global".to_string(),
        TabScope::Workspace(tab) => format!("tabs:{tab:?}"),
        TabScope::PreviewLocal => "tabs:preview".to_string(),
        TabScope::Modal => "tabs:modal".to_string(),
    });
    match node.default_action {
        DefaultAction::None => {}
        DefaultAction::Activate => badges.push("default:activate".to_string()),
        DefaultAction::Open => badges.push("default:open".to_string()),
        DefaultAction::Edit => badges.push("default:edit".to_string()),
    }
    match node.child_navigation {
        ChildNavigation::None => {}
        ChildNavigation::Numbered => badges.push("children:1-9".to_string()),
        ChildNavigation::Sequential => badges.push("children:seq".to_string()),
    }
    badges.push(match node.bubble_policy {
        BubblePolicy::Bubble => "bubble:up".to_string(),
        BubblePolicy::Stop => "bubble:stop".to_string(),
    });

    badges
}

fn format_mode(mode: InteractionMode) -> String {
    match mode {
        InteractionMode::Normal => "Normal".to_string(),
        InteractionMode::NavigateChildren(owner) => format!("NavigateChildren({owner:?})"),
        InteractionMode::Capture { owner } => format!("Capture({owner:?})"),
        InteractionMode::Modal { owner } => format!("Modal({owner:?})"),
    }
}

fn section_header(state: &AppState, text: &str) -> StyledLine {
    StyledLine {
        spans: vec![colored_span(
            text,
            state.theme_color(TokenRole::TextMuted),
            true,
            false,
        )],
    }
}

fn text_line(state: &AppState, prefix: &str, text: &str, highlight: bool) -> StyledLine {
    StyledLine {
        spans: vec![tree_label_span(
            state,
            &format!("{prefix}{text}"),
            highlight,
            highlight,
        )],
    }
}

fn tree_label_span(state: &AppState, text: &str, highlight: bool, bold: bool) -> StyledSpan {
    StyledSpan {
        text: text.to_string(),
        style: SpanStyle {
            fg: Some(if highlight {
                state.theme_color(TokenRole::Selection)
            } else {
                state.theme_color(TokenRole::Text)
            }),
            bg: None,
            bold,
            italic: false,
        },
    }
}

fn badge_span(state: &AppState, text: &str, highlight: bool) -> StyledSpan {
    StyledSpan {
        text: format!("[{text}]"),
        style: SpanStyle {
            fg: Some(if highlight {
                state.theme_color(TokenRole::Selection)
            } else {
                state.theme_color(TokenRole::TextMuted)
            }),
            bg: None,
            bold: false,
            italic: false,
        },
    }
}
