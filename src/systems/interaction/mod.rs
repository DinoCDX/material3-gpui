pub mod controller;
pub mod state_layer;

use crate::tokens::state::StateLayer;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InteractionFlags {
    pub hovered: bool,
    pub focused: bool,
    pub pressed: bool,
    pub dragged: bool,
    pub disabled: bool,
}

impl InteractionFlags {
    pub fn state_layer(self) -> Option<StateLayer> {
        if self.disabled {
            None
        } else if self.dragged {
            Some(StateLayer::Drag)
        } else if self.pressed {
            Some(StateLayer::Press)
        } else if self.focused {
            Some(StateLayer::Focus)
        } else if self.hovered {
            Some(StateLayer::Hover)
        } else {
            None
        }
    }
}

pub fn is_activation_key(key: &str) -> bool {
    matches!(key, "enter" | "space" | " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(
        hovered: bool,
        focused: bool,
        pressed: bool,
        dragged: bool,
        disabled: bool,
    ) -> InteractionFlags {
        InteractionFlags {
            hovered,
            focused,
            pressed,
            dragged,
            disabled,
        }
    }

    #[test]
    fn drag_outranks_press_focus_and_hover() {
        assert_eq!(flags(true, true, true, true, false).state_layer(), Some(StateLayer::Drag));
    }

    #[test]
    fn press_outranks_focus_and_hover() {
        assert_eq!(flags(true, true, true, false, false).state_layer(), Some(StateLayer::Press));
    }

    #[test]
    fn focus_outranks_hover() {
        assert_eq!(flags(true, true, false, false, false).state_layer(), Some(StateLayer::Focus));
    }

    #[test]
    fn hover_alone_applies_hover_layer() {
        assert_eq!(flags(true, false, false, false, false).state_layer(), Some(StateLayer::Hover));
    }

    #[test]
    fn idle_surface_has_no_layer() {
        assert_eq!(flags(false, false, false, false, false).state_layer(), None);
    }

    #[test]
    fn disabled_suppresses_every_layer() {
        assert_eq!(flags(true, true, true, true, true).state_layer(), None);
    }

    #[test]
    fn enter_and_space_activate() {
        assert!(is_activation_key("enter"));
        assert!(is_activation_key("space"));
        assert!(is_activation_key(" "));
        assert!(!is_activation_key("a"));
    }
}
