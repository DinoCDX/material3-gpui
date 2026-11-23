use crate::systems::geometry::Point2;
use super::polygon::{normalize, resample};

pub const MORPH_SAMPLES: usize = 128;

pub struct Morph {
    start: Vec<Point2>,
    end: Vec<Point2>,
}

fn distance_squared(a: Point2, b: Point2) -> f32 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}

fn best_alignment(start: &[Point2], end: &[Point2]) -> Vec<Point2> {
    let count = end.len();
    let reversed: Vec<Point2> = end.iter().rev().copied().collect();
    let mut best_cost = f32::MAX;
    let mut best = end.to_vec();
    for candidate in [end, reversed.as_slice()] {
        for shift in 0..count {
            let cost: f32 = (0..count)
                .map(|index| distance_squared(start[index], candidate[(index + shift) % count]))
                .sum();
            if cost < best_cost {
                best_cost = cost;
                best = (0..count)
                    .map(|index| candidate[(index + shift) % count])
                    .collect();
            }
        }
    }
    best
}

impl Morph {
    pub fn new(from: &[Point2], to: &[Point2]) -> Self {
        let start = resample(&normalize(from), MORPH_SAMPLES);
        let end = best_alignment(&start, &resample(&normalize(to), MORPH_SAMPLES));
        Self { start, end }
    }

    pub fn points(&self, progress: f32) -> Vec<Point2> {
        let t = progress.clamp(0.0, 1.0);
        self.start
            .iter()
            .zip(&self.end)
            .map(|(a, b)| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Point2> {
        vec![(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
    }

    #[test]
    fn morph_starts_at_the_first_shape_and_ends_at_the_second() {
        let morph = Morph::new(&square(), &square());
        let start = morph.points(0.0);
        let end = morph.points(1.0);
        for (a, b) in start.iter().zip(&end) {
            assert!(distance_squared(*a, *b) < 1e-4);
        }
    }

    #[test]
    fn half_progress_lies_between_the_endpoints() {
        let circle: Vec<Point2> = (0..32)
            .map(|index| {
                let angle = std::f32::consts::TAU * index as f32 / 32.0;
                (2.0 * angle.cos(), 2.0 * angle.sin())
            })
            .collect();
        let morph = Morph::new(&square(), &circle);
        let mid = morph.points(0.5);
        let max_radius = mid.iter().map(|p| (p.0 * p.0 + p.1 * p.1).sqrt()).fold(0.0, f32::max);
        assert!(max_radius <= 2.0_f32.sqrt() + 1e-3);
    }
}
