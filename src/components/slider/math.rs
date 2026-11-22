#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderRange {
    pub min: f32,
    pub max: f32,
    pub steps: usize,
}

impl SliderRange {
    pub fn new(min: f32, max: f32, steps: usize) -> Self {
        let max = if max > min { max } else { min + 1.0 };
        Self { min, max, steps }
    }

    pub fn span(self) -> f32 {
        self.max - self.min
    }

    pub fn fraction(self, value: f32) -> f32 {
        ((value - self.min) / self.span()).clamp(0.0, 1.0)
    }

    pub fn value_at(self, fraction: f32) -> f32 {
        let fraction = fraction.clamp(0.0, 1.0);
        if self.steps == 0 {
            self.min + fraction * self.span()
        } else {
            let intervals = (self.steps + 1) as f32;
            let index = (fraction * intervals).round();
            self.min + index / intervals * self.span()
        }
    }

    pub fn step(self) -> f32 {
        if self.steps == 0 {
            self.span() / 100.0
        } else {
            self.span() / (self.steps + 1) as f32
        }
    }

    pub fn stop_fractions(self) -> Vec<f32> {
        let intervals = (self.steps + 1) as f32;
        (1..=self.steps).map(|index| index as f32 / intervals).collect()
    }

    pub fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_value_round_trips_through_fraction() {
        let range = SliderRange::new(0.0, 200.0, 0);
        assert!((range.value_at(range.fraction(50.0)) - 50.0).abs() < 1e-3);
    }

    #[test]
    fn stepped_values_snap_to_intervals() {
        let range = SliderRange::new(0.0, 4.0, 3);
        assert_eq!(range.value_at(0.30), 1.0);
        assert_eq!(range.value_at(0.60), 2.0);
        assert_eq!(range.value_at(0.99), 4.0);
    }

    #[test]
    fn stop_indicators_sit_between_the_steps() {
        let range = SliderRange::new(0.0, 4.0, 3);
        assert_eq!(range.stop_fractions(), vec![0.25, 0.5, 0.75]);
    }

    #[test]
    fn keyboard_step_matches_the_interval() {
        assert_eq!(SliderRange::new(0.0, 4.0, 3).step(), 1.0);
        assert_eq!(SliderRange::new(0.0, 100.0, 0).step(), 1.0);
    }

    #[test]
    fn degenerate_range_is_widened() {
        assert_eq!(SliderRange::new(5.0, 5.0, 0).span(), 1.0);
    }
}
