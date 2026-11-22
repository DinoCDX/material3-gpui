pub mod tokens;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    App, Context, ElementId, Entity, Hsla, IntoElement, Render, Subscription, Window, div, px,
};

use super::{ChangeHandler, sync_enabled};
use crate::systems::blend::with_alpha;
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::surface::{StateElevation, Surface, SurfaceColors, SurfaceStyle};
use crate::systems::theme::{ActiveTheme, MaterialTheme};
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::ShapeToken;
use tokens::{DISABLED_OPACITY, DOT_SIZE, ICON_SIZE, STATE_LAYER_SIZE};

#[derive(Clone, Copy, Debug)]
struct RadioColors {
    ring: Hsla,
    dot: Hsla,
}

fn radio_colors(
    theme: &MaterialTheme,
    selected: bool,
    disabled: bool,
    emphasized: bool,
) -> RadioColors {
    let on_surface = theme.color(ColorRole::OnSurface);
    if disabled {
        let tint = with_alpha(on_surface, DISABLED_OPACITY);
        RadioColors { ring: tint, dot: tint }
    } else if selected {
        let primary = theme.color(ColorRole::Primary);
        RadioColors { ring: primary, dot: primary }
    } else {
        let ring = if emphasized {
            on_surface
        } else {
            theme.color(ColorRole::OnSurfaceVariant)
        };
        RadioColors {
            ring,
            dot: ring,
        }
    }
}

pub struct RadioButton {
    id: ElementId,
    selected: bool,
    enabled: bool,
    on_select: Option<ChangeHandler<bool>>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl RadioButton {
    pub fn new(id: impl Into<ElementId>, cx: &mut Context<Self>) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            selected: false,
            enabled: true,
            on_select: None,
            controller,
            _observation: observation,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }
}

impl Render for RadioButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);
        let (emphasized, disabled) = {
            let state = self.controller.read(cx);
            let focused = state.focus_handle().is_focused(window);
            (state.is_hovered() || focused, state.is_disabled())
        };
        let colors = radio_colors(cx.theme(), self.selected, disabled, emphasized);

        let mut ring = div()
            .size(px(ICON_SIZE))
            .rounded(px(ICON_SIZE / 2.0))
            .flex()
            .items_center()
            .justify_center()
            .border_2()
            .border_color(colors.ring);
        if self.selected {
            ring = ring.child(
                div()
                    .size(px(DOT_SIZE))
                    .rounded(px(DOT_SIZE / 2.0))
                    .bg(colors.dot),
            );
        }

        let view = cx.entity();
        let on_select = self.on_select.clone();
        Surface::new(self.id.clone())
            .style(SurfaceStyle {
                shape: ShapeToken::CornerFull,
                tonal_elevation: ElevationLevel::Level0,
                elevation: StateElevation::constant(ElevationLevel::Level0),
                outlined: false,
            })
            .colors(SurfaceColors {
                container: gpui::transparent_black(),
                content: colors.dot,
                disabled_container: gpui::transparent_black(),
                disabled_content: colors.dot,
                outline: colors.ring,
            })
            .state_layer_radius(STATE_LAYER_SIZE / 2.0)
            .interactive(self.controller.clone())
            .on_activate(move |window, cx| {
                view.update(cx, |this, cx| {
                    this.selected = true;
                    cx.notify();
                });
                if let Some(handler) = &on_select {
                    (handler.as_ref())(true, window, cx);
                }
            })
            .child(
                div()
                    .size(px(STATE_LAYER_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(ring),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::ThemeMode;
    use material_color_utils::utils::color_utils::Argb;

    #[test]
    fn selected_ring_uses_primary() {
        let theme = MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light);
        let colors = radio_colors(&theme, true, false, false);
        assert_eq!(colors.ring.h, theme.color(ColorRole::Primary).h);
    }

    #[test]
    fn disabled_is_muted_regardless_of_selection() {
        let theme = MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light);
        assert_eq!(radio_colors(&theme, true, true, false).ring.a, DISABLED_OPACITY);
        assert_eq!(radio_colors(&theme, false, true, false).ring.a, DISABLED_OPACITY);
    }
}
