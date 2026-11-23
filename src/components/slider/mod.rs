pub mod math;
pub mod tokens;

use gpui::prelude::*;
use gpui::{
    App, Bounds, Context, DispatchPhase, Entity, ElementId, FocusHandle, Hsla, IntoElement,
    MouseButton, MouseMoveEvent, Pixels, Render, SharedString, Window, canvas, div, px,
};

use super::ChangeHandler;
use crate::systems::blend::with_alpha;
use crate::systems::text::{TrackedText, measure_text};
use crate::systems::theme::{ActiveTheme, MaterialTheme};
use crate::tokens::color::ColorRole;
use crate::tokens::typography::TypeStyle;
use math::SliderRange;
use tokens::{
    DISABLED_ACTIVE_OPACITY, DISABLED_HANDLE_OPACITY, DISABLED_INACTIVE_OPACITY,
    DISABLED_STOP_OPACITY, FOCUS_HANDLE_WIDTH, HANDLE_HEIGHT, HANDLE_WIDTH, LABEL_AREA_HEIGHT,
    LABEL_HEIGHT, LABEL_PADDING, LABEL_WIDTH, STOP_SIZE, TRACK_HEIGHT,
    VALUE_INDICATOR_BOTTOM_SPACE,
};

#[derive(Clone, Copy, Debug)]
struct SliderColors {
    active: Hsla,
    inactive: Hsla,
    handle: Hsla,
    stop: Hsla,
    label_background: Hsla,
    label_text: Hsla,
}

fn slider_colors(theme: &MaterialTheme, enabled: bool) -> SliderColors {
    let on_surface = theme.color(ColorRole::OnSurface);
    let label_background = theme.color(ColorRole::InverseSurface);
    let label_text = theme.color(ColorRole::InverseOnSurface);
    if enabled {
        SliderColors {
            active: theme.color(ColorRole::Primary),
            inactive: theme.color(ColorRole::SecondaryContainer),
            handle: theme.color(ColorRole::Primary),
            stop: theme.color(ColorRole::SecondaryContainer),
            label_background,
            label_text,
        }
    } else {
        SliderColors {
            active: with_alpha(on_surface, DISABLED_ACTIVE_OPACITY),
            inactive: with_alpha(on_surface, DISABLED_INACTIVE_OPACITY),
            handle: with_alpha(on_surface, DISABLED_HANDLE_OPACITY),
            stop: with_alpha(on_surface, DISABLED_STOP_OPACITY),
            label_background,
            label_text,
        }
    }
}

fn fraction_from_x(bounds: Option<Bounds<Pixels>>, x: Pixels) -> Option<f32> {
    let bounds = bounds?;
    let left = f32::from(&bounds.origin.x);
    let width = f32::from(&bounds.size.width).max(1.0);
    Some(((f32::from(&x) - left) / width).clamp(0.0, 1.0))
}

fn track_segment(from: f32, to: f32, width: f32, color: Hsla) -> impl IntoElement {
    let top = LABEL_AREA_HEIGHT + (HANDLE_HEIGHT - TRACK_HEIGHT) / 2.0;
    div()
        .absolute()
        .top(px(top))
        .left(px(from * width))
        .w(px((to - from).max(0.0) * width))
        .h(px(TRACK_HEIGHT))
        .rounded(px(TRACK_HEIGHT / 2.0))
        .bg(color)
}

fn handle_bar(fraction: f32, width: f32, handle_width: f32, color: Hsla) -> impl IntoElement {
    let left = (fraction * width - handle_width / 2.0).clamp(0.0, (width - handle_width).max(0.0));
    div()
        .absolute()
        .top(px(LABEL_AREA_HEIGHT))
        .left(px(left))
        .w(px(handle_width))
        .h(px(HANDLE_HEIGHT))
        .rounded(px(handle_width / 2.0))
        .bg(color)
}

fn stop_dot(fraction: f32, width: f32, color: Hsla) -> impl IntoElement {
    let top = LABEL_AREA_HEIGHT + HANDLE_HEIGHT / 2.0 - STOP_SIZE / 2.0;
    div()
        .absolute()
        .top(px(top))
        .left(px(fraction * width - STOP_SIZE / 2.0))
        .size(px(STOP_SIZE))
        .rounded(px(STOP_SIZE / 2.0))
        .bg(color)
}

