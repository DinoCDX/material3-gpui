use std::time::{Duration, Instant};

use super::easing::fast_out_slow_in;
use crate::tokens::motion::{
    BOUNDED_RIPPLE_EXTRA_DP, RIPPLE_FADE_IN, RIPPLE_FADE_OUT, RIPPLE_RADIUS_DURATION,
    RIPPLE_START_FRACTION,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementRect {
    pub origin: (f32, f32),
    pub size: (f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RippleFrame {
    pub center: (f32, f32),
    pub radius: f32,
    pub alpha: f32,
}

#[derive(Clone, Debug)]
pub struct RippleAnimation {
    origin: Option<(f32, f32)>,
    bounded: bool,
    started_at: Instant,
    finished_at: Option<Instant>,
}

impl RippleAnimation {
    pub fn new(origin: Option<(f32, f32)>, bounded: bool, started_at: Instant) -> Self {
        Self {
            origin,
            bounded,
            started_at,
            finished_at: None,
        }
    }

    pub fn is_bounded(&self) -> bool {
        self.bounded
    }

    pub fn finish(&mut self, at: Instant) {
        if self.finished_at.is_none() {
            self.finished_at = Some(at);
        }
    }

    pub fn is_finished(&self, now: Instant) -> bool {
        self.fade_out_start().is_some_and(|start| now >= start + RIPPLE_FADE_OUT)
    }

    pub fn sample(&self, now: Instant, element: ElementRect) -> Option<RippleFrame> {
        if self.is_finished(now) {
            return None;
        }
        let elapsed = now.saturating_duration_since(self.started_at);
        let (width, height) = element.size;
        let start_radius = width.max(height) * RIPPLE_START_FRACTION;
        let end_radius = target_radius(width, height, self.bounded);
        let radius_progress = fast_out_slow_in()(progress(elapsed, RIPPLE_RADIUS_DURATION));
        let center_progress = progress(elapsed, RIPPLE_RADIUS_DURATION);
        let target = (width / 2.0, height / 2.0);
        let start = self.origin.map_or(target, |origin| {
            (origin.0 - element.origin.0, origin.1 - element.origin.1)
        });
        Some(RippleFrame {
            center: (
                lerp(start.0, target.0, center_progress),
                lerp(start.1, target.1, center_progress),
            ),
            radius: lerp(start_radius, end_radius, radius_progress),
            alpha: self.alpha(now, elapsed),
        })
    }

    fn fade_out_start(&self) -> Option<Instant> {
        self.finished_at.map(|finished| finished.max(self.started_at + RIPPLE_FADE_IN))
    }

    fn alpha(&self, now: Instant, elapsed: Duration) -> f32 {
        match self.fade_out_start() {
            None => progress(elapsed, RIPPLE_FADE_IN),
            Some(start) if now < start => 1.0,
            Some(start) => 1.0 - progress(now.saturating_duration_since(start), RIPPLE_FADE_OUT),
        }
    }
}

fn progress(elapsed: Duration, duration: Duration) -> f32 {
    (elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
}

fn target_radius(width: f32, height: f32, bounded: bool) -> f32 {
    let covering = (width * width + height * height).sqrt() / 2.0;
    if bounded {
        covering + BOUNDED_RIPPLE_EXTRA_DP
    } else {
        covering
    }
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    const ELEMENT: ElementRect = ElementRect {
        origin: (0.0, 0.0),
        size: (100.0, 50.0),
    };

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-3, "expected {expected}, got {actual}");
    }

    fn sample_at(ripple: &RippleAnimation, now: Instant) -> RippleFrame {
        ripple.sample(now, ELEMENT).expect("active ripple")
    }

    fn half_diagonal() -> f32 {
        (100.0_f32.powi(2) + 50.0_f32.powi(2)).sqrt() / 2.0
    }

    #[test]
    fn starts_transparent_at_start_radius() {
        let start = Instant::now();
        let ripple = RippleAnimation::new(None, false, start);
        let frame = sample_at(&ripple, start);
        assert_close(frame.alpha, 0.0);
        assert_close(frame.radius, 100.0 * RIPPLE_START_FRACTION);
    }

    #[test]
    fn reaches_full_alpha_after_fade_in() {
        let start = Instant::now();
        let ripple = RippleAnimation::new(None, false, start);
        assert_close(sample_at(&ripple, start + RIPPLE_FADE_IN).alpha, 1.0);
    }

    #[test]
    fn radius_reaches_half_diagonal_after_radius_duration() {
        let start = Instant::now();
        let ripple = RippleAnimation::new(None, false, start);
        let frame = sample_at(&ripple, start + RIPPLE_RADIUS_DURATION);
        assert_close(frame.radius, half_diagonal());
    }

    #[test]
    fn bounded_ripple_extends_radius_by_extra_dp() {
        let start = Instant::now();
        let ripple = RippleAnimation::new(None, true, start);
        let frame = sample_at(&ripple, start + RIPPLE_RADIUS_DURATION);
        assert_close(frame.radius, half_diagonal() + BOUNDED_RIPPLE_EXTRA_DP);
    }

    #[test]
    fn origin_moves_toward_center_over_time() {
        let start = Instant::now();
        let ripple = RippleAnimation::new(Some((0.0, 0.0)), true, start);
        let frame = sample_at(&ripple, start + RIPPLE_RADIUS_DURATION);
        assert_close(frame.center.0, 50.0);
        assert_close(frame.center.1, 25.0);
    }

    #[test]
    fn release_holds_full_alpha_then_fades_out() {
        let start = Instant::now();
        let mut ripple = RippleAnimation::new(None, false, start);
        ripple.finish(start);
        let out_start = start + RIPPLE_FADE_IN;
        let held = sample_at(&ripple, out_start - Duration::from_millis(1));
        assert_close(held.alpha, 1.0);
        let fading = sample_at(&ripple, out_start + RIPPLE_FADE_OUT / 2);
        assert_close(fading.alpha, 0.5);
    }

    #[test]
    fn finished_ripple_is_removed_after_fade_out() {
        let start = Instant::now();
        let mut ripple = RippleAnimation::new(None, false, start);
        assert!(!ripple.is_finished(start + RIPPLE_FADE_OUT * 10));
        ripple.finish(start);
        let gone = start + RIPPLE_FADE_IN + RIPPLE_FADE_OUT;
        assert!(ripple.is_finished(gone));
        assert!(ripple.sample(gone, ELEMENT).is_none());
    }
}
