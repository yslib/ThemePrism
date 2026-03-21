#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GuiEventAdapter;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiNativeEvent {
    WindowRequestedClose,
    KeyPressed(String),
    ControlActivated(String),
}
