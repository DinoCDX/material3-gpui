pub mod tokens;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    App, Bounds, Context, ElementId, Entity, Hsla, IntoElement, Pixels, Render, Subscription,
    Window, canvas, div, px, transparent_black,
};

use super::{ChangeHandler, sync_enabled};
use crate::systems::geometry::{Stroke, paint_strokes};
use crate::systems::blend::with_alpha;
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::surface::{StateElevation, Surface, SurfaceColors, SurfaceStyle};
use crate::systems::theme::{ActiveTheme, MaterialTheme};
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::ShapeToken;
use tokens::{CONTAINER_SIZE, CORNER_RADIUS, DISABLED_OPACITY, STATE_LAYER_SIZE, STROKE_WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckboxState {
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckboxState {
    pub fn next(self) -> Self {
        match self {
            Self::Unchecked | Self::Indeterminate => Self::Checked,
            Self::Checked => Self::Unchecked,
        }
    }

    pub fn is_filled(self) -> bool {
        !matches!(self, Self::Unchecked)
    }
}

fn paint_mark(window: &mut Window, bounds: Bounds<Pixels>, state: CheckboxState, color: Hsla) {
    let origin = (f32::from(&bounds.origin.x), f32::from(&bounds.origin.y));
    let strokes: Vec<Stroke> = match state {
        CheckboxState::Checked => vec![
            [(3.5, 9.5), (7.5, 13.5)],
            [(7.5, 13.5), (14.5, 5.5)],
        ],
        CheckboxState::Indeterminate => vec![[(4.0, 9.0), (14.0, 9.0)]],
        CheckboxState::Unchecked => Vec::new(),
    };
    paint_strokes(window, origin, &strokes, STROKE_WIDTH, color);
}

#[derive(Clone, Copy, Debug)]
struct CheckboxColors {
    fill: Hsla,
    glyph: Hsla,
    outline: Hsla,
}

fn checkbox_colors(
    theme: &MaterialTheme,
    state: CheckboxState,
    error: bool,
    disabled: bool,
    emphasized: bool,
) -> CheckboxColors {
    let on_surface = theme.color(ColorRole::OnSurface);
    if state.is_filled() {
        let fill = if disabled {
            with_alpha(on_surface, DISABLED_OPACITY)
        } else if error {
            theme.color(ColorRole::Error)
        } else {
            theme.color(ColorRole::Primary)
        };
        let glyph = if disabled {
            theme.color(ColorRole::Surface)
        } else if error {
            theme.color(ColorRole::OnError)
        } else {
            theme.color(ColorRole::OnPrimary)
        };
        CheckboxColors { fill, glyph, outline: fill }
    } else {
        let outline = if disabled {
            with_alpha(on_surface, DISABLED_OPACITY)
        } else if error {
            theme.color(ColorRole::Error)
        } else if emphasized {
            on_surface
        } else {
            theme.color(ColorRole::OnSurfaceVariant)
        };
        CheckboxColors {
            fill: transparent_black(),
            glyph: transparent_black(),
            outline,
        }
    }
}

pub struct Checkbox {
    id: ElementId,
    state: CheckboxState,
    enabled: bool,
    error: bool,
    on_change: Option<ChangeHandler<CheckboxState>>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, cx: &mut Context<Self>) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            state: CheckboxState::Unchecked,
            enabled: true,
            error: false,
            on_change: None,
            controller,
            _observation: observation,
        }
    }

    pub fn state(mut self, state: CheckboxState) -> Self {
        self.state = state;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn current(&self) -> CheckboxState {
        self.state
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }
}

impl Render for Checkbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);
        let (emphasized, disabled) = {
            let state = self.controller.read(cx);
            let focused = state.focus_handle().is_focused(window);
            (state.is_hovered() || focused, state.is_disabled())
        };
        let colors = checkbox_colors(cx.theme(), self.state, self.error, disabled, emphasized);
        let mut box_element = div()
            .size(px(CONTAINER_SIZE))
            .rounded(px(CORNER_RADIUS))
            .relative()
            .bg(colors.fill);
        if !self.state.is_filled() {
            box_element = box_element.border_2().border_color(colors.outline);
        }
        let state = self.state;
        let mark_color = colors.glyph;
        box_element = box_element.child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| paint_mark(window, bounds, state, mark_color),
            )
            .absolute()
            .size_full(),
        );

        let view = cx.entity();
        let next = self.state.next();
        let on_change = self.on_change.clone();
        Surface::new(self.id.clone())
            .style(SurfaceStyle {
                shape: ShapeToken::CornerFull,
                tonal_elevation: ElevationLevel::Level0,
                elevation: StateElevation::constant(ElevationLevel::Level0),
                outlined: false,
            })
            .colors(SurfaceColors {
                container: transparent_black(),
                content: colors.glyph,
                disabled_container: transparent_black(),
                disabled_content: colors.glyph,
                outline: colors.outline,
            })
            .state_layer_radius(STATE_LAYER_SIZE / 2.0)
            .interactive(self.controller.clone())
            .on_activate(move |window, cx| {
                view.update(cx, |this, cx| {
                    this.state = next;
                    cx.notify();
                });
                if let Some(handler) = &on_change {
                    (handler.as_ref())(next, window, cx);
                }
            })
            .child(
                div()
                    .size(px(STATE_LAYER_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(box_element),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::ThemeMode;
    use material_color_utils::utils::color_utils::Argb;

    #[test]
    fn tri_state_cycles_to_checked_then_unchecked() {
        assert_eq!(CheckboxState::Unchecked.next(), CheckboxState::Checked);
        assert_eq!(CheckboxState::Indeterminate.next(), CheckboxState::Checked);
        assert_eq!(CheckboxState::Checked.next(), CheckboxState::Unchecked);
    }

    #[test]
    fn filled_states_are_the_checked_and_indeterminate_ones() {
        assert!(!CheckboxState::Unchecked.is_filled());
        assert!(CheckboxState::Checked.is_filled());
        assert!(CheckboxState::Indeterminate.is_filled());
    }

    #[test]
    fn checked_error_uses_the_error_role() {
        let theme = MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light);
        let colors = checkbox_colors(&theme, CheckboxState::Checked, true, false, false);
        assert_eq!(colors.fill.h, theme.color(ColorRole::Error).h);
    }

    #[test]
    fn unchecked_has_no_fill() {
        let theme = MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light);
        let colors = checkbox_colors(&theme, CheckboxState::Unchecked, false, false, false);
        assert_eq!(colors.fill.a, 0.0);
    }
}
