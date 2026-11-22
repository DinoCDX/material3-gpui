use std::time::{Duration, Instant};

use crate::tokens::motion::SpringSpec;

const SPRING_STEP_SECONDS: f32 = 0.001;
const SPRING_SETTLE_THRESHOLD: f32 = 0.001;
const MAX_FRAME_SECONDS: f32 = 0.1;

#[derive(Clone, Copy, Debug)]
pub struct Tween {
    from: f32,
    to: f32,
    start: Instant,
    duration: Duration,
}

impl Tween {
    pub fn new(from: f32, to: f32, start: Instant, duration: Duration) -> Self {
        Self {
            from,
            to,
            start,
            duration,
        }
    }

    pub fn settled(value: f32, now: Instant) -> Self {
        Self::new(value, value, now, Duration::ZERO)
    }

    pub fn value(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start).as_secs_f32();
        let fraction = if self.duration.is_zero() {
            1.0
        } else {
            (elapsed / self.duration.as_secs_f32()).clamp(0.0, 1.0)
        };
        self.from + (self.to - self.from) * fraction
    }

    pub fn is_finished(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.start) >= self.duration
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spring {
    value: f32,
    velocity: f32,
    target: f32,
    spec: SpringSpec,
}

impl Spring {
    pub fn new(value: f32, spec: SpringSpec) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            spec,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn set_target(&mut self, target: f32, spec: SpringSpec) {
        self.target = target;
        self.spec = spec;
    }

    pub fn is_settled(&self) -> bool {
        (self.value - self.target).abs() < SPRING_SETTLE_THRESHOLD
            && self.velocity.abs() < SPRING_SETTLE_THRESHOLD
    }

    pub fn advance(&mut self, elapsed: Duration) {
        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
            return;
        }
        let seconds = elapsed.as_secs_f32().min(MAX_FRAME_SECONDS);
        let steps = (seconds / SPRING_STEP_SECONDS).ceil().max(1.0) as u32;
        let step = seconds / steps as f32;
        let stiffness = self.spec.stiffness;
        let damping = 2.0 * self.spec.damping_ratio * stiffness.sqrt();
        for _ in 0..steps {
            let acceleration = -stiffness * (self.value - self.target) - damping * self.velocity;
            self.velocity += acceleration * step;
            self.value += self.velocity * step;
        }
        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::motion::{STANDARD_FAST_EFFECTS, STANDARD_FAST_SPATIAL};

    #[test]
    fn tween_is_linear_between_endpoints() {
        let start = Instant::now();
        let tween = Tween::new(0.0, 1.0, start, Duration::from_millis(100));
        assert!((tween.value(start + Duration::from_millis(50)) - 0.5).abs() < 1e-4);
        assert_eq!(tween.value(start + Duration::from_millis(200)), 1.0);
        assert!(tween.is_finished(start + Duration::from_millis(100)));
    }

    #[test]
    fn zero_duration_tween_jumps_to_target() {
        let start = Instant::now();
        let tween = Tween::new(0.0, 0.8, start, Duration::ZERO);
        assert_eq!(tween.value(start), 0.8);
    }

    #[test]
    fn spring_converges_to_its_target() {
        let mut spring = Spring::new(0.0, STANDARD_FAST_SPATIAL);
        spring.set_target(1.0, STANDARD_FAST_SPATIAL);
        for _ in 0..120 {
            spring.advance(Duration::from_millis(16));
        }
        assert!(spring.is_settled());
        assert_eq!(spring.value(), 1.0);
    }

    #[test]
    fn underdamped_spring_overshoots_its_target() {
        let bouncy = SpringSpec {
            damping_ratio: 0.3,
            stiffness: 800.0,
        };
        let mut spring = Spring::new(0.0, bouncy);
        spring.set_target(1.0, bouncy);
        let mut peak: f32 = 0.0;
        for _ in 0..120 {
            spring.advance(Duration::from_millis(8));
            peak = peak.max(spring.value());
        }
        assert!(peak > 1.0);
    }

    #[test]
    fn effects_spring_settles_faster_than_spatial() {
        let settle_frames = |spec: SpringSpec| {
            let mut spring = Spring::new(0.0, spec);
            spring.set_target(1.0, spec);
            let mut frames = 0;
            while !spring.is_settled() && frames < 1000 {
                spring.advance(Duration::from_millis(16));
                frames += 1;
            }
            frames
        };
        assert!(settle_frames(STANDARD_FAST_EFFECTS) < settle_frames(STANDARD_FAST_SPATIAL));
    }
}
