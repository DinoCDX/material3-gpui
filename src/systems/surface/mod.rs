pub mod interactive;
pub mod tonal;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, BoxShadow, ElementId, Entity, FocusHandle, Hsla, IntoElement, MouseButton,
    ParentElement, RenderOnce, Window, div, point, px,
};

use crate::systems::blend::{composite_over, with_alpha};
use crate::systems::interaction::controller::InteractionController;
use crate::systems::interaction::state_layer::overlay_color;
use crate::systems::interaction::{InteractionFlags, is_activation_key};
use crate::systems::ripple::layer::ripple_layer;
use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::CornerRadius;
use crate::tokens::state::{DISABLED_CONTAINER_OPACITY, DISABLED_CONTENT_OPACITY, StateLayer};

use tonal::surface_color_at_elevation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SurfaceStyle {
    pub shape: CornerRadius,
    pub tonal_elevation: ElevationLevel,
    pub shadow_elevation: ElevationLevel,
    pub outlined: bool,
}

impl Default for SurfaceStyle {
    fn default() -> Self {
        Self {
            shape: CornerRadius::None,
            tonal_elevation: ElevationLevel::Level0,
            shadow_elevation: ElevationLevel::Level0,
            outlined: false,
        }
    }
}

#[derive(IntoElement)]
pub struct Surface {
    id: ElementId,
    style: SurfaceStyle,
    controller: Option<Entity<InteractionController>>,
    children: Vec<AnyElement>,
}

impl Surface {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: SurfaceStyle::default(),
            controller: None,
            children: Vec::new(),
        }
    }

    pub fn style(mut self, style: SurfaceStyle) -> Self {
        self.style = style;
        self
    }

    pub fn interactive(mut self, controller: Entity<InteractionController>) -> Self {
        self.controller = Some(controller);
        self
    }
}

impl ParentElement for Surface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

fn shadow_for(level: ElevationLevel, color: Hsla) -> Vec<BoxShadow> {
    level
        .shadow_layers()
        .iter()
        .map(|layer| BoxShadow {
            color: with_alpha(color, layer.alpha),
            offset: point(px(0.0), px(layer.offset_y)),
            blur_radius: px(layer.blur),
            spread_radius: px(layer.spread),
        })
        .collect()
}

fn state_overlay(color: Hsla, shape: CornerRadius) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .rounded(shape.pixels())
        .bg(color)
}

fn surface_flags(
    controller: Option<&Entity<InteractionController>>,
    window: &Window,
    cx: &App,
) -> (InteractionFlags, Option<FocusHandle>) {
    match controller {
        Some(controller) => {
            let state = controller.read(cx);
            let handle = state.focus_handle().clone();
            (state.flags(handle.is_focused(window)), Some(handle))
        }
        None => (InteractionFlags::default(), None),
    }
}

impl RenderOnce for Surface {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (flags, focus_handle) = surface_flags(self.controller.as_ref(), window, cx);
        let theme = cx.theme();
        let on_surface = theme.color(ColorRole::OnSurface);
        let base = surface_color_at_elevation(
            theme.color(ColorRole::Surface),
            theme.color(ColorRole::SurfaceTint),
            self.style.tonal_elevation.dp(),
        );
        let outline = theme.color(ColorRole::Outline);
        let shadow = theme.color(ColorRole::Shadow);

        let (container, content, shadows) = if flags.disabled {
            (
                composite_over(with_alpha(on_surface, DISABLED_CONTAINER_OPACITY), base),
                with_alpha(on_surface, DISABLED_CONTENT_OPACITY),
                Vec::new(),
            )
        } else {
            (
                base,
                on_surface,
                shadow_for(self.style.shadow_elevation, shadow),
            )
        };

        let shape = self.style.shape;
        let mut element = div()
            .id(self.id)
            .relative()
            .rounded(shape.pixels())
            .overflow_hidden()
            .bg(container)
            .text_color(content)
            .shadow(shadows);

        if self.style.outlined {
            element = element.border_1().border_color(outline);
        }

        if let Some(layer) = flags.state_layer() {
            element = element.child(state_overlay(overlay_color(on_surface, layer), shape));
        }

        if let (Some(controller), Some(focus_handle)) = (self.controller, focus_handle) {
            let hover = controller.clone();
            let press = controller.clone();
            let release = controller.clone();
            let key_down = controller.clone();
            let key_up = controller.clone();
            element = element
                .track_focus(&focus_handle)
                .on_hover(move |hovered, _window, cx| {
                    hover.update(cx, |state, cx| state.set_hovered(*hovered, cx));
                })
                .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                    let position = event.position;
                    press.update(cx, |state, cx| state.press(Some(position), cx));
                })
                .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                    release.update(cx, |state, cx| state.release(cx));
                })
                .on_key_down(move |event, _window, cx| {
                    if is_activation_key(&event.keystroke.key) {
                        key_down.update(cx, |state, cx| state.press(None, cx));
                    }
                })
                .on_key_up(move |event, _window, cx| {
                    if is_activation_key(&event.keystroke.key) {
                        key_up.update(cx, |state, cx| state.release(cx));
                    }
                });
            let ripple_color = with_alpha(on_surface, StateLayer::Press.opacity());
            element = element.child(ripple_layer(controller, ripple_color, shape));
        }

        element.children(self.children)
    }
}