fn value_label(
    window: &Window,
    fraction: f32,
    width: f32,
    text: String,
    colors: SliderColors,
    family: SharedString,
) -> impl IntoElement {
    let measured = measure_text(window, &text, TypeStyle::LabelLarge, family.clone());
    let label_width = (measured + 2.0 * LABEL_PADDING).max(LABEL_WIDTH);
    let left = (fraction * width - label_width / 2.0).clamp(0.0, (width - label_width).max(0.0));
    let top = LABEL_AREA_HEIGHT - VALUE_INDICATOR_BOTTOM_SPACE - LABEL_HEIGHT;
    div()
        .absolute()
        .top(px(top))
        .left(px(left))
        .w(px(label_width))
        .h(px(LABEL_HEIGHT))
        .rounded(px(LABEL_HEIGHT / 2.0))
        .flex()
        .items_center()
        .justify_center()
        .bg(colors.label_background)
        .child(TrackedText::new(
            text,
            TypeStyle::LabelLarge,
            colors.label_text,
            family,
        ))
}

fn format_value(value: f32) -> String {
    format!("{:.2}", value).trim_end_matches('0').trim_end_matches('.').to_string()
}

fn drag_listener<V: 'static, T: 'static>(
    view: Entity<V>,
    update: fn(&mut V, Pixels, &mut Context<V>) -> Option<T>,
    handler_of: fn(&V) -> Option<ChangeHandler<T>>,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            let view = view.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || event.pressed_button != Some(MouseButton::Left)
                {
                    return;
                }
                let handler = handler_of(view.read(cx));
                let changed = view.update(cx, |this, cx| update(this, event.position.x, cx));
                if let (Some(value), Some(handler)) = (changed, handler) {
                    (handler.as_ref())(value, window, cx);
                }
            });
        },
    )
    .absolute()
    .size_0()
}

pub struct Slider {
    id: ElementId,
    range: SliderRange,
    value: f32,
    width: f32,
    enabled: bool,
    hovered: bool,
    dragging: bool,
    bounds: Option<Bounds<Pixels>>,
    focus_handle: FocusHandle,
    on_change: Option<ChangeHandler<f32>>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>, min: f32, max: f32, cx: &mut Context<Self>) -> Self {
        let range = SliderRange::new(min, max, 0);
        Self {
            id: id.into(),
            range,
            value: range.min,
            width: 240.0,
            enabled: true,
            hovered: false,
            dragging: false,
            bounds: None,
            focus_handle: cx.focus_handle(),
            on_change: None,
        }
    }

    pub fn steps(mut self, steps: usize) -> Self {
        self.range.steps = steps;
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = self.range.clamp(value);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn current(&self) -> f32 {
        self.value
    }

    fn apply_fraction(&mut self, fraction: f32, cx: &mut Context<Self>) -> Option<f32> {
        let value = self.range.value_at(fraction);
        if (value - self.value).abs() > f32::EPSILON {
            self.value = value;
            cx.notify();
            Some(value)
        } else {
            None
        }
    }

    fn step_by(&mut self, direction: f32, cx: &mut Context<Self>) -> Option<f32> {
        let target = self.range.clamp(self.value + direction * self.range.step());
        let fraction = self.range.fraction(target);
        self.apply_fraction(fraction, cx)
    }

    fn drag_to(&mut self, x: Pixels, cx: &mut Context<Self>) -> Option<f32> {
        let fraction = fraction_from_x(self.bounds, x)?;
        self.apply_fraction(fraction, cx)
    }
}

fn slider_drag(this: &mut Slider, x: Pixels, cx: &mut Context<Slider>) -> Option<f32> {
    if !this.dragging {
        return None;
    }
    this.drag_to(x, cx)
}

fn slider_handler(this: &Slider) -> Option<ChangeHandler<f32>> {
    this.on_change.clone()
}

