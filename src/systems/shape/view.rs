use std::time::Instant;

use gpui::{
    Bounds, Context, Hsla, IntoElement, PathBuilder, Pixels, Point, Render, Window, canvas, div,
    point, prelude::*, px,
};

use super::morph::Morph;
use super::presets::Preset;
use crate::systems::geometry::Point2;
use crate::systems::ripple::easing::fast_out_slow_in;
use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;

pub fn cycle_state(elapsed: f32, hold: f32, morph: f32, count: usize) -> (usize, f32) {
    if count == 0 {
        return (0, 0.0);
    }
    let period = (hold + morph).max(f32::EPSILON);
    let cycle = (elapsed / period).floor().max(0.0) as usize;
    let within = elapsed - cycle as f32 * period;
    let progress = if within < hold {
        0.0
    } else {
        ((within - hold) / morph.max(f32::EPSILON)).clamp(0.0, 1.0)
    };
    (cycle % count, fast_out_slow_in()(progress))
}

fn paint_outline(window: &mut Window, bounds: Bounds<Pixels>, points: &[Point2], color: Hsla) {
    if points.len() < 3 {
        return;
    }
    let left = f32::from(&bounds.origin.x);
    let top = f32::from(&bounds.origin.y);
    let width = f32::from(&bounds.size.width);
    let height = f32::from(&bounds.size.height);
    let mapped: Vec<Point<Pixels>> = points
        .iter()
        .map(|(x, y)| {
            point(
                px(left + (x * 0.45 + 0.5) * width),
                px(top + (y * 0.45 + 0.5) * height),
            )
        })
        .collect();
    let mut builder = PathBuilder::fill();
    builder.move_to(mapped[0]);
    for vertex in &mapped[1..] {
        builder.line_to(*vertex);
    }
    builder.line_to(mapped[0]);
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

pub struct MorphView {
    morphs: Vec<Morph>,
    hold: f32,
    morph: f32,
    started: Instant,
}

impl MorphView {
    pub fn new(presets: &[Preset], hold: f32, morph: f32) -> Self {
        let outlines: Vec<Vec<Point2>> = presets.iter().map(|preset| preset.outline()).collect();
        let count = outlines.len();
        let morphs = (0..count)
            .map(|index| Morph::new(&outlines[index], &outlines[(index + 1) % count]))
            .collect();
        Self {
            morphs,
            hold,
            morph,
            started: Instant::now(),
        }
    }
}

impl Render for MorphView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();
        let elapsed = self.started.elapsed().as_secs_f32();
        let (from, progress) = cycle_state(elapsed, self.hold, self.morph, self.morphs.len());
        let points = self
            .morphs
            .get(from)
            .map(|morph| morph.points(progress))
            .unwrap_or_default();
        let color: Hsla = cx.theme().color(ColorRole::Primary);
        div().size_full().child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| paint_outline(window, bounds, &points, color),
            )
            .absolute()
            .size_full(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_before_morphing_then_wraps_around() {
        assert_eq!(cycle_state(0.5, 1.0, 1.0, 3), (0, 0.0));
        let (from, progress) = cycle_state(2.5, 1.0, 1.0, 3);
        assert_eq!(from, 1);
        assert!(progress > 0.0);
        assert_eq!(cycle_state(6.5, 1.0, 1.0, 3).0, 0);
    }

    #[test]
    fn empty_cycle_is_safe() {
        assert_eq!(cycle_state(3.0, 1.0, 1.0, 0), (0, 0.0));
    }
}
