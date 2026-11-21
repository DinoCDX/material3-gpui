use std::time::Instant;

use gpui::{Context, FocusHandle, Pixels, Point};

use super::InteractionFlags;
use crate::systems::ripple::animation::RippleAnimation;

pub struct InteractionController {
    focus_handle: FocusHandle,
    bounded: bool,
    hovered: bool,
    pressed: bool,
    dragged: bool,
    disabled: bool,
    ripples: Vec<RippleAnimation>,
}

impl InteractionController {
    pub fn new(bounded: bool, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            bounded,
            hovered: false,
            pressed: false,
            dragged: false,
            disabled: false,
            ripples: Vec::new(),
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub fn flags(&self, focused: bool) -> InteractionFlags {
        InteractionFlags {
            hovered: self.hovered,
            focused,
            pressed: self.pressed,
            dragged: self.dragged,
            disabled: self.disabled,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn ripples(&self) -> &[RippleAnimation] {
        &self.ripples
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        if disabled {
            self.hovered = false;
            self.dragged = false;
            self.end_press(Instant::now());
        }
        cx.notify();
    }

    pub fn set_hovered(&mut self, hovered: bool, cx: &mut Context<Self>) {
        self.hovered = hovered;
        if !hovered {
            self.end_press(Instant::now());
        }
        cx.notify();
    }

    pub fn set_dragged(&mut self, dragged: bool, cx: &mut Context<Self>) {
        self.dragged = dragged && !self.disabled;
        cx.notify();
    }

    pub fn press(&mut self, position: Option<Point<Pixels>>, cx: &mut Context<Self>) {
        if self.disabled || self.pressed {
            return;
        }
        let now = Instant::now();
        self.finish_ripples(now);
        let origin = if self.bounded {
            position.map(|point| (point.x.0, point.y.0))
        } else {
            None
        };
        self.ripples.push(RippleAnimation::new(origin, self.bounded, now));
        self.pressed = true;
        cx.notify();
    }

    pub fn release(&mut self, cx: &mut Context<Self>) {
        if !self.pressed {
            return;
        }
        self.end_press(Instant::now());
        cx.notify();
    }

    fn end_press(&mut self, now: Instant) {
        self.pressed = false;
        self.finish_ripples(now);
    }

    fn finish_ripples(&mut self, now: Instant) {
        for ripple in &mut self.ripples {
            ripple.finish(now);
        }
        self.ripples.retain(|ripple| !ripple.is_finished(now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, point, px};

    #[gpui::test]
    fn pointer_press_and_release_track_state_and_ripples(cx: &mut TestAppContext) {
        let controller = cx.update(|cx| cx.new(|cx| InteractionController::new(true, cx)));
        let position = point(px(4.0), px(8.0));
        cx.update(|cx| controller.update(cx, |c, cx| c.press(Some(position), cx)));
        cx.update(|cx| {
            let state = controller.read(cx);
            assert!(state.is_pressed());
            assert_eq!(state.ripples().len(), 1);
        });
        cx.update(|cx| controller.update(cx, |c, cx| c.release(cx)));
        cx.update(|cx| assert!(!controller.read(cx).is_pressed()));
    }

    #[gpui::test]
    fn keyboard_press_creates_ripple_without_origin(cx: &mut TestAppContext) {
        let controller = cx.update(|cx| cx.new(|cx| InteractionController::new(true, cx)));
        cx.update(|cx| controller.update(cx, |c, cx| c.press(None, cx)));
        cx.update(|cx| assert_eq!(controller.read(cx).ripples().len(), 1));
    }

    #[gpui::test]
    fn leaving_while_pressed_cancels_the_press(cx: &mut TestAppContext) {
        let controller = cx.update(|cx| cx.new(|cx| InteractionController::new(true, cx)));
        cx.update(|cx| {
            controller.update(cx, |c, cx| {
                c.set_hovered(true, cx);
                c.press(None, cx);
                c.set_hovered(false, cx);
            })
        });
        cx.update(|cx| assert!(!controller.read(cx).is_pressed()));
    }

    #[gpui::test]
    fn disabled_controller_ignores_presses(cx: &mut TestAppContext) {
        let controller = cx.update(|cx| cx.new(|cx| InteractionController::new(false, cx)));
        cx.update(|cx| {
            controller.update(cx, |c, cx| {
                c.set_disabled(true, cx);
                c.press(None, cx);
            })
        });
        cx.update(|cx| {
            let state = controller.read(cx);
            assert!(!state.is_pressed());
            assert!(state.ripples().is_empty());
        });
    }

    #[gpui::test]
    fn flags_reflect_hover_and_focus_inputs(cx: &mut TestAppContext) {
        let controller = cx.update(|cx| cx.new(|cx| InteractionController::new(false, cx)));
        cx.update(|cx| controller.update(cx, |c, cx| c.set_hovered(true, cx)));
        cx.update(|cx| {
            let flags = controller.read(cx).flags(true);
            assert!(flags.hovered);
            assert!(flags.focused);
            assert!(!flags.pressed);
        });
    }
}
