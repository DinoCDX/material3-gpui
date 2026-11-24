pub mod editing;
pub mod tokens;

use std::ops::Range;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    App, Bounds, Context, ElementId, ElementInputHandler, EntityInputHandler, FocusHandle, Hsla,
    IntoElement, MouseButton, Pixels, Point, Render, SharedString, UTF16Selection, Window, canvas,
    div, point, px, size,
};

use super::ChangeHandler;
use super::icon::{Icon, IconKind};
use crate::systems::blend::with_alpha;
use crate::systems::ripple::easing::fast_out_slow_in;
use crate::systems::text::{TrackedText, measure_text};
use crate::systems::theme::{ActiveTheme, MaterialTheme};
use crate::tokens::color::ColorRole;
use crate::tokens::typography::TypeStyle;
use editing::{Editing, byte_to_utf16, nearest_boundary, utf16_to_byte};
use tokens::{
    CARET_BLINK_MS, CARET_HEIGHT, CARET_WIDTH, CONTAINER_CORNER, CONTAINER_HEIGHT, DEFAULT_WIDTH,
    DISABLED_OPACITY, ICON_GAP, ICON_SIZE, INDICATOR_FOCUS, INDICATOR_IDLE, LABEL_FLOAT_MS,
    LABEL_FLOATING_TOP, LABEL_RESTING_TOP, OUTLINE_FOCUS, OUTLINE_IDLE, PADDING, SELECTION_OPACITY,
    SUPPORTING_GAP, TEXT_FLOATING_TOP, TEXT_RESTING_TOP,
};

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

pub struct TextField {
    id: ElementId,
    editing: Editing,
    label: SharedString,
    supporting: Option<SharedString>,
    error: Option<SharedString>,
    leading: Option<IconKind>,
    trailing: Option<IconKind>,
    filled: bool,
    enabled: bool,
    width: f32,
    focus_handle: FocusHandle,
    bounds: Option<Bounds<Pixels>>,
    text_origin: f32,
    caret_visible: bool,
    was_focused: bool,
    was_floating: bool,
    float_changed: Instant,
    on_change: Option<ChangeHandler<String>>,
}

