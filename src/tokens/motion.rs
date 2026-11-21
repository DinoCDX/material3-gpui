use std::time::Duration;

pub const RIPPLE_FADE_IN: Duration = Duration::from_millis(75);
pub const RIPPLE_RADIUS_DURATION: Duration = Duration::from_millis(225);
pub const RIPPLE_FADE_OUT: Duration = Duration::from_millis(150);
pub const RIPPLE_START_FRACTION: f32 = 0.3;
pub const BOUNDED_RIPPLE_EXTRA_DP: f32 = 10.0;
pub const FAST_OUT_SLOW_IN_CONTROL_POINTS: (f32, f32, f32, f32) = (0.4, 0.0, 0.2, 1.0);