impl Render for Slider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let theme = cx.theme();
        let colors = slider_colors(theme, self.enabled);
        let family = theme.typography().font_family(TypeStyle::LabelLarge.font()).clone();
        let width = self.width;
        let fraction = self.range.fraction(self.value);
        let handle_width = if focused { FOCUS_HANDLE_WIDTH } else { HANDLE_WIDTH };
        let show_label = self.dragging || focused;
        let label = format_value(self.value);

        let view = cx.entity();
        let capture = view.clone();
        let press = view.clone();
        let release = view.clone();
        let release_outside = view.clone();
        let keys = view.clone();
        let hover = view.clone();
        let dragging = self.dragging;

        let mut root = div()
            .id(self.id.clone())
            .relative()
            .w(px(width))
            .h(px(LABEL_AREA_HEIGHT + HANDLE_HEIGHT))
            .track_focus(&self.focus_handle)
            .on_hover(move |hovered, _window, cx| {
                hover.update(cx, |this, cx| {
                    this.hovered = *hovered;
                    cx.notify();
                });
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                let handler = press.read(cx).on_change.clone();
                let changed = press.update(cx, |this, cx| {
                    this.dragging = true;
                    this.drag_to(event.position.x, cx)
                });
                if let (Some(value), Some(handler)) = (changed, handler) {
                    (handler.as_ref())(value, window, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                release.update(cx, |this, cx| {
                    this.dragging = false;
                    cx.notify();
                });
            })
            .on_mouse_up_out(MouseButton::Left, move |_event, _window, cx| {
                release_outside.update(cx, |this, cx| {
                    this.dragging = false;
                    cx.notify();
                });
            })
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let change = match key {
                    "left" | "down" => keys.update(cx, |this, cx| this.step_by(-1.0, cx)),
                    "right" | "up" => keys.update(cx, |this, cx| this.step_by(1.0, cx)),
                    "home" => keys.update(cx, |this, cx| this.apply_fraction(0.0, cx)),
                    "end" => keys.update(cx, |this, cx| this.apply_fraction(1.0, cx)),
                    _ => return,
                };
                let handler = keys.read(cx).on_change.clone();
                if let (Some(value), Some(handler)) = (change, handler) {
                    (handler.as_ref())(value, window, cx);
                }
            });

        root = root.child(
            canvas(
                move |bounds, _window, cx| {
                    capture.update(cx, |this, _| this.bounds = Some(bounds));
                },
                |_, _, _, _| (),
            )
            .absolute()
            .size_full(),
        );
        if dragging {
            root = root.child(drag_listener(view.clone(), slider_drag, slider_handler));
        }
        root = root
            .child(track_segment(0.0, 1.0, width, colors.inactive))
            .child(track_segment(0.0, fraction, width, colors.active));
        if self.range.steps > 0 {
            for stop in self.range.stop_fractions() {
                root = root.child(stop_dot(stop, width, colors.stop));
            }
        }
        root = root.child(handle_bar(fraction, width, handle_width, colors.handle));
        if show_label {
            root = root.child(value_label(window, fraction, width, label, colors, family));
        }
        root
    }
}

pub struct RangeSlider {
    id: ElementId,
    range: SliderRange,
    low: f32,
    high: f32,
    active: usize,
    width: f32,
    enabled: bool,
    dragging: bool,
    bounds: Option<Bounds<Pixels>>,
    focus_handle: FocusHandle,
    on_change: Option<ChangeHandler<(f32, f32)>>,
}

impl RangeSlider {
    pub fn new(id: impl Into<ElementId>, min: f32, max: f32, cx: &mut Context<Self>) -> Self {
        let range = SliderRange::new(min, max, 0);
        Self {
            id: id.into(),
            range,
            low: range.min,
            high: range.max,
            active: 0,
            width: 240.0,
            enabled: true,
            dragging: false,
            bounds: None,
            focus_handle: cx.focus_handle(),
            on_change: None,
        }
    }

    pub fn steps(mut self, steps: usize) -> Self {
        self.range.steps = steps;
        self
    }

