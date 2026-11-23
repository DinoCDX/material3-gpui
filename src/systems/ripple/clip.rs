pub type Point2 = (f32, f32);

pub const CIRCLE_SEGMENTS: usize = 128;
pub const MIN_CIRCLE_SEGMENTS: usize = 24;
pub const CORNER_SEGMENTS: usize = 16;
const DEGENERATE_EDGE_LENGTH_SQUARED: f32 = 1e-10;

pub fn circle_segments(radius: f32) -> usize {
    ((radius * 0.5).ceil() as usize).clamp(MIN_CIRCLE_SEGMENTS, CIRCLE_SEGMENTS)
}

pub fn circle_polygon(center: Point2, radius: f32, segments: usize) -> Vec<Point2> {
    (0..segments)
        .map(|index| {
            let angle = std::f32::consts::TAU * index as f32 / segments as f32;
            (center.0 + radius * angle.cos(), center.1 + radius * angle.sin())
        })
        .collect()
}

pub fn clamp_corners(radii: [f32; 4], size: Point2) -> [f32; 4] {
    let limit = size.0.min(size.1) / 2.0;
    radii.map(|radius| radius.clamp(0.0, limit))
}

pub fn rounded_rect_polygon(
    origin: Point2,
    size: Point2,
    radii: [f32; 4],
    segments_per_corner: usize,
) -> Vec<Point2> {
    let (x, y) = origin;
    let (width, height) = size;
    let [top_left, top_right, bottom_right, bottom_left] = clamp_corners(radii, size);
    let corners = [
        ((x + top_left, y + top_left), std::f32::consts::PI, top_left),
        (
            (x + width - top_right, y + top_right),
            std::f32::consts::PI * 1.5,
            top_right,
        ),
        (
            (x + width - bottom_right, y + height - bottom_right),
            0.0,
            bottom_right,
        ),
        (
            (x + bottom_left, y + height - bottom_left),
            std::f32::consts::FRAC_PI_2,
            bottom_left,
        ),
    ];
    let mut points = Vec::with_capacity(4 * (segments_per_corner + 1));
    for (center, start, radius) in corners {
        for step in 0..=segments_per_corner {
            let fraction = step as f32 / segments_per_corner as f32;
            let angle = start + std::f32::consts::FRAC_PI_2 * fraction;
            points.push((
                center.0 + radius * angle.cos(),
                center.1 + radius * angle.sin(),
            ));
        }
    }
    points
}

pub fn signed_area(polygon: &[Point2]) -> f32 {
    let count = polygon.len();
    (0..count)
        .map(|index| {
            let (x1, y1) = polygon[index];
            let (x2, y2) = polygon[(index + 1) % count];
            x1 * y2 - x2 * y1
        })
        .sum::<f32>()
        / 2.0
}

pub fn clip_convex(subject: &[Point2], clip: &[Point2]) -> Vec<Point2> {
    let clip: Vec<Point2> = if signed_area(clip) < 0.0 {
        clip.iter().rev().copied().collect()
    } else {
        clip.to_vec()
    };
    let mut output = subject.to_vec();
    for index in 0..clip.len() {
        if output.is_empty() {
            break;
        }
        let edge_start = clip[index];
        let edge_end = clip[(index + 1) % clip.len()];
        if length_squared(edge_start, edge_end) <= DEGENERATE_EDGE_LENGTH_SQUARED {
            continue;
        }
        let input = std::mem::take(&mut output);
        let count = input.len();
        for current_index in 0..count {
            let current = input[current_index];
            let previous = input[(current_index + count - 1) % count];
            let current_inside = is_inside(edge_start, edge_end, current);
            let previous_inside = is_inside(edge_start, edge_end, previous);
            if current_inside {
                if !previous_inside {
                    output.push(intersection(previous, current, edge_start, edge_end));
                }
                output.push(current);
            } else if previous_inside {
                output.push(intersection(previous, current, edge_start, edge_end));
            }
        }
    }
    output
}

fn is_inside(edge_start: Point2, edge_end: Point2, point: Point2) -> bool {
    cross(
        (edge_end.0 - edge_start.0, edge_end.1 - edge_start.1),
        (point.0 - edge_start.0, point.1 - edge_start.1),
    ) >= 0.0
}

fn intersection(p1: Point2, p2: Point2, a: Point2, b: Point2) -> Point2 {
    let direction = (p2.0 - p1.0, p2.1 - p1.1);
    let edge = (b.0 - a.0, b.1 - a.1);
    let denominator = cross(direction, edge);
    if denominator.abs() < f32::EPSILON {
        return p2;
    }
    let t = cross((a.0 - p1.0, a.1 - p1.1), edge) / denominator;
    (p1.0 + direction.0 * t, p1.1 + direction.1 * t)
}

