pub mod animation;
pub mod config;
pub mod controller;
pub mod state_layer;

#[derive(Clone, Copy, Debug)]
pub enum InteractionEvent {
    HoverEnter,
    HoverExit,
    FocusGained,
    FocusLost,
    PressStart(Option<gpui::Point<gpui::Pixels>>),
    PressRelease,
    PressCancel,
    DragStart,
    DragStop,
    DragCancel,
}

pub fn is_activation_key(key: &str) -> bool {
    matches!(key, "enter" | "space" | " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_and_space_activate() {
        assert!(is_activation_key("enter"));
        assert!(is_activation_key("space"));
        assert!(is_activation_key(" "));
        assert!(!is_activation_key("a"));
    }
}
