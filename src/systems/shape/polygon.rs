use crate::systems::geometry::Point2;

pub const CORNER_SAMPLES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corner {
    pub at: Point2,
    pub radius: f32,
}

impl Corner {
    pub const fn new(x: f32, y: f32, radius: f32) -> Self {
        Self {
            at: (x, y),
            radius,
        }
    }
}

fn sub(a: Point2, b: Point2) -> Point2 {
    (a.0 - b.0, a.1 - b.1)
}

fn add(a: Point2, b: Point2) -> Point2 {
    (a.0 + b.0, a.1 + b.1)
}

fn scale(a: Point2, factor: f32) -> Point2 {
    (a.0 * factor, a.1 * factor)
}

fn length(a: Point2) -> f32 {
    (a.0 * a.0 + a.1 * a.1).sqrt()
}

fn unit(a: Point2) -> Point2 {
    let len = length(a).max(f32::EPSILON);
    scale(a, 1.0 / len)
}

fn quadratic(start: Point2, control: Point2, end: Point2, t: f32) -> Point2 {
    let inverse = 1.0 - t;
    add(
        add(scale(start, inverse * inverse), scale(control, 2.0 * inverse * t)),
        scale(end, t * t),
    )
}

pub fn rounded_outline(corners: &[Corner]) -> Vec<Point2> {
    let count = corners.len();
    let mut outline = Vec::with_capacity(count * (CORNER_SAMPLES + 1));
    for index in 0..count {
        let previous = corners[(index + count - 1) % count].at;
        let current = corners[index];
        let next = corners[(index + 1) % count].at;
        if current.radius <= 0.0 {
            outline.push(current.at);
            continue;
        }
        let to_previous = sub(previous, current.at);
        let to_next = sub(next, current.at);
        let edge_previous = length(to_previous);
        let edge_next = length(to_next);
        let direction_previous = unit(to_previous);
        let direction_next = unit(to_next);
        let cosine = (direction_previous.0 * direction_next.0
            + direction_previous.1 * direction_next.1)
            .clamp(-1.0, 1.0);
        let angle = cosine.acos();
        let half_tangent = (angle / 2.0).tan().abs().max(f32::EPSILON);
        let tangent = (current.radius / half_tangent).min(edge_previous.min(edge_next) * 0.5);
        let start = add(current.at, scale(direction_previous, tangent));
        let end = add(current.at, scale(direction_next, tangent));
        for step in 0..=CORNER_SAMPLES {
            let t = step as f32 / CORNER_SAMPLES as f32;
            outline.push(quadratic(start, current.at, end, t));
        }
    }
    outline
}

pub fn normalize(points: &[Point2]) -> Vec<Point2> {
    if points.is_empty() {
        return Vec::new();
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in points {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let centre = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    let half = ((max_x - min_x).max(max_y - min_y) / 2.0).max(f32::EPSILON);
    points
        .iter()
        .map(|point| scale(sub(*point, centre), 1.0 / half))
        .collect()
}

pub fn resample(points: &[Point2], count: usize) -> Vec<Point2> {
    if points.len() < 2 || count == 0 {
        return points.to_vec();
    }
    let segments = points.len();
    let lengths: Vec<f32> = (0..segments)
        .map(|index| length(sub(points[(index + 1) % segments], points[index])))
        .collect();
    let total: f32 = lengths.iter().sum();
    if total <= f32::EPSILON {
        return vec![points[0]; count];
    }
    let mut result = Vec::with_capacity(count);
    let mut segment = 0;
    let mut segment_start = 0.0_f32;
    for sample in 0..count {
        let target = total * sample as f32 / count as f32;
        while segment + 1 < segments && segment_start + lengths[segment] < target {
            segment_start += lengths[segment];
            segment += 1;
        }
        let fraction = if lengths[segment] > 0.0 {
            ((target - segment_start) / lengths[segment]).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let from = points[segment];
        let to = points[(segment + 1) % segments];
        result.push(add(from, scale(sub(to, from), fraction)));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrounded_corners_pass_through_unchanged() {
        let outline = rounded_outline(&[
            Corner::new(0.0, 0.0, 0.0),
            Corner::new(1.0, 0.0, 0.0),
            Corner::new(0.0, 1.0, 0.0),
        ]);
        assert_eq!(outline, vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]);
    }

    #[test]
    fn rounding_adds_samples_along_each_corner() {
        let outline = rounded_outline(&[
            Corner::new(0.0, 0.0, 0.2),
            Corner::new(1.0, 0.0, 0.2),
            Corner::new(0.0, 1.0, 0.2),
        ]);
        assert_eq!(outline.len(), 3 * (CORNER_SAMPLES + 1));
    }

    #[test]
    fn normalize_fits_the_box_to_unit_half_extent() {
        let normalized = normalize(&[(0.0, 0.0), (10.0, 0.0), (10.0, 4.0), (0.0, 4.0)]);
        let max_x = normalized.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        let min_x = normalized.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        assert!((max_x - 1.0).abs() < 1e-5);
        assert!((min_x + 1.0).abs() < 1e-5);
    }

    #[test]
    fn resample_returns_the_requested_count_on_the_outline() {
        let square = vec![(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
        let samples = resample(&square, 64);
        assert_eq!(samples.len(), 64);
        assert!(samples.iter().all(|p| p.0.abs() <= 1.0 + 1e-5 && p.1.abs() <= 1.0 + 1e-5));
    }
}
