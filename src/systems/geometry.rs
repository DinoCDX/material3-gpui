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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_segment_polygon_has_the_stroke_height() {
        let polygon = segment_polygon((0.0, 0.0), (10.0, 0.0), 1.0);
        let height = polygon[0].1 - polygon[3].1;
        assert!((height - 2.0).abs() < 1e-5);
        assert!((polygon[1].0 - 10.0).abs() < 1e-5);
    }
}
