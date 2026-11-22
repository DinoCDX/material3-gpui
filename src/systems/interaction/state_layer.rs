use std::time::Duration;

use super::config::{FocusIndication, RippleConfig};
use crate::tokens::motion::{
    STATE_LAYER_DRAG_IN, STATE_LAYER_DRAG_OUT, STATE_LAYER_FOCUS_IN, STATE_LAYER_FOCUS_OUT,
    STATE_LAYER_HOVER_IN, STATE_LAYER_HOVER_OUT,
};
use crate::tokens::state::StateLayer;

pub fn layer_alpha(config: &RippleConfig, layer: StateLayer) -> Option<f32> {
    match layer {
        StateLayer::Hover => config.hover_alpha,
        StateLayer::Focus => match config.focus {
            FocusIndication::Opacity(alpha) => Some(alpha),
            FocusIndication::None | FocusIndication::InsetRing { .. } => None,
        },
        StateLayer::Drag => config.drag_alpha,
        StateLayer::Press => None,
    }
}

pub fn incoming_duration(layer: StateLayer) -> Duration {
    match layer {
        StateLayer::Hover | StateLayer::Press => STATE_LAYER_HOVER_IN,
        StateLayer::Focus => STATE_LAYER_FOCUS_IN,
        StateLayer::Drag => STATE_LAYER_DRAG_IN,
    }
}

pub fn outgoing_duration(layer: StateLayer) -> Duration {
    match layer {
        StateLayer::Hover | StateLayer::Press => STATE_LAYER_HOVER_OUT,
        StateLayer::Focus => STATE_LAYER_FOCUS_OUT,
        StateLayer::Drag => STATE_LAYER_DRAG_OUT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inset_ring_focus_has_no_opacity_layer() {
        let config = RippleConfig::new(gpui::black(), true).without_focus();
        assert_eq!(layer_alpha(&config, StateLayer::Focus), None);
    }

    #[test]
    fn press_is_drawn_by_ripples_not_a_state_layer() {
        let config = RippleConfig::new(gpui::black(), true);
        assert_eq!(layer_alpha(&config, StateLayer::Press), None);
    }

    #[test]
    fn timings_match_the_reference_tweens() {
        assert_eq!(incoming_duration(StateLayer::Hover), Duration::from_millis(15));
        assert_eq!(incoming_duration(StateLayer::Focus), Duration::from_millis(45));
        assert_eq!(outgoing_duration(StateLayer::Drag), Duration::from_millis(150));
        assert_eq!(outgoing_duration(StateLayer::Focus), Duration::from_millis(15));
    }
}
