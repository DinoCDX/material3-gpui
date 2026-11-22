use std::time::Duration;

pub const RIPPLE_FADE_IN: Duration = Duration::from_millis(75);
pub const RIPPLE_RADIUS_DURATION: Duration = Duration::from_millis(225);
pub const RIPPLE_FADE_OUT: Duration = Duration::from_millis(150);
pub const RIPPLE_START_FRACTION: f32 = 0.3;
pub const BOUNDED_RIPPLE_EXTRA_DP: f32 = 10.0;
pub const FAST_OUT_SLOW_IN_CONTROL_POINTS: (f32, f32, f32, f32) = (0.4, 0.0, 0.2, 1.0);

pub const STATE_LAYER_HOVER_IN: Duration = Duration::from_millis(15);
pub const STATE_LAYER_HOVER_OUT: Duration = Duration::from_millis(15);
pub const STATE_LAYER_FOCUS_IN: Duration = Duration::from_millis(45);
pub const STATE_LAYER_FOCUS_OUT: Duration = Duration::from_millis(15);
pub const STATE_LAYER_DRAG_IN: Duration = Duration::from_millis(45);
pub const STATE_LAYER_DRAG_OUT: Duration = Duration::from_millis(150);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringSpec {
    pub damping_ratio: f32,
    pub stiffness: f32,
}

pub const STANDARD_FAST_SPATIAL: SpringSpec = SpringSpec {
    damping_ratio: 0.9,
    stiffness: 1400.0,
};

pub const STANDARD_FAST_EFFECTS: SpringSpec = SpringSpec {
    damping_ratio: 1.0,
    stiffness: 3800.0,
};

pub const FOCUS_RING_FOCUSING_SPEC: SpringSpec = STANDARD_FAST_SPATIAL;
pub const FOCUS_RING_UNFOCUSING_SPEC: SpringSpec = STANDARD_FAST_EFFECTS;
