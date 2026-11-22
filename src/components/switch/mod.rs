pub mod tokens;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    App, Context, ElementId, Entity, Hsla, IntoElement, Render, Subscription, Window, div, px,
    transparent_black,
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
use tokens::{
    DISABLED_OPACITY, DISABLED_TRACK_OPACITY, SELECTED_HANDLE_SIZE, SELECTED_ICON_SIZE,
    STATE_LAYER_SIZE, TRACK_HEIGHT, TRACK_WIDTH, UNSELECTED_HANDLE_SIZE,
};

#[derive(Clone, Copy, Debug)]
struct SwitchColors {
    track: Hsla,
    track_outline: Hsla,
    handle: Hsla,
    icon: Hsla,
}

fn switch_colors(
    theme: &MaterialTheme,
    selected: bool,
    disabled: bool,
    emphasized: bool,
) -> SwitchColors {
    let on_surface = theme.color(ColorRole::OnSurface);
    let container_highest = theme.color(ColorRole::SurfaceContainerHighest);
    let transparent = transparent_black();
    match (selected, disabled) {
        (true, false) => SwitchColors {
            track: theme.color(ColorRole::Primary),
            track_outline: transparent,
            handle: theme.color(ColorRole::OnPrimary),
            icon: theme.color(ColorRole::OnPrimaryContainer),
        },
        (true, true) => SwitchColors {
            track: with_alpha(on_surface, DISABLED_TRACK_OPACITY),
            track_outline: transparent,
            handle: theme.color(ColorRole::Surface),
            icon: with_alpha(on_surface, DISABLED_OPACITY),
        },
        (false, false) => SwitchColors {
            track: container_highest,
            track_outline: theme.color(ColorRole::Outline),
            handle: if emphasized {
                theme.color(ColorRole::OnSurfaceVariant)
            } else {
                theme.color(ColorRole::Outline)
            },
            icon: container_highest,
        },
        (false, true) => SwitchColors {
            track: with_alpha(container_highest, DISABLED_TRACK_OPACITY),
            track_outline: with_alpha(on_surface, DISABLED_TRACK_OPACITY),
            handle: with_alpha(on_surface, DISABLED_OPACITY),
            icon: container_highest,
        },
    }
}

pub struct Switch {
    id: ElementId,
    selected: bool,
    enabled: bool,
    show_icon: bool,
    on_change: Option<ChangeHandler<bool>>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>, cx: &mut Context<Self>) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            selected: false,
            enabled: true,
            show_icon: false,
            on_change: None,
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

    pub fn show_icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }
}

impl Render for Switch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);
        let (emphasized, disabled) = {
            let state = self.controller.read(cx);
            let focused = state.focus_handle().is_focused(window);
            (state.is_hovered() || focused, state.is_disabled())
        };
        let colors = switch_colors(cx.theme(), self.selected, disabled, emphasized);

        let handle_size = if self.selected {
            SELECTED_HANDLE_SIZE
        } else {
            UNSELECTED_HANDLE_SIZE
        };
        let handle_top = (TRACK_HEIGHT - handle_size) / 2.0;
        let handle_left = if self.selected {
            TRACK_WIDTH - handle_top - handle_size
        } else {
            handle_top
        };
        let handle_centre_x = handle_left + handle_size / 2.0;
        let track_top = (STATE_LAYER_SIZE - TRACK_HEIGHT) / 2.0;
        let icon_glyph = if self.selected { "✓" } else { "×" };

        let mut handle = div()
            .absolute()
            .top(px(handle_top))
            .left(px(handle_left))
            .size(px(handle_size))
            .rounded(px(handle_size / 2.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.handle);
        if self.show_icon {
            handle = handle.child(
                div()
                    .text_size(px(SELECTED_ICON_SIZE))
                    .text_color(colors.icon)
                    .child(icon_glyph),
            );
        }

        let mut track = div()
            .absolute()
            .top(px(track_top))
            .left(px(0.0))
            .w(px(TRACK_WIDTH))
            .h(px(TRACK_HEIGHT))
            .rounded(px(TRACK_HEIGHT / 2.0))
            .bg(colors.track)
            .child(handle);
        if !self.selected {
            track = track.border_2().border_color(colors.track_outline);
        }

        let view = cx.entity();
        let next = !self.selected;
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
                content: colors.handle,
                disabled_container: transparent_black(),
                disabled_content: colors.handle,
                outline: colors.track_outline,
            })
            .state_layer_radius(STATE_LAYER_SIZE / 2.0)
            .state_layer_center(handle_centre_x, STATE_LAYER_SIZE / 2.0)
            .interactive(self.controller.clone())
            .on_activate(move |window, cx| {
                view.update(cx, |this, cx| {
                    this.selected = next;
                    cx.notify();
                });
                if let Some(handler) = &on_change {
                    (handler.as_ref())(next, window, cx);
                }
            })
            .child(
                div()
                    .relative()
                    .w(px(TRACK_WIDTH))
                    .h(px(STATE_LAYER_SIZE))
                    .child(track),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::ThemeMode;
    use material_color_utils::utils::color_utils::Argb;

    #[test]
    fn handle_geometry_matches_the_reference() {
        assert_eq!(TRACK_WIDTH, 52.0);
        assert_eq!(TRACK_HEIGHT, 32.0);
        assert_eq!(UNSELECTED_HANDLE_SIZE, 16.0);
        assert_eq!(SELECTED_HANDLE_SIZE, 24.0);
    }

    #[test]
    fn selected_track_is_primary() {
        let theme = MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light);
        let colors = switch_colors(&theme, true, false, false);
        assert_eq!(colors.track.h, theme.color(ColorRole::Primary).h);
    }

    #[test]
    fn disabled_track_uses_twelve_percent() {
        let theme = MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light);
        assert_eq!(switch_colors(&theme, true, true, false).track.a, DISABLED_TRACK_OPACITY);
    }
}