fn length_squared(a: Point2, b: Point2) -> f32 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    dx * dx + dy * dy
}

fn cross(a: Point2, b: Point2) -> f32 {
    a.0 * b.1 - a.1 * b.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: [f32; 4] = [0.0; 4];

    fn area(polygon: &[Point2]) -> f32 {
        signed_area(polygon).abs()
    }

    #[test]
    fn small_circles_use_fewer_segments() {
        assert_eq!(circle_segments(4.0), MIN_CIRCLE_SEGMENTS);
        assert_eq!(circle_segments(1000.0), CIRCLE_SEGMENTS);
    }

    #[test]
    fn circle_inside_rectangle_is_unchanged() {
        let circle = circle_polygon((50.0, 50.0), 20.0, CIRCLE_SEGMENTS);
        let rect = rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), SQUARE, CORNER_SEGMENTS);
        let clipped = clip_convex(&circle, &rect);
        assert!((area(&clipped) - area(&circle)).abs() < 1e-2 * area(&circle));
    }

    #[test]
    fn circle_outside_rectangle_is_removed() {
        let circle = circle_polygon((500.0, 500.0), 20.0, CIRCLE_SEGMENTS);
        let rect = rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), SQUARE, CORNER_SEGMENTS);
        assert!(clip_convex(&circle, &rect).len() < 3);
    }

    #[test]
    fn rounded_corners_remove_area_from_a_rectangle_clip() {
        let circle = circle_polygon((50.0, 50.0), 60.0, CIRCLE_SEGMENTS);
        let square = rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), SQUARE, CORNER_SEGMENTS);
        let rounded =
            rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), [50.0; 4], CORNER_SEGMENTS);
        assert!(area(&clip_convex(&circle, &rounded)) < area(&clip_convex(&circle, &square)));
    }

    #[test]
    fn full_radius_clip_is_a_circle() {
        let circle = circle_polygon((50.0, 50.0), 60.0, CIRCLE_SEGMENTS);
        let rounded =
            rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), [9999.0; 4], CORNER_SEGMENTS);
        let expected = std::f32::consts::PI * 50.0 * 50.0;
        let clipped = area(&clip_convex(&circle, &rounded));
        assert!(
            (clipped - expected).abs() < 0.02 * expected,
            "clipped area {clipped} differs from circle area {expected}"
        );
    }

    #[test]
    fn directional_corners_only_round_the_named_corners() {
        let rect = rounded_rect_polygon(
            (0.0, 0.0),
            (100.0, 100.0),
            [50.0, 0.0, 0.0, 0.0],
            CORNER_SEGMENTS,
        );
        let full = rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), SQUARE, CORNER_SEGMENTS);
        assert!(area(&rect) < area(&full));
        assert!(rect.iter().any(|point| (point.0 - 100.0).abs() < 1e-3 && point.1.abs() < 1e-3));
    }

    #[test]
    fn radii_are_clamped_to_half_the_short_side() {
        assert_eq!(clamp_corners([99.0; 4], (40.0, 60.0)), [20.0; 4]);
    }

    #[test]
    fn near_duplicate_corner_joins_do_not_empty_the_clip() {
        let circle = circle_polygon((50.0, 50.0), 60.0, CIRCLE_SEGMENTS);
        let mut rounded =
            rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), [9999.0; 4], CORNER_SEGMENTS);
        let join = rounded[CORNER_SEGMENTS];
        rounded.insert(CORNER_SEGMENTS + 1, (join.0 + 1e-6, join.1 - 1e-6));
        let expected = std::f32::consts::PI * 50.0 * 50.0;
        let clipped = area(&clip_convex(&circle, &rounded));
        assert!(
            (clipped - expected).abs() < 0.02 * expected,
            "clipped area {clipped} differs from circle area {expected}"
        );
    }

    #[test]
    fn clip_orientation_does_not_matter() {
        let circle = circle_polygon((50.0, 50.0), 20.0, CIRCLE_SEGMENTS);
        let mut rect = rounded_rect_polygon((0.0, 0.0), (100.0, 100.0), SQUARE, CORNER_SEGMENTS);
        let forward = area(&clip_convex(&circle, &rect));
        rect.reverse();
        let backward = area(&clip_convex(&circle, &rect));
        assert!((forward - backward).abs() < 1e-2 * forward);
    }
}
