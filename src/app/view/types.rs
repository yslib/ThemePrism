use crate::app::actions::ActionHint;
use crate::app::controls::ControlSpec;
use crate::app::workspace::PanelId;
use crate::domain::color::Color;

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Length(u16),
    Min(u16),
    Percentage(u16),
}

#[derive(Debug, Clone)]
pub struct ViewTheme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub selection: Color,
    pub text: Color,
    pub text_muted: Color,
}

#[derive(Debug, Clone)]
pub struct ViewTree {
    pub theme: ViewTheme,
    pub main_window: MainWindowView,
    pub overlays: Vec<OverlayView>,
}

#[derive(Debug, Clone)]
pub struct MainWindowView {
    pub menu_bar: MenuBarView,
    pub tab_bar: TabBarView,
    pub workspace: ViewNode,
    pub status_bar: StatusBarView,
}

#[derive(Debug, Clone)]
pub struct MenuBarView {
    pub title: String,
    pub actions: Vec<ActionHint>,
}

#[derive(Debug, Clone)]
pub struct TabBarView {
    pub tabs: Vec<TabItemView>,
}

#[derive(Debug, Clone)]
pub struct TabItemView {
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub enum ViewNode {
    Split(SplitView),
    Panel(PanelView),
    StatusBar(StatusBarView),
}

#[derive(Debug, Clone)]
pub struct SplitView {
    pub axis: Axis,
    pub constraints: Vec<Size>,
    pub children: Vec<ViewNode>,
}

#[derive(Debug, Clone)]
pub struct PanelView {
    pub id: PanelId,
    pub title: String,
    pub active: bool,
    pub shortcut: Option<u8>,
    pub tabs: Vec<PanelTabView>,
    pub header_lines: Vec<StyledLine>,
    pub body: PanelBody,
}

#[derive(Debug, Clone)]
pub struct PanelTabView {
    pub label: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub enum PanelBody {
    SelectionList(SelectionListView),
    Form(FormView),
    Document(DocumentView),
    SwatchList(SwatchListView),
}

#[derive(Debug, Clone)]
pub struct SelectionListView {
    pub rows: Vec<SelectionRowView>,
}

#[derive(Debug, Clone)]
pub enum SelectionRowView {
    Header(String),
    Item {
        label: String,
        color: Color,
        selected: bool,
    },
}

#[derive(Debug, Clone)]
pub struct FormView {
    pub header_lines: Vec<StyledLine>,
    pub fields: Vec<FormFieldView>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FormFieldView {
    pub control: ControlSpec,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct DocumentView {
    pub lines: Vec<StyledLine>,
    pub scroll: u16,
}

#[derive(Debug, Clone)]
pub struct SwatchListView {
    pub items: Vec<SwatchItemView>,
}

#[derive(Debug, Clone)]
pub struct SwatchItemView {
    pub label: String,
    pub color: Color,
    pub value_text: String,
}

#[derive(Debug, Clone)]
pub struct StatusBarView {
    pub focus_label: String,
    pub status_text: String,
}

#[derive(Debug, Clone)]
pub enum OverlayView {
    Picker(PickerOverlayView),
    Surface(SurfaceView),
}

#[derive(Debug, Clone)]
pub struct PickerOverlayView {
    pub title: String,
    pub filter: String,
    pub rows: Vec<PickerRowView>,
    pub selected_row: Option<usize>,
    pub total_matches: usize,
}

#[derive(Debug, Clone)]
pub struct PickerRowView {
    pub label: String,
    pub is_header: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceSize {
    Percent { width: u16, height: u16 },
    Absolute { width: u16, height: u16 },
}

#[derive(Debug, Clone)]
pub struct SurfaceView {
    pub title: String,
    pub size: SurfaceSize,
    pub body: SurfaceBody,
    pub footer_lines: Vec<StyledLine>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SurfaceBody {
    Lines { lines: Vec<StyledLine>, scroll: u16 },
    Node(Box<ViewNode>),
    Window(Box<MainWindowView>),
}

#[derive(Debug, Clone)]
pub struct StyledLine {
    pub spans: Vec<StyledSpan>,
}

#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub style: SpanStyle,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SpanStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}
