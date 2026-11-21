use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    BorderStyle, Bounds, Corners, Edges, Entity, Hsla, IntoElement, PathBuilder, Pixels, Window,
    canvas, point, px, quad, size,
};

use super::animation::{ElementRect, RippleAnimation, RippleFrame};
use super::clip::{
    CIRCLE_SEGMENTS, CORNER_SEGMENTS, circle_polygon, clip_convex, rounded_rect_polygon,
};
use crate::systems::interaction::controller::InteractionController;
use crate::tokens::shape::CornerRadius;

pub fn ripple_layer(
    controller: Entity<InteractionController>,
    color: Hsla,
    shape: CornerRadius,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            let now = Instant::now();
            let element = element_rect(bounds);
            let corner_radius = shape.dp();
            let ripples = controller.read(cx).ripples();
            let mut animating = false;
            for ripple in ripples {
                if let Some(frame) = ripple.sample(now, element) {
                    animating = true;
                    let clip = ripple.is_bounded().then_some(corner_radius);
                    paint_ripple(window, bounds, clip, frame, color);
                }
            }
            if animating {
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

fn paint_ripple(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    clip_corner_radius: Option<f32>,
    frame: RippleFrame,
    color: Hsla,
) {
    let fill = Hsla {
        a: color.a * frame.alpha,
        ..color
    };
    match clip_corner_radius {
        None => paint_circle(window, bounds, frame, fill),
        Some(corner_radius) => paint_clipped_circle(window, bounds, corner_radius, frame, fill),
    }
}

fn paint_circle(window: &mut Window, bounds: Bounds<Pixels>, frame: RippleFrame, fill: Hsla) {
    let radius = frame.radius;
    let circle = Bounds::new(
        point(
            px(bounds.origin.x.0 + frame.center.0 - radius),
            px(bounds.origin.y.0 + frame.center.1 - radius),
        ),
        size(px(radius * 2.0), px(radius * 2.0)),
    );
    window.paint_quad(quad(
        circle,
        Corners::all(px(radius)),
        fill,
        Edges::all(px(0.0)),
        fill,
        BorderStyle::default(),
    ));
}

fn paint_clipped_circle(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    corner_radius: f32,
    frame: RippleFrame,
    fill: Hsla,
) {
    let origin = (bounds.origin.x.0, bounds.origin.y.0);
    let extent = (bounds.size.width.0, bounds.size.height.0);
    let limit = extent.0.min(extent.1) / 2.0;
    let clamped_radius = corner_radius.min(limit);
    let surface = rounded_rect_polygon(origin, extent, clamped_radius, CORNER_SEGMENTS);
    let center = (origin.0 + frame.center.0, origin.1 + frame.center.1);
    let circle = circle_polygon(center, frame.radius, CIRCLE_SEGMENTS);
    let region = clip_convex(&circle, &surface);
    if region.len() < 3 {
        return;
    }
    let first = region[0];
    let rest = &region[1..];
    let mut builder = PathBuilder::fill();
    builder.move_to(point(px(first.0), px(first.1)));
    for vertex in rest {
        builder.line_to(point(px(vertex.0), px(vertex.1)));
    }
    builder.line_to(point(px(first.0), px(first.1)));
    if let Ok(path) = builder.build() {
        window.paint_path(path, fill);
    }
}
