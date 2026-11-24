pub mod tokens;

use std::sync::OnceLock;
use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    App, BorderStyle, Bounds, Corners, Edges, Hsla, IntoElement, Pixels, RenderOnce, Window,
    canvas, div, px, quad,
};

use crate::systems::geometry::{annulus, fill_polygon};
use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;
use tokens::{
    ARC_SEGMENTS, CIRCULAR_SIZE, CIRCULAR_THICKNESS, INDETERMINATE_BAR_FRACTION,
    INDETERMINATE_MS, LINEAR_HEIGHT, LINEAR_TRACK_GAP, LINEAR_WIDTH,
};

static EPOCH: OnceLock<Instant> = OnceLock::new();

fn cycle_fraction(period_ms: f32) -> f32 {
    let epoch = EPOCH.get_or_init(Instant::now);
    let elapsed_ms = epoch.elapsed().as_secs_f32() * 1000.0;
    (elapsed_ms / period_ms).fract()
}

fn paint_bar(window: &mut Window, origin: (f32, f32), left: f32, right: f32, color: Hsla) {
    if right <= left {
        return;
    }
    let bounds = Bounds::new(
        gpui::point(px(origin.0 + left), px(origin.1)),
        gpui::size(px(right - left), px(LINEAR_HEIGHT)),
    );
    window.paint_quad(quad(
        bounds,
        Corners::all(px(LINEAR_HEIGHT / 2.0)),
        color,
        Edges::all(px(0.0)),
        color,
        BorderStyle::default(),
    ));
}

#[derive(IntoElement)]
pub struct LinearProgress {
    value: Option<f32>,
    width: f32,
}

impl LinearProgress {
    pub fn determinate(value: f32) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            width: LINEAR_WIDTH,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            value: None,
            width: LINEAR_WIDTH,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl RenderOnce for LinearProgress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let active = theme.color(ColorRole::Primary);
        let track = theme.color(ColorRole::SurfaceContainerHighest);
        let width = self.width;
        let value = self.value;
        div().w(px(width)).h(px(LINEAR_HEIGHT)).child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let origin = (f32::from(&bounds.origin.x), f32::from(&bounds.origin.y));
                    match value {
                        Some(fraction) => {
                            let split = fraction * width;
                            let active_end = (split - LINEAR_TRACK_GAP).max(0.0);
                            let track_start = (split + LINEAR_TRACK_GAP).min(width);
                            paint_bar(window, origin, 0.0, active_end, active);
                            paint_bar(window, origin, track_start, width, track);
                        }
                        None => {
                            paint_bar(window, origin, 0.0, width, track);
                            let bar_width = width * INDETERMINATE_BAR_FRACTION;
                            let phase = cycle_fraction(INDETERMINATE_MS);
                            let left = -bar_width + phase * (width + bar_width);
                            paint_bar(
                                window,
                                origin,
                                left.max(0.0),
                                (left + bar_width).min(width),
                                active,
                            );
                            window.request_animation_frame();
                        }
                    }
                },
            )
            .absolute()
            .size_full(),
        )
    }
}

#[derive(IntoElement)]
pub struct CircularProgress {
    value: Option<f32>,
    size: f32,
}

impl CircularProgress {
    pub fn determinate(value: f32) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            size: CIRCULAR_SIZE,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            value: None,
            size: CIRCULAR_SIZE,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for CircularProgress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let active = theme.color(ColorRole::Primary);
        let track = theme.color(ColorRole::SurfaceContainerHighest);
        let size = self.size;
        let value = self.value;
        let radius = (size - CIRCULAR_THICKNESS) / 2.0;
        div().w(px(size)).h(px(size)).child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let origin = (f32::from(&bounds.origin.x), f32::from(&bounds.origin.y));
                    let centre = (origin.0 + size / 2.0, origin.1 + size / 2.0);
                    let full = annulus(
                        centre,
                        radius,
                        CIRCULAR_THICKNESS,
                        0.0,
                        std::f32::consts::TAU,
                        ARC_SEGMENTS * 2,
                    );
                    fill_polygon(window, &full, track);
                    let turn = std::f32::consts::TAU;
                    match value {
                        Some(fraction) => {
                            if fraction > 0.0 {
                                let arc = annulus(
                                    centre,
                                    radius,
                                    CIRCULAR_THICKNESS,
                                    -std::f32::consts::FRAC_PI_2,
                                    turn * fraction,
                                    ARC_SEGMENTS,
                                );
                                fill_polygon(window, &arc, active);
                            }
                        }
                        None => {
                            let phase = cycle_fraction(INDETERMINATE_MS);
                            let start = turn * phase - std::f32::consts::FRAC_PI_2;
                            let arc = annulus(
                                centre,
                                radius,
                                CIRCULAR_THICKNESS,
                                start,
                                turn / 4.0,
                                ARC_SEGMENTS,
                            );
                            fill_polygon(window, &arc, active);
                            window.request_animation_frame();
                        }
                    }
                },
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
    fn cycle_fraction_stays_in_the_unit_interval() {
        let fraction = cycle_fraction(INDETERMINATE_MS);
        assert!((0.0..1.0).contains(&fraction));
    }
}
