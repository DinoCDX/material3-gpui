use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    BorderStyle, Bounds, Corners, Edges, Entity, Hsla, IntoElement, PathBuilder, Pixels, Window,
    canvas, point, px, quad, size, transparent_black,
};

use super::animation::{ElementRect, end_radius};
use super::clip::{
    CIRCLE_SEGMENTS, CORNER_SEGMENTS, circle_polygon, clip_convex, rounded_rect_polygon,
};
use crate::systems::interaction::config::{FocusIndication, InsetRingGeometry};
use crate::systems::interaction::controller::InteractionController;

struct ClipShape {
    origin: (f32, f32),
    size: (f32, f32),
    corners: [f32; 4],
}

pub fn indication_layer(
    controller: Entity<InteractionController>,
    corners: [f32; 4],
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            let now = Instant::now();
            controller.update(cx, |state, _| state.tick(now));
            let state = controller.read(cx);
            let config = *state.config();
            let element = element_rect(bounds);
            let clip = ClipShape {
                origin: element.origin,
                size: element.size,
                corners,
            };
            let alpha = state.state_layer_alpha(now);
            if alpha > 0.0 {
                let radius = config
                    .radius
                    .unwrap_or_else(|| end_radius(element.size.0, element.size.1, config.bounded));
                let center = (
                    element.origin.0 + element.size.0 / 2.0,
                    element.origin.1 + element.size.1 / 2.0,
                );
                let fill = Hsla {
                    a: config.color.a * alpha,
                    ..config.color
                };
                fill_clipped_circle(window, &clip, center, radius, fill);
            }
            if let Some(press_alpha) = config.press_alpha {
                for ripple in state.ripples() {
                    let Some(frame) = ripple.sample(now, element) else {
                        continue;
                    };
                    let fill = Hsla {
                        a: config.color.a * press_alpha * frame.alpha,
                        ..config.color
                    };
                    let center = (
                        element.origin.0 + frame.center.0,
                        element.origin.1 + frame.center.1,
                    );
                    fill_clipped_circle(window, &clip, center, frame.radius, fill);
                }
            }
            if let FocusIndication::InsetRing {
                geometry,
                outer_color,
                inner_color,
                corners: ring_corners,
            } = config.focus
            {
                let progress = state.focus_progress();
                if progress > 0.0 {
                    paint_inset_ring(
                        window,
                        bounds,
                        ring_corners,
                        geometry,
                        (outer_color, inner_color),
                        progress,
                    );
                }
            }
            if state.needs_frame(now) {
                window.request_animation_frame();
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

fn element_rect(bounds: Bounds<Pixels>) -> ElementRect {
    ElementRect {
        origin: (bounds.origin.x.0, bounds.origin.y.0),
        size: (bounds.size.width.0, bounds.size.height.0),
    }
}

fn fill_clipped_circle(
    window: &mut Window,
    clip: &ClipShape,
    center: (f32, f32),
    radius: f32,
    fill: Hsla,
) {
    if radius <= 0.0 || fill.a <= 0.0 {
        return;
    }
    let surface = rounded_rect_polygon(clip.origin, clip.size, clip.corners, CORNER_SEGMENTS);
    let circle = circle_polygon(center, radius, CIRCLE_SEGMENTS);
    let region = clip_convex(&circle, &surface);
    if region.len() < 3 {
        return;
    }
    let first = region[0];
    let mut builder = PathBuilder::fill();
    builder.move_to(point(px(first.0), px(first.1)));
    for vertex in &region[1..] {
        builder.line_to(point(px(vertex.0), px(vertex.1)));
    }
    builder.line_to(point(px(first.0), px(first.1)));
    if let Ok(path) = builder.build() {
        window.paint_path(path, fill);
    }
}

fn paint_inset_ring(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    corners: [f32; 4],
    geometry: InsetRingGeometry,
    colors: (Hsla, Hsla),
    progress: f32,
) {
    let (outer_color, inner_color) = colors;
    paint_ring(
        window,
        bounds,
        corners,
        geometry.inner_inset * progress,
        geometry.inner_width * progress,
        inner_color,
    );
    paint_ring(
        window,
        bounds,
        corners,
        geometry.outer_inset * progress,
        geometry.outer_width * progress,
        outer_color,
    );
}

fn paint_ring(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    corners: [f32; 4],
    inset: f32,
    width: f32,
    color: Hsla,
) {
    if width <= 0.0 {
        return;
    }
    let ring_bounds = Bounds::new(
        point(bounds.origin.x + px(inset), bounds.origin.y + px(inset)),
        size(
            bounds.size.width - px(2.0 * inset),
            bounds.size.height - px(2.0 * inset),
        ),
    );
    let radii = corners.map(|radius| (radius - inset).max(0.0));
    window.paint_quad(quad(
        ring_bounds,
        Corners {
            top_left: px(radii[0]),
            top_right: px(radii[1]),
            bottom_right: px(radii[2]),
            bottom_left: px(radii[3]),
        },
        transparent_black(),
        Edges::all(px(width)),
        color,
        BorderStyle::default(),
    ));
}
