use gpui::Hsla;

use crate::tokens::state::StateLayer;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InsetRingGeometry {
    pub outer_inset: f32,
    pub outer_width: f32,
    pub inner_inset: f32,
    pub inner_width: f32,
}

pub const INSET_FOCUS_RING: InsetRingGeometry = InsetRingGeometry {
    outer_inset: 0.0,
    outer_width: 2.0,
    inner_inset: 1.0,
    inner_width: 3.0,
};

#[derive(Clone, Copy, Debug)]
pub enum FocusIndication {
    None,
    Opacity(f32),
    InsetRing {
        geometry: InsetRingGeometry,
        outer_color: Hsla,
        inner_color: Hsla,
        corners: [f32; 4],
    },
}

impl FocusIndication {
    pub fn default_opacity() -> Self {
        Self::Opacity(StateLayer::Focus.opacity())
    }

    pub fn with_corners(self, corners: [f32; 4]) -> Self {
        match self {
            Self::InsetRing {
                geometry,
                outer_color,
                inner_color,
                ..
            } => Self::InsetRing {
                geometry,
                outer_color,
                inner_color,
                corners,
            },
            other => other,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RippleConfig {
    pub bounded: bool,
    pub radius: Option<f32>,
    pub color: Hsla,
    pub press_alpha: Option<f32>,
    pub hover_alpha: Option<f32>,
    pub drag_alpha: Option<f32>,
    pub focus: FocusIndication,
}

impl RippleConfig {
    pub fn new(color: Hsla, bounded: bool) -> Self {
        Self {
            bounded,
            radius: None,
            color,
            press_alpha: Some(StateLayer::Press.opacity()),
            hover_alpha: Some(StateLayer::Hover.opacity()),
            drag_alpha: Some(StateLayer::Drag.opacity()),
            focus: FocusIndication::default_opacity(),
        }
    }

    pub fn without_press(mut self) -> Self {
        self.press_alpha = None;
        self
    }

    pub fn without_hover(mut self) -> Self {
        self.hover_alpha = None;
        self
    }

    pub fn without_drag(mut self) -> Self {
        self.drag_alpha = None;
        self
    }

    pub fn without_focus(mut self) -> Self {
        self.focus = FocusIndication::None;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_state_layer_tokens() {
        let config = RippleConfig::new(gpui::black(), true);
        assert_eq!(config.press_alpha, Some(0.10));
        assert_eq!(config.hover_alpha, Some(0.08));
        assert_eq!(config.drag_alpha, Some(0.16));
        assert!(matches!(config.focus, FocusIndication::Opacity(alpha) if alpha == 0.10));
    }

    #[test]
    fn indications_can_be_disabled_individually() {
        let config = RippleConfig::new(gpui::black(), true)
            .without_press()
            .without_hover()
            .without_drag()
            .without_focus();
        assert!(config.press_alpha.is_none());
        assert!(config.hover_alpha.is_none());
        assert!(config.drag_alpha.is_none());
        assert!(matches!(config.focus, FocusIndication::None));
    }

    #[test]
    fn inset_focus_ring_matches_the_reference_preset() {
        assert_eq!(INSET_FOCUS_RING.outer_inset, 0.0);
        assert_eq!(INSET_FOCUS_RING.outer_width, 2.0);
        assert_eq!(INSET_FOCUS_RING.inner_inset, 1.0);
        assert_eq!(INSET_FOCUS_RING.inner_width, 3.0);
    }
}
