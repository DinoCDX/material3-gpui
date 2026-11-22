use std::time::Instant;

use gpui::{Context, FocusHandle, Pixels, Point};

use super::InteractionEvent;
use super::animation::{Spring, Tween};
use super::config::{FocusIndication, RippleConfig};
use super::state_layer::{incoming_duration, layer_alpha, outgoing_duration};
use crate::systems::ripple::animation::RippleAnimation;
use crate::tokens::motion::{FOCUS_RING_FOCUSING_SPEC, FOCUS_RING_UNFOCUSING_SPEC};
use crate::tokens::state::StateLayer;

pub struct InteractionController {
    focus_handle: FocusHandle,
    config: RippleConfig,
    disabled: bool,
    pressed: bool,
    stack: Vec<StateLayer>,
    ripples: Vec<RippleAnimation>,
    layer_kind: Option<StateLayer>,
    layer_tween: Tween,
    focus_spring: Spring,
    last_tick: Instant,
}

impl InteractionController {
    pub fn new(config: RippleConfig, cx: &mut Context<Self>) -> Self {
        let now = Instant::now();
        Self {
            focus_handle: cx.focus_handle(),
            config,
            disabled: false,
            pressed: false,
            stack: Vec::new(),
            ripples: Vec::new(),
            layer_kind: None,
            layer_tween: Tween::settled(0.0, now),
            focus_spring: Spring::new(0.0, FOCUS_RING_FOCUSING_SPEC),
            last_tick: now,
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub fn config(&self) -> &RippleConfig {
        &self.config
    }

    pub fn apply_config(&mut self, config: RippleConfig) {
        self.config = config;
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_focused(&self) -> bool {
        self.stack.contains(&StateLayer::Focus)
    }

    pub fn is_hovered(&self) -> bool {
        self.stack.contains(&StateLayer::Hover)
    }

    pub fn ripples(&self) -> &[RippleAnimation] {
        &self.ripples
    }

    pub fn current_layer(&self) -> Option<StateLayer> {
        self.layer_kind
    }

    pub fn state_layer_alpha(&self, now: Instant) -> f32 {
        self.layer_tween.value(now)
    }

    pub fn focus_progress(&self) -> f32 {
        self.focus_spring.value().clamp(0.0, 1.0)
    }

    pub fn needs_frame(&self, now: Instant) -> bool {
        self.ripples.iter().any(|ripple| !ripple.is_finished(now))
            || !self.layer_tween.is_finished(now)
            || !self.focus_spring.is_settled()
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        let now = Instant::now();
        self.disabled = disabled;
        if disabled {
            self.pressed = false;
            self.stack.retain(|layer| *layer == StateLayer::Focus);
            self.finish_ripples(now);
            self.refresh_layer(now);
        }
        cx.notify();
    }

    pub fn handle(&mut self, event: InteractionEvent, cx: &mut Context<Self>) {
        self.handle_at(event, Instant::now());
        cx.notify();
    }

    pub fn sync_focus(&mut self, focused: bool, cx: &mut Context<Self>) {
        if focused == self.is_focused() {
            return;
        }
        let event = if focused {
            InteractionEvent::FocusGained
        } else {
            InteractionEvent::FocusLost
        };
        self.handle(event, cx);
    }

    pub fn tick(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last_tick);
        self.focus_spring.advance(elapsed);
        self.last_tick = now;
        self.ripples.retain(|ripple| !ripple.is_finished(now));
    }

    pub(crate) fn handle_at(&mut self, event: InteractionEvent, now: Instant) {
        match event {
            InteractionEvent::HoverEnter => self.stack.push(StateLayer::Hover),
            InteractionEvent::HoverExit => {
                self.remove_last(StateLayer::Hover);
                if self.pressed {
                    self.end_press(now);
                }
            }
            InteractionEvent::FocusGained => {
                self.stack.push(StateLayer::Focus);
                self.retarget_focus(true, now);
            }
            InteractionEvent::FocusLost => {
                self.remove_last(StateLayer::Focus);
                self.retarget_focus(false, now);
            }
            InteractionEvent::PressStart(position) => self.start_press(position, now),
            InteractionEvent::PressRelease | InteractionEvent::PressCancel => {
                self.end_press(now);
            }
            InteractionEvent::DragStart => self.stack.push(StateLayer::Drag),
            InteractionEvent::DragStop | InteractionEvent::DragCancel => {
                self.remove_last(StateLayer::Drag);
            }
        }
        self.refresh_layer(now);
    }

    fn start_press(&mut self, position: Option<Point<Pixels>>, now: Instant) {
        if self.disabled || self.pressed {
            return;
        }
        self.finish_ripples(now);
        if self.config.press_alpha.is_some() {
            let origin = if self.config.bounded {
                position.map(|point| (point.x.0, point.y.0))
            } else {
                None
            };
            self.ripples.push(RippleAnimation::new(
                origin,
                self.config.bounded,
                self.config.radius,
                now,
            ));
        }
        self.pressed = true;
    }

    fn end_press(&mut self, now: Instant) {
        self.pressed = false;
        self.finish_ripples(now);
    }

    fn finish_ripples(&mut self, now: Instant) {
        for ripple in &mut self.ripples {
            ripple.finish(now);
        }
    }

    fn remove_last(&mut self, layer: StateLayer) {
        if let Some(index) = self.stack.iter().rposition(|entry| *entry == layer) {
            self.stack.remove(index);
        }
    }

    fn retarget_focus(&mut self, focused: bool, now: Instant) {
        self.last_tick = now;
        match self.config.focus {
            FocusIndication::InsetRing { .. } => {
                let spec = if focused {
                    FOCUS_RING_FOCUSING_SPEC
                } else {
                    FOCUS_RING_UNFOCUSING_SPEC
                };
                self.focus_spring
                    .set_target(if focused { 1.0 } else { 0.0 }, spec);
            }
            FocusIndication::Opacity(_) | FocusIndication::None => {
                self.focus_spring = Spring::new(0.0, FOCUS_RING_UNFOCUSING_SPEC);
            }
        }
    }

    fn refresh_layer(&mut self, now: Instant) {
        let next = self
            .stack
            .iter()
            .rev()
            .copied()
            .find(|layer| layer_alpha(&self.config, *layer).is_some());
        if next == self.layer_kind {
            return;
        }
        let current = self.layer_tween.value(now);
        let (target, duration) = match next {
            Some(kind) => (
                layer_alpha(&self.config, kind).unwrap_or(0.0),
                incoming_duration(kind),
            ),
            None => (
                0.0,
                outgoing_duration(self.layer_kind.unwrap_or(StateLayer::Hover)),
            ),
        };
        self.layer_tween = Tween::new(current, target, now, duration);
        self.layer_kind = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TestAppContext, point, px};
    use std::time::Duration;

    fn controller(
        cx: &mut TestAppContext,
        config: RippleConfig,
    ) -> gpui::Entity<InteractionController> {
        cx.update(|cx| cx.new(|cx| InteractionController::new(config, cx)))
    }

    fn with_state<R>(
        cx: &mut TestAppContext,
        entity: &gpui::Entity<InteractionController>,
        f: impl FnOnce(&mut InteractionController) -> R,
    ) -> R {
        cx.update(|cx| entity.update(cx, |state, _| f(state)))
    }

    #[gpui::test]
    fn most_recent_state_layer_wins(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            state.handle_at(InteractionEvent::HoverEnter, Instant::now());
            state.handle_at(InteractionEvent::FocusGained, Instant::now());
            assert_eq!(state.current_layer(), Some(StateLayer::Focus));
            state.handle_at(InteractionEvent::FocusLost, Instant::now());
            assert_eq!(state.current_layer(), Some(StateLayer::Hover));
        });
    }