impl TextField {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(CARET_BLINK_MS))
                    .await;
                let alive = this.update(cx, |this, cx| {
                    if this.was_focused {
                        this.caret_visible = !this.caret_visible;
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            id: id.into(),
            editing: Editing::new(""),
            label: label.into(),
            supporting: None,
            error: None,
            leading: None,
            trailing: None,
            filled: true,
            enabled: true,
            width: DEFAULT_WIDTH,
            focus_handle: cx.focus_handle(),
            bounds: None,
            text_origin: PADDING,
            caret_visible: true,
            was_focused: false,
            was_floating: false,
            float_changed: Instant::now(),
            on_change: None,
        }
    }

    pub fn value(mut self, text: impl Into<String>) -> Self {
        let max = self.editing.max_chars();
        self.editing = Editing::new(text);
        if let Some(max) = max {
            self.editing = self.editing.with_max_chars(max);
        }
        self
    }

    pub fn max_chars(mut self, max: usize) -> Self {
        let text = self.editing.text().to_string();
        self.editing = Editing::new(text).with_max_chars(max);
        self
    }

    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    pub fn error(mut self, text: impl Into<SharedString>) -> Self {
        self.error = Some(text.into());
        self
    }

    pub fn leading_icon(mut self, kind: IconKind) -> Self {
        self.leading = Some(kind);
        self
    }

    pub fn trailing_icon(mut self, kind: IconKind) -> Self {
        self.trailing = Some(kind);
        self
    }

    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn current(&self) -> &str {
        self.editing.text()
    }

    fn after_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.caret_visible = true;
        if let Some(handler) = self.on_change.clone() {
            (handler.as_ref())(self.editing.text().to_string(), window, cx);
        }
        cx.notify();
    }

    fn replace_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let changed = match range {
            Some(range) => self.editing.replace_utf16(range, text),
            None => self.editing.insert(text),
        };
        if changed {
            self.after_edit(window, cx);
        }
    }

    fn family(cx: &App) -> SharedString {
        cx.theme()
            .typography()
            .font_family(TypeStyle::BodyLarge.font())
            .clone()
    }

    fn prefix_width(window: &Window, text: &str, byte: usize, family: &SharedString) -> f32 {
        measure_text(window, &text[..byte], TypeStyle::BodyLarge, family.clone())
    }

    fn index_for_x(&self, window: &Window, family: &SharedString, x: f32) -> usize {
        let text = self.editing.text();
        let mut points: Vec<(usize, f32)> = vec![(0, 0.0)];
        for (byte, character) in text.char_indices() {
            let end = byte + character.len_utf8();
            points.push((end, Self::prefix_width(window, text, end, family)));
        }
        nearest_boundary(&points, x)
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.editing.text();
        let start = utf16_to_byte(text, range_utf16.start);
        let end = utf16_to_byte(text, range_utf16.end);
        *adjusted_range = Some(byte_to_utf16(text, start)..byte_to_utf16(text, end));
        Some(text[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let text = self.editing.text();
        let selection = self.editing.selection();
        Some(UTF16Selection {
            range: byte_to_utf16(text, selection.start)..byte_to_utf16(text, selection.end),
            reversed: self.editing.caret() == selection.start,
        })
    }

    fn marked_text_range(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_range(range, text, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_range(range, new_text, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let text = self.editing.text();
        let byte = utf16_to_byte(text, range_utf16.start);
        let family = Self::family(cx);
        let x = self.text_origin + Self::prefix_width(window, text, byte, &family);
        Some(Bounds::new(
            point(element_bounds.origin.x + px(x), element_bounds.origin.y),
            size(px(1.0), element_bounds.size.height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.bounds?;
        let x = f32::from(&position.x) - f32::from(&bounds.origin.x) - self.text_origin;
        let family = Self::family(cx);
        let byte = self.index_for_x(window, &family, x);
        Some(byte_to_utf16(self.editing.text(), byte))
    }
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        self.was_focused = focused;
        let floating = focused || !self.editing.text().is_empty();
        if floating != self.was_floating {
            self.was_floating = floating;
            self.float_changed = Instant::now();
        }
        let elapsed_ms = self.float_changed.elapsed().as_secs_f32() * 1000.0;
        let progress = (elapsed_ms / LABEL_FLOAT_MS).clamp(0.0, 1.0);
        let eased = fast_out_slow_in()(progress);
        let amount = if floating { eased } else { 1.0 - eased };
        if progress < 1.0 {
            window.request_animation_frame();
        }

        let has_error = self.error.is_some();
        let enabled = self.enabled;
        let theme = cx.theme();
        let family = theme.typography().font_family(TypeStyle::BodyLarge.font()).clone();
        let on_surface = theme.color(ColorRole::OnSurface);
        let variant = theme.color(ColorRole::OnSurfaceVariant);
        let primary = theme.color(ColorRole::Primary);
        let error = theme.color(ColorRole::Error);
        let (label_color, indicator, indicator_height, outline_width, caret_color, input_color): (
            Hsla,
            Hsla,
            f32,
            f32,
            Hsla,
            Hsla,
        ) = if !enabled {
            let muted = with_alpha(on_surface, DISABLED_OPACITY);
            (muted, muted, INDICATOR_IDLE, OUTLINE_IDLE, muted, muted)
        } else if has_error {
            let indicator_height = if focused { INDICATOR_FOCUS } else { INDICATOR_IDLE };
            (error, error, indicator_height, OUTLINE_FOCUS, error, on_surface)
        } else if focused {
            (primary, primary, INDICATOR_FOCUS, OUTLINE_FOCUS, primary, on_surface)
        } else {
            (variant, variant, INDICATOR_IDLE, OUTLINE_IDLE, primary, on_surface)
        };
        let label_style = if amount > 0.5 { TypeStyle::BodySmall } else { TypeStyle::BodyLarge };
        let label_top = lerp(LABEL_RESTING_TOP, LABEL_FLOATING_TOP, amount);
        let text_top = lerp(TEXT_RESTING_TOP, TEXT_FLOATING_TOP, amount);
        let text_left = PADDING
            + if self.leading.is_some() {
                ICON_SIZE + ICON_GAP
            } else {
                0.0
            };
        self.text_origin = text_left;

        let text = self.editing.text().to_string();
        let selection = self.editing.selection();
        let selection_start_x = Self::prefix_width(window, &text, selection.start, &family);
        let selection_end_x = Self::prefix_width(window, &text, selection.end, &family);
        let caret_x = Self::prefix_width(window, &text, self.editing.caret(), &family);
        let line_height = TypeStyle::BodyLarge.line_height();

        let view = cx.entity();
        let capture = view.clone();
        let paint_view = view.clone();
        let key_view = view.clone();
        let press_view = view.clone();
        let focus = self.focus_handle.clone();

        let mut container = div()
            .id(self.id.clone())
            .relative()
            .w(px(self.width))
            .h(px(CONTAINER_HEIGHT))
            .track_focus(&self.focus_handle)
            .on_key_down(move |event, window, cx| {
                let shift = event.keystroke.modifiers.shift;
                let control = event.keystroke.modifiers.control;
                let key = event.keystroke.key.clone();
                key_view.update(cx, |this, cx| {
                    let before = this.editing.text().to_string();
                    match key.as_str() {
                        "backspace" => {
                            this.editing.backspace();
                        }
                        "delete" => {
                            this.editing.delete();
                        }
                        "left" => this.editing.move_left(shift),
                        "right" => this.editing.move_right(shift),
                        "home" => this.editing.home(shift),
                        "end" => this.editing.end(shift),
                        "a" if control => this.editing.select_all(),
                        _ => {}
                    }
                    if before != this.editing.text() {
                        this.after_edit(window, cx);
                    } else {
                        cx.notify();
                    }
                });
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                press_view.update(cx, |this, cx| {
                    let Some(bounds) = this.bounds else {
                        return;
                    };
                    let pointer = f32::from(&event.position.x);
                    let origin = f32::from(&bounds.origin.x);
                    let x = pointer - origin - this.text_origin;
                    let family = Self::family(cx);
                    let byte = this.index_for_x(window, &family, x);
                    this.editing.place_caret(byte, event.modifiers.shift);
                    this.caret_visible = true;
                    cx.notify();
                });
            });

        if self.filled {
            container = container
                .bg(theme.color(ColorRole::SurfaceContainerHighest))
                .rounded_tl(px(CONTAINER_CORNER))
                .rounded_tr(px(CONTAINER_CORNER));
        }

        container = container.child(
            canvas(
                move |bounds, _window, cx| {
                    capture.update(cx, |this, _| this.bounds = Some(bounds));
                },
                move |bounds, _, window, cx| {
                    window.handle_input(
                        &focus,
                        ElementInputHandler::new(bounds, paint_view.clone()),
                        cx,
                    );
                },
            )
            .absolute()
            .size_full(),
        );

        if focused && selection.start != selection.end {
            container = container.child(
                div()
                    .absolute()
                    .top(px(text_top))
                    .left(px(text_left + selection_start_x))
                    .w(px(selection_end_x - selection_start_x))
                    .h(px(line_height))
                    .bg(with_alpha(primary, SELECTION_OPACITY)),
            );
        }

        container = container.child(
            div()
                .absolute()
                .top(px(label_top))
                .left(px(text_left))
                .child(TrackedText::new(
                    self.label.clone(),
                    label_style,
                    label_color,
                    family.clone(),
                )),
        );

        container = container.child(
            div()
                .absolute()
                .top(px(text_top))
                .left(px(text_left))
                .child(TrackedText::new(text, TypeStyle::BodyLarge, input_color, family.clone())),
        );

        if focused && enabled && self.caret_visible {
            container = container.child(
                div()
                    .absolute()
                    .top(px(text_top + (line_height - CARET_HEIGHT) / 2.0))
                    .left(px(text_left + caret_x))
                    .w(px(CARET_WIDTH))
                    .h(px(CARET_HEIGHT))
                    .bg(caret_color),
            );
        }

        if let Some(kind) = self.leading {
            container = container.child(
                div()
                    .absolute()
                    .top(px((CONTAINER_HEIGHT - ICON_SIZE) / 2.0))
                    .left(px(PADDING))
                    .child(Icon::new(kind).size(ICON_SIZE).color(variant)),
            );
        }
        if let Some(kind) = self.trailing {
            container = container.child(
                div()
                    .absolute()
                    .top(px((CONTAINER_HEIGHT - ICON_SIZE) / 2.0))
                    .right(px(PADDING))
                    .child(Icon::new(kind).size(ICON_SIZE).color(variant)),
            );
        }

        if self.filled {
            container = container.child(
                div()
                    .absolute()
                    .bottom(px(0.0))
                    .left(px(0.0))
                    .w(px(self.width))
                    .h(px(indicator_height))
                    .bg(indicator),
            );
        } else {
            container = container.child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .size_full()
                    .rounded(px(CONTAINER_CORNER))
                    .border(px(outline_width))
                    .border_color(indicator),
            );
        }

        let message = if has_error {
            self.error.clone()
        } else {
            self.supporting.clone()
        };
        let message_color = if has_error { error } else { variant };
        let counter = self
            .editing
            .max_chars()
            .map(|max| format!("{}/{}", self.editing.char_count(), max));

        let mut footer = div()
            .flex()
            .justify_between()
            .w(px(self.width))
            .px(px(PADDING))
            .pt(px(SUPPORTING_GAP))
            .text_color(message_color);
        if let Some(message) = message {
            footer = footer.child(TrackedText::new(
                message,
                TypeStyle::BodySmall,
                message_color,
                family.clone(),
            ));
        }
        if let Some(counter) = counter {
            footer = footer.child(TrackedText::new(counter, TypeStyle::BodySmall, variant, family));
        }

        div().flex().flex_col().child(container).child(footer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_label_moves_between_resting_and_floating_positions() {
        assert_eq!(lerp(LABEL_RESTING_TOP, LABEL_FLOATING_TOP, 0.0), LABEL_RESTING_TOP);
        assert_eq!(lerp(LABEL_RESTING_TOP, LABEL_FLOATING_TOP, 1.0), LABEL_FLOATING_TOP);
    }
}
