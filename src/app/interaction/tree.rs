use crate::app::state::AppState;
use crate::app::workspace::{PanelId, WorkspaceTab};
use crate::app::workspace::{PanelId::*, WorkspaceTab::*};
use crate::app::view::{panel_order, workspace_layout_for_tab};

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
    InspectorPanel,
    ProjectConfigPanel,
    ExportTargetsPanel,
    EditorPreferencesPanel,
    NumericEditorSurface,
    SourcePicker,
    ConfigDialog,
    ShortcutHelp,
}

impl SurfaceId {
    pub const fn panel_id(self) -> Option<PanelId> {
        match self {
            Self::TokensPanel => Some(Tokens),
            Self::ParamsPanel => Some(Params),
            Self::PreviewPanel | Self::PreviewTabs | Self::PreviewBody => Some(Preview),
            Self::PalettePanel => Some(Palette),
            Self::InspectorPanel => Some(Inspector),
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
            ResolvedPrimary | ResolvedSecondary => Self::PreviewPanel,
            Inspector => Self::InspectorPanel,
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
                | Self::InspectorPanel
                | Self::ProjectConfigPanel
                | Self::ExportTargetsPanel
                | Self::EditorPreferencesPanel
        )
    }

    pub const fn is_workspace_surface(self) -> bool {
        matches!(self, Self::MainWindow) || self.is_workspace_panel()
    }

    pub const fn is_modal_surface(self) -> bool {
        matches!(
            self,
            Self::NumericEditorSurface
                | Self::SourcePicker
                | Self::ConfigDialog
                | Self::ShortcutHelp
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabScope {
    Global,
    Workspace(WorkspaceTab),
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
    Numbered,
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

    let nodes = vec![
        SurfaceNode::new(
            SurfaceId::AppRoot,
            None,
            vec![
                SurfaceId::MainWindow,
                SurfaceId::NumericEditorSurface,
                SurfaceId::SourcePicker,
                SurfaceId::ConfigDialog,
                SurfaceId::ShortcutHelp,
            ],
            false,
            true,
            TabScope::Global,
            DefaultAction::None,
            ChildNavigation::None,
            BubblePolicy::Stop,
            None,
        ),
        SurfaceNode::new(
            SurfaceId::MainWindow,
            Some(SurfaceId::AppRoot),
            vec![
                SurfaceId::TokensPanel,
                SurfaceId::ParamsPanel,
                SurfaceId::PreviewPanel,
                SurfaceId::PalettePanel,
                SurfaceId::InspectorPanel,
                SurfaceId::ProjectConfigPanel,
                SurfaceId::ExportTargetsPanel,
                SurfaceId::EditorPreferencesPanel,
            ],
            true,
            true,
            TabScope::Global,
            DefaultAction::Activate,
            ChildNavigation::Numbered,
            BubblePolicy::Bubble,
            Some(SurfaceId::AppRoot),
        ),
        SurfaceNode::new(
            SurfaceId::TokensPanel,
            Some(SurfaceId::MainWindow),
            vec![],
            true,
            is_visible(Tokens),
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::ParamsPanel,
            Some(SurfaceId::MainWindow),
            vec![],
            true,
            is_visible(Params),
            TabScope::Workspace(Theme),
            DefaultAction::Edit,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::PreviewPanel,
            Some(SurfaceId::MainWindow),
            vec![SurfaceId::PreviewTabs, SurfaceId::PreviewBody],
            true,
            is_visible(Preview),
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::Sequential,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::PreviewTabs,
            Some(SurfaceId::PreviewPanel),
            vec![],
            true,
            is_visible(Preview),
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::PreviewPanel),
        ),
        SurfaceNode::new(
            SurfaceId::PreviewBody,
            Some(SurfaceId::PreviewPanel),
            vec![],
            true,
            is_visible(Preview),
            TabScope::Workspace(Theme),
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::PreviewPanel),
        ),
        SurfaceNode::new(
            SurfaceId::PalettePanel,
            Some(SurfaceId::MainWindow),
            vec![],
            true,
            is_visible(Palette),
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::InspectorPanel,
            Some(SurfaceId::MainWindow),
            vec![],
            true,
            is_visible(Inspector),
            TabScope::Workspace(Theme),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::ProjectConfigPanel,
            Some(SurfaceId::MainWindow),
            vec![],
            true,
            is_visible(ProjectConfig),
            TabScope::Workspace(Project),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::ExportTargetsPanel,
            Some(SurfaceId::MainWindow),
            vec![],
            true,
            is_visible(ExportTargets),
            TabScope::Workspace(Project),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::EditorPreferencesPanel,
            Some(SurfaceId::MainWindow),
            vec![],
            true,
            is_visible(EditorPreferences),
            TabScope::Workspace(Project),
            DefaultAction::Activate,
            ChildNavigation::None,
            BubblePolicy::Bubble,
            Some(SurfaceId::MainWindow),
        ),
        SurfaceNode::new(
            SurfaceId::NumericEditorSurface,
            Some(SurfaceId::AppRoot),
            vec![],
            true,
            state.ui.text_input.is_some(),
            TabScope::Modal,
            DefaultAction::Edit,
            ChildNavigation::None,
            BubblePolicy::Stop,
            Some(SurfaceId::AppRoot),
        ),
        SurfaceNode::new(
            SurfaceId::SourcePicker,
            Some(SurfaceId::AppRoot),
            vec![],
            true,
            state.ui.source_picker.is_some(),
            TabScope::Modal,
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Stop,
            Some(SurfaceId::AppRoot),
        ),
        SurfaceNode::new(
            SurfaceId::ConfigDialog,
            Some(SurfaceId::AppRoot),
            vec![],
            true,
            state.ui.config_modal.is_some(),
            TabScope::Modal,
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Stop,
            Some(SurfaceId::AppRoot),
        ),
        SurfaceNode::new(
            SurfaceId::ShortcutHelp,
            Some(SurfaceId::AppRoot),
            vec![],
            true,
            state.ui.shortcut_help_open,
            TabScope::Modal,
            DefaultAction::Open,
            ChildNavigation::None,
            BubblePolicy::Stop,
            Some(SurfaceId::AppRoot),
        ),
    ];

    InteractionTree::new(nodes)
}
