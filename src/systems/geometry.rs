use gpui::{Hsla, Pixels, Point, PathBuilder, Window, point, px};

pub type Point2 = (f32, f32);
pub type Stroke = [Point2; 2];

pub fn segment_polygon(from: Point2, to: Point2, half: f32) -> [Point2; 4] {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    let (nx, ny) = (-dy / length * half, dx / length * half);
    [
        (from.0 + nx, from.1 + ny),
        (to.0 + nx, to.1 + ny),
        (to.0 - nx, to.1 - ny),
        (from.0 - nx, from.1 - ny),
    ]
}

pub fn paint_strokes(
    window: &mut Window,
    origin: Point2,
    strokes: &[Stroke],
    width: f32,
    color: Hsla,
) {
    for [from, to] in strokes {
        let polygon = segment_polygon(*from, *to, width / 2.0);
        let vertices: Vec<Point<Pixels>> = polygon
            .iter()
            .map(|vertex| point(px(origin.0 + vertex.0), px(origin.1 + vertex.1)))
            .collect();
        let mut builder = PathBuilder::fill();
        builder.move_to(vertices[0]);
        for vertex in &vertices[1..] {
            builder.line_to(*vertex);
        }
        builder.line_to(vertices[0]);
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }
}

pub fn annulus(
    centre: Point2,
    radius: f32,
    thickness: f32,
    start: f32,
    sweep: f32,
    segments: usize,
) -> Vec<Point2> {
    let outer = radius + thickness / 2.0;
    let inner = radius - thickness / 2.0;
    let steps = segments.max(1);
    let mut ring = Vec::with_capacity((steps + 1) * 2);
    for step in 0..=steps {
        let angle = start + sweep * step as f32 / steps as f32;
        ring.push((
            centre.0 + outer * angle.cos(),
            centre.1 + outer * angle.sin(),
        ));
    }
    for step in (0..=steps).rev() {
        let angle = start + sweep * step as f32 / steps as f32;
        ring.push((
            centre.0 + inner * angle.cos(),
            centre.1 + inner * angle.sin(),
        ));
    }
    ring
}

pub fn fill_polygon(window: &mut Window, points: &[Point2], color: Hsla) {
    if points.len() < 3 {
        return;
    }
    let vertices: Vec<Point<Pixels>> = points
        .iter()
        .map(|vertex| point(px(vertex.0), px(vertex.1)))
        .collect();
    let mut builder = PathBuilder::fill();
    builder.move_to(vertices[0]);
    for vertex in &vertices[1..] {
        builder.line_to(*vertex);
    }
    builder.line_to(vertices[0]);
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annulus_has_two_rings_of_samples() {
        let ring = annulus((0.0, 0.0), 10.0, 2.0, 0.0, std::f32::consts::PI, 8);
        assert_eq!(ring.len(), 18);
        let first_outer = (ring[0].0.powi(2) + ring[0].1.powi(2)).sqrt();
        assert!((first_outer - 11.0).abs() < 1e-4);
    }

    #[test]
    fn horizontal_segment_polygon_has_the_stroke_height() {
        let polygon = segment_polygon((0.0, 0.0), (10.0, 0.0), 1.0);
        let height = polygon[0].1 - polygon[3].1;
        assert!((height - 2.0).abs() < 1e-5);
        assert!((polygon[1].0 - 10.0).abs() < 1e-5);
    }
}
