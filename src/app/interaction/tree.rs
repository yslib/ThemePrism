use crate::app::state::AppState;
use crate::app::view::{panel_order, workspace_layout_for_tab};
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::app::workspace::{PanelId::*, WorkspaceTab::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceId {
    AppRoot,
    MainWindow,
    TokensPanel,
    ParamsPanel,
    PreviewPanel,
    PreviewTabs,
    PreviewBody,
    PalettePanel,
    ResolvedPrimaryPanel,
    ResolvedSecondaryPanel,
    InspectorPanel,
    InteractionInspectorPanel,
    ProjectConfigPanel,
    ExportTargetsPanel,
    EditorPreferencesPanel,
    NumericEditorSurface,
    SourcePicker,
    ConfigDialog,
    CommandPalette,
    ShortcutHelp,
}

impl SurfaceId {
    pub const fn panel_id(self) -> Option<PanelId> {
        match self {
            Self::TokensPanel => Some(Tokens),
            Self::ParamsPanel => Some(Params),
            Self::PreviewPanel | Self::PreviewTabs | Self::PreviewBody => Some(Preview),
            Self::PalettePanel => Some(Palette),
            Self::ResolvedPrimaryPanel => Some(ResolvedPrimary),
            Self::ResolvedSecondaryPanel => Some(ResolvedSecondary),
            Self::InspectorPanel => Some(Inspector),
            Self::InteractionInspectorPanel => Some(InteractionInspector),
            Self::ProjectConfigPanel => Some(ProjectConfig),
            Self::ExportTargetsPanel => Some(ExportTargets),
            Self::EditorPreferencesPanel => Some(EditorPreferences),
            _ => None,
        }
    }

    pub const fn workspace_surface(panel: PanelId) -> Self {
        match panel {
            Tokens => Self::TokensPanel,
            Params => Self::ParamsPanel,
            Preview => Self::PreviewPanel,
            Palette => Self::PalettePanel,
            ResolvedPrimary => Self::ResolvedPrimaryPanel,
            ResolvedSecondary => Self::ResolvedSecondaryPanel,
            Inspector => Self::InspectorPanel,
            InteractionInspector => Self::InteractionInspectorPanel,
            ProjectConfig => Self::ProjectConfigPanel,
            ExportTargets => Self::ExportTargetsPanel,
            EditorPreferences => Self::EditorPreferencesPanel,
        }
    }

    pub const fn is_workspace_panel(self) -> bool {
        matches!(
            self,
            Self::TokensPanel
                | Self::ParamsPanel
                | Self::PreviewPanel
                | Self::PreviewTabs
                | Self::PreviewBody
                | Self::PalettePanel
                | Self::ResolvedPrimaryPanel
                | Self::ResolvedSecondaryPanel
                | Self::InspectorPanel
                | Self::InteractionInspectorPanel
                | Self::ProjectConfigPanel
                | Self::ExportTargetsPanel
                | Self::EditorPreferencesPanel
        )
    }

    pub const fn is_workspace_surface(self) -> bool {
        matches!(self, Self::MainWindow) || self.is_workspace_panel()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabScope {
    Global,
    Workspace(WorkspaceTab),
    PreviewLocal,
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultAction {
    None,
    Activate,
    Open,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildNavigation {
    None,
    Sequential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubblePolicy {
    Bubble,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceNode {
    pub id: SurfaceId,
    pub parent: Option<SurfaceId>,
    pub children: Vec<SurfaceId>,
    pub focusable: bool,
    pub visible: bool,
    pub tab_scope_owner: Option<SurfaceId>,
    pub tab_scope: TabScope,
    pub default_action: DefaultAction,
    pub child_navigation: ChildNavigation,
    pub bubble_policy: BubblePolicy,
    pub view_anchor: Option<SurfaceId>,
}

impl SurfaceNode {
    pub fn new(
        id: SurfaceId,
        parent: Option<SurfaceId>,
        children: Vec<SurfaceId>,
        focusable: bool,
        visible: bool,
        tab_scope_owner: Option<SurfaceId>,
        tab_scope: TabScope,
        default_action: DefaultAction,
        child_navigation: ChildNavigation,
        bubble_policy: BubblePolicy,
        view_anchor: Option<SurfaceId>,
    ) -> Self {
        Self {
            id,
            parent,
            children,
            focusable,
            visible,
            tab_scope_owner,
            tab_scope,
            default_action,
            child_navigation,
            bubble_policy,
            view_anchor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionTree {
    nodes: Vec<SurfaceNode>,
}

impl InteractionTree {
    pub fn new(nodes: Vec<SurfaceNode>) -> Self {
        Self { nodes }
    }

    pub fn parent_of(&self, id: SurfaceId) -> Option<SurfaceId> {
        self.node(id).and_then(|node| node.parent)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_visible(&self, id: SurfaceId) -> bool {
        let Some(node) = self.node(id) else {
            return false;
        };
        node.visible
            && node
                .parent
                .map(|parent| self.is_visible(parent))
                .unwrap_or(true)
    }

    pub fn node(&self, id: SurfaceId) -> Option<&SurfaceNode> {
        self.nodes.iter().find(|node| node.id == id)
    }
}

pub fn build_interaction_tree(state: &AppState) -> InteractionTree {
    let visible_panels = panel_order(&workspace_layout_for_tab(state.ui.active_tab));
    let is_visible = |panel: PanelId| visible_panels.contains(&panel);
    let workspace_parent = |panel: PanelId| {
        if is_visible(panel) {
            Some(SurfaceId::MainWindow)
        } else {
            None
        }
    };
    let workspace_children = visible_panels
        .iter()
        .copied()
        .map(SurfaceId::workspace_surface)
        .collect::<Vec<_>>();
    let numeric_parent = modal_parent(state, SurfaceId::NumericEditorSurface);
    let source_picker_parent = modal_parent(state, SurfaceId::SourcePicker);
    let config_parent = modal_parent(state, SurfaceId::ConfigDialog);
    let command_palette_parent = modal_parent(state, SurfaceId::CommandPalette);
    let shortcut_help_parent = modal_parent(state, SurfaceId::ShortcutHelp);
    let modal_children = |surface: SurfaceId| {
        [
            (SurfaceId::NumericEditorSurface, numeric_parent),
            (SurfaceId::SourcePicker, source_picker_parent),
            (SurfaceId::ConfigDialog, config_parent),
            (SurfaceId::CommandPalette, command_palette_parent),
            (SurfaceId::ShortcutHelp, shortcut_help_parent),
        ]
        .into_iter()
        .filter_map(|(modal, parent)| (parent == Some(surface)).then_some(modal))
        .collect::<Vec<_>>()
    };
    let mut app_root_children = vec![SurfaceId::MainWindow];
    app_root_children.extend(modal_children(SurfaceId::AppRoot));
    let mut main_window_children = workspace_children.clone();
    main_window_children.extend(modal_children(SurfaceId::MainWindow));

    let nodes = vec![
        SurfaceNode::new(
            SurfaceId::AppRoot,
            None,
            app_root_children,
            false,
            true,
            None,
            TabScope::Global,
            DefaultAction::None,
            ChildNavigation::None,
            BubblePolicy::Stop,
            None,
        ),
        SurfaceNode::new(
            SurfaceId::MainWindow,
            Some(SurfaceId::AppRoot),
            main_window_children,
            true,
            true,
            None,
            TabScope::Global,
            DefaultAction::None,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::AppRoot),
        ),
        SurfaceNode::new(
            SurfaceId::TokensPanel,
            workspace_parent(Tokens),
            modal_children(SurfaceId::TokensPanel),
            true,
            is_visible(Tokens),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::ParamsPanel,
            workspace_parent(Params),
            modal_children(SurfaceId::ParamsPanel),
            true,
            is_visible(Params),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Edit,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::PreviewPanel,
            workspace_parent(Preview),
            {
                let mut children = vec![SurfaceId::PreviewTabs, SurfaceId::PreviewBody];
                children.extend(modal_children(SurfaceId::PreviewPanel));
                children
            },
            true,
            is_visible(Preview),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::Sequential,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::PreviewTabs,
            Some(SurfaceId::PreviewPanel),
            modal_children(SurfaceId::PreviewTabs),
            true,
            is_visible(Preview),
            None,
            TabScope::PreviewLocal,
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::PreviewPanel),
        ),
        SurfaceNode::new(
            SurfaceId::PreviewBody,
            Some(SurfaceId::PreviewPanel),
            modal_children(SurfaceId::PreviewBody),
            true,
            is_visible(Preview),
            Some(SurfaceId::PreviewTabs),
            TabScope::Workspace(Theme),
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::PreviewPanel),
        ),
        SurfaceNode::new(
            SurfaceId::ResolvedPrimaryPanel,
            workspace_parent(ResolvedPrimary),
            modal_children(SurfaceId::ResolvedPrimaryPanel),
            true,
            is_visible(ResolvedPrimary),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::ResolvedSecondaryPanel,
            workspace_parent(ResolvedSecondary),
            modal_children(SurfaceId::ResolvedSecondaryPanel),
            true,
            is_visible(ResolvedSecondary),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::PalettePanel,
            workspace_parent(Palette),
            modal_children(SurfaceId::PalettePanel),
            true,
            is_visible(Palette),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::InspectorPanel,
            workspace_parent(Inspector),
            modal_children(SurfaceId::InspectorPanel),
            true,
            is_visible(Inspector),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::InteractionInspectorPanel,
            workspace_parent(InteractionInspector),
            modal_children(SurfaceId::InteractionInspectorPanel),
            true,
            is_visible(InteractionInspector),
            None,
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::ProjectConfigPanel,
            workspace_parent(ProjectConfig),
            modal_children(SurfaceId::ProjectConfigPanel),
            true,
            is_visible(ProjectConfig),
            None,
            TabScope::Workspace(Project),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::ExportTargetsPanel,
            workspace_parent(ExportTargets),
            modal_children(SurfaceId::ExportTargetsPanel),
            true,
            is_visible(ExportTargets),
            None,
            TabScope::Workspace(Project),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::EditorPreferencesPanel,
            workspace_parent(EditorPreferences),
            modal_children(SurfaceId::EditorPreferencesPanel),
            true,
            is_visible(EditorPreferences),
            None,
            TabScope::Workspace(Project),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::NumericEditorSurface,
            numeric_parent,
            modal_children(SurfaceId::NumericEditorSurface),
            true,
            state.ui.text_input.is_some(),
            None,
            TabScope::Modal,
            DefaultAction::Edit,
            ChildNavigation::None,
            BubblePolicy::Stop,
            numeric_parent,
        ),
        SurfaceNode::new(
            SurfaceId::SourcePicker,
            source_picker_parent,
            modal_children(SurfaceId::SourcePicker),
            true,
            state.ui.source_picker.is_some(),
            None,
            TabScope::Modal,
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Stop,
            source_picker_parent,
        ),
        SurfaceNode::new(
            SurfaceId::ConfigDialog,
            config_parent,
            modal_children(SurfaceId::ConfigDialog),
            true,
            state.ui.config_modal.is_some(),
            None,
            TabScope::Modal,
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Stop,
            config_parent,
        ),
        SurfaceNode::new(
            SurfaceId::CommandPalette,
            command_palette_parent,
            modal_children(SurfaceId::CommandPalette),
            true,
            state.ui.command_palette.is_some(),
            None,
            TabScope::Modal,
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Stop,
            command_palette_parent,
        ),
        SurfaceNode::new(
            SurfaceId::ShortcutHelp,
            shortcut_help_parent,
            modal_children(SurfaceId::ShortcutHelp),
            true,
            state.ui.shortcut_help_open,
            None,
            TabScope::Modal,
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Stop,
            shortcut_help_parent,
        ),
    ];

    InteractionTree::new(nodes)
}

fn modal_parent(state: &AppState, modal: SurfaceId) -> Option<SurfaceId> {
    if !modal_is_visible(state, modal) {
        return None;
    }

    let focus_path = &state.ui.interaction.focus_path;
    if let Some(index) = focus_path.iter().position(|surface| *surface == modal) {
        return index
            .checked_sub(1)
            .and_then(|parent| focus_path.get(parent).copied());
    }

    if matches!(
        state.ui.interaction.current_mode(),
        crate::app::interaction::InteractionMode::Modal { owner } if owner == modal
    ) {
        return focus_path.last().copied().or(Some(SurfaceId::MainWindow));
    }

    focus_path.last().copied().or(Some(SurfaceId::MainWindow))
}

fn modal_is_visible(state: &AppState, modal: SurfaceId) -> bool {
    match modal {
        SurfaceId::NumericEditorSurface => state.ui.text_input.is_some(),
        SurfaceId::SourcePicker => state.ui.source_picker.is_some(),
        SurfaceId::ConfigDialog => state.ui.config_modal.is_some(),
        SurfaceId::CommandPalette => state.ui.command_palette.is_some(),
        SurfaceId::ShortcutHelp => state.ui.shortcut_help_open,
        _ => false,
    }
}