    pub fn values(mut self, low: f32, high: f32) -> Self {
        self.low = self.range.clamp(low.min(high));
        self.high = self.range.clamp(low.max(high));
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn((f32, f32), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn current(&self) -> (f32, f32) {
        (self.low, self.high)
    }

    fn set_handle(
        &mut self,
        index: usize,
        value: f32,
        cx: &mut Context<Self>,
    ) -> Option<(f32, f32)> {
        let value = self.range.value_at(self.range.fraction(value));
        let (old_low, old_high) = (self.low, self.high);
        if index == 0 {
            self.low = value.min(self.high);
        } else {
            self.high = value.max(self.low);
        }
        let low_moved = (self.low - old_low).abs() > f32::EPSILON;
        let high_moved = (self.high - old_high).abs() > f32::EPSILON;
        if low_moved || high_moved {
            cx.notify();
            Some((self.low, self.high))
        } else {
            None
        }
    }

    fn nearest_handle(&self, fraction: f32) -> usize {
        let low = self.range.fraction(self.low);
        let high = self.range.fraction(self.high);
        if (fraction - low).abs() < (fraction - high).abs() {
            0
        } else {
            1
        }
    }

    fn drag_to(&mut self, x: Pixels, cx: &mut Context<Self>) -> Option<(f32, f32)> {
        let fraction = fraction_from_x(self.bounds, x)?;
        let index = self.active;
        let value = self.range.value_at(fraction);
        self.set_handle(index, value, cx)
    }
}

fn range_drag(
    this: &mut RangeSlider,
    x: Pixels,
    cx: &mut Context<RangeSlider>,
) -> Option<(f32, f32)> {
    if !this.dragging {
        return None;
    }
    this.drag_to(x, cx)
}

fn range_handler(this: &RangeSlider) -> Option<ChangeHandler<(f32, f32)>> {
    this.on_change.clone()
}

impl Render for RangeSlider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let theme = cx.theme();
        let colors = slider_colors(theme, self.enabled);
        let family = theme.typography().font_family(TypeStyle::LabelLarge.font()).clone();
        let width = self.width;
        let low = self.range.fraction(self.low);
        let high = self.range.fraction(self.high);
        let handle_width = if focused { FOCUS_HANDLE_WIDTH } else { HANDLE_WIDTH };
        let active_index = self.active;
        let show_label = self.dragging || focused;
        let label = format_value(if active_index == 0 { self.low } else { self.high });
        let label_fraction = if active_index == 0 { low } else { high };
        let dragging = self.dragging;

        let view = cx.entity();
        let capture = view.clone();
        let press = view.clone();
        let release = view.clone();
        let release_outside = view.clone();
        let keys = view.clone();

        let mut root = div()
            .id(self.id.clone())
            .relative()
            .w(px(width))
            .h(px(LABEL_AREA_HEIGHT + HANDLE_HEIGHT))
            .track_focus(&self.focus_handle)
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                let handler = press.read(cx).on_change.clone();
                let changed = press.update(cx, |this, cx| {
                    this.dragging = true;
                    let fraction = fraction_from_x(this.bounds, event.position.x)?;
                    this.active = this.nearest_handle(fraction);
                    this.drag_to(event.position.x, cx)
                });
                if let (Some(value), Some(handler)) = (changed, handler) {
                    (handler.as_ref())(value, window, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                release.update(cx, |this, cx| {
                    this.dragging = false;
                    cx.notify();
                });
            })
            .on_mouse_up_out(MouseButton::Left, move |_event, _window, cx| {
                release_outside.update(cx, |this, cx| {
                    this.dragging = false;
                    cx.notify();
                });
            })
            .on_key_down(move |event, window, cx| {
                let direction = match event.keystroke.key.as_str() {
                    "left" | "down" => -1.0,
                    "right" | "up" => 1.0,
                    _ => return,
                };
                let handler = keys.read(cx).on_change.clone();
                let changed = keys.update(cx, |this, cx| {
                    let index = this.active;
                    let current = if index == 0 { this.low } else { this.high };
                    let step = direction * this.range.step();
                    let target = this.range.clamp(current + step);
                    this.set_handle(index, target, cx)
                });
                if let (Some(value), Some(handler)) = (changed, handler) {
                    (handler.as_ref())(value, window, cx);
                }
            });

        root = root.child(
            canvas(
                move |bounds, _window, cx| {
                    capture.update(cx, |this, _| this.bounds = Some(bounds));
                },
                |_, _, _, _| (),
            )
            .absolute()
            .size_full(),
        );
        if dragging {
            root = root.child(drag_listener(view.clone(), range_drag, range_handler));
        }
        root = root
            .child(track_segment(0.0, 1.0, width, colors.inactive))
            .child(track_segment(low, high, width, colors.active));
        if self.range.steps > 0 {
            for stop in self.range.stop_fractions() {
                root = root.child(stop_dot(stop, width, colors.stop));
            }
        }
        root = root
            .child(handle_bar(low, width, handle_width, colors.handle))
            .child(handle_bar(high, width, handle_width, colors.handle));
        if show_label {
            root = root.child(value_label(window, label_fraction, width, label, colors, family));
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_labels_drop_trailing_zeros() {
        assert_eq!(format_value(2.0), "2");
        assert_eq!(format_value(2.5), "2.5");
        assert_eq!(format_value(2.25), "2.25");
    }
}