    #[gpui::test]
    fn drag_is_the_most_recent_layer_while_dragging(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            state.handle_at(InteractionEvent::HoverEnter, Instant::now());
            state.handle_at(InteractionEvent::DragStart, Instant::now());
            assert_eq!(state.current_layer(), Some(StateLayer::Drag));
            state.handle_at(InteractionEvent::DragStop, Instant::now());
            assert_eq!(state.current_layer(), Some(StateLayer::Hover));
        });
    }

    #[gpui::test]
    fn press_does_not_change_the_state_layer(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            let now = Instant::now();
            state.handle_at(InteractionEvent::HoverEnter, now);
            state.handle_at(InteractionEvent::PressStart(None), now);
            assert!(state.is_pressed());
            assert_eq!(state.current_layer(), Some(StateLayer::Hover));
        });
    }

    #[gpui::test]
    fn pointer_press_creates_an_origin_ripple_when_bounded(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            let position = point(px(4.0), px(8.0));
            state.handle_at(InteractionEvent::PressStart(Some(position)), Instant::now());
            assert_eq!(state.ripples().len(), 1);
        });
    }

    #[gpui::test]
    fn leaving_while_pressed_cancels_the_press(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            let now = Instant::now();
            state.handle_at(InteractionEvent::HoverEnter, now);
            state.handle_at(InteractionEvent::PressStart(None), now);
            state.handle_at(InteractionEvent::HoverExit, now);
            assert!(!state.is_pressed());
            assert_eq!(state.current_layer(), None);
        });
    }

    #[gpui::test]
    fn release_finishes_the_ripple_so_it_can_fade(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            let start = Instant::now();
            state.handle_at(InteractionEvent::PressStart(None), start);
            state.handle_at(InteractionEvent::PressRelease, start);
            let later = start + Duration::from_secs(1);
            assert!(!state.ripples().iter().any(|ripple| !ripple.is_finished(later)));
        });
    }

    #[gpui::test]
    fn disabled_controller_ignores_presses(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            state.disabled = true;
            state.handle_at(InteractionEvent::PressStart(None), Instant::now());
            assert!(!state.is_pressed());
            assert!(state.ripples().is_empty());
        });
    }

    #[gpui::test]
    fn inset_ring_focus_animates_toward_one(cx: &mut TestAppContext) {
        let config = RippleConfig {
            focus: FocusIndication::InsetRing {
                geometry: super::super::config::INSET_FOCUS_RING,
                outer_color: gpui::black(),
                inner_color: gpui::black(),
                corners: [0.0; 4],
            },
            ..RippleConfig::new(gpui::black(), true)
        };
        let entity = controller(cx, config);
        with_state(cx, &entity, |state| {
            let start = Instant::now();
            state.handle_at(InteractionEvent::FocusGained, start);
            for step in 1..=60 {
                state.tick(start + Duration::from_millis(16 * step));
            }
            assert!(state.focus_progress() > 0.99);
            assert!(!state.needs_frame(start + Duration::from_secs(2)));
        });
    }

    #[gpui::test]
    fn opacity_focus_never_drives_the_ring(cx: &mut TestAppContext) {
        let entity = controller(cx, RippleConfig::new(gpui::black(), true));
        with_state(cx, &entity, |state| {
            let start = Instant::now();
            state.handle_at(InteractionEvent::FocusGained, start);
            state.tick(start + Duration::from_millis(200));
            assert_eq!(state.focus_progress(), 0.0);
            assert_eq!(state.current_layer(), Some(StateLayer::Focus));
        });
    }
}
