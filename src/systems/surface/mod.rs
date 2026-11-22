pub mod interactive;
pub mod tonal;

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, BoxShadow, ElementId, Entity, Hsla, IntoElement, MouseButton, ParentElement,
    RenderOnce, Window, div, point, px,
};

use crate::systems::blend::{composite_over, with_alpha};
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::interaction::{InteractionEvent, is_activation_key};
use crate::systems::ripple::layer::indication_layer;
use crate::systems::theme::{ActiveTheme, MaterialTheme};
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::ShapeToken;
use crate::tokens::state::{DISABLED_CONTAINER_OPACITY, DISABLED_CONTENT_OPACITY};
use tonal::surface_color_at_elevation;

pub type ActivateFn = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateElevation {
    pub default: ElevationLevel,
    pub hovered: ElevationLevel,
    pub focused: ElevationLevel,
    pub pressed: ElevationLevel,
    pub disabled: ElevationLevel,
}

impl StateElevation {
    pub const fn constant(level: ElevationLevel) -> Self {
        Self {
            default: level,
            hovered: level,
            focused: level,
            pressed: level,
            disabled: level,
        }
    }

    pub fn select(
        self,
        hovered: bool,
        focused: bool,
        pressed: bool,
        disabled: bool,
    ) -> ElevationLevel {
        if disabled {
            self.disabled
        } else if pressed {
            self.pressed
        } else if focused {
            self.focused
        } else if hovered {
            self.hovered
        } else {
            self.default
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SurfaceStyle {
    pub shape: ShapeToken,
    pub tonal_elevation: ElevationLevel,
    pub elevation: StateElevation,
    pub outlined: bool,
}

impl Default for SurfaceStyle {
    fn default() -> Self {
        Self {
            shape: ShapeToken::CornerNone,
            tonal_elevation: ElevationLevel::Level0,
            elevation: StateElevation::constant(ElevationLevel::Level0),
            outlined: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceColors {
    pub container: Hsla,
    pub content: Hsla,
    pub disabled_container: Hsla,
    pub disabled_content: Hsla,
    pub outline: Hsla,
}

impl SurfaceColors {
    pub fn default_for(theme: &MaterialTheme, tonal_dp: f32) -> Self {
        let on_surface = theme.color(ColorRole::OnSurface);
        let base = surface_color_at_elevation(
            theme.color(ColorRole::Surface),
            theme.color(ColorRole::SurfaceTint),
            tonal_dp,
        );
        Self {
            container: base,
            content: on_surface,
            disabled_container: composite_over(
                with_alpha(on_surface, DISABLED_CONTAINER_OPACITY),
                base,
            ),
            disabled_content: with_alpha(on_surface, DISABLED_CONTENT_OPACITY),
            outline: theme.color(ColorRole::Outline),
        }
    }
}

#[derive(IntoElement)]
pub struct Surface {
    id: ElementId,
    style: SurfaceStyle,
    colors: Option<SurfaceColors>,
    controller: Option<Entity<InteractionController>>,
    activate: Option<ActivateFn>,
    state_radius: Option<f32>,
    state_center: Option<(f32, f32)>,
    children: Vec<AnyElement>,
}

impl Surface {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: SurfaceStyle::default(),
            colors: None,
            controller: None,
            activate: None,
            state_radius: None,
            state_center: None,
            children: Vec::new(),
        }
    }

    pub fn style(mut self, style: SurfaceStyle) -> Self {
        self.style = style;
        self
    }

    pub fn colors(mut self, colors: SurfaceColors) -> Self {
        self.colors = Some(colors);
        self
    }

    pub fn interactive(mut self, controller: Entity<InteractionController>) -> Self {
        self.controller = Some(controller);
        self
    }

    pub fn state_layer_radius(mut self, radius: f32) -> Self {
        self.state_radius = Some(radius);
        self
    }

    pub fn state_layer_center(mut self, x: f32, y: f32) -> Self {
        self.state_center = Some((x, y));
        self
    }

    pub fn on_activate(mut self, activate: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.activate = Some(Rc::new(activate));
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

impl RenderOnce for Surface {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let corners = self.style.shape.radii().as_array();
        let theme = cx.theme();
        let colors = self
            .colors
            .unwrap_or_else(|| SurfaceColors::default_for(theme, self.style.tonal_elevation.dp()));
        let shadow_color = theme.color(ColorRole::Shadow);
        let focus = theme.ripple_focus().with_corners(corners);

        let (hovered, focused_now, pressed, disabled) = match &self.controller {
            Some(controller) => {
                let state = controller.read(cx);
                (
                    state.is_hovered(),
                    state.focus_handle().is_focused(window),
                    state.is_pressed(),
                    state.is_disabled(),
                )
            }
            None => (false, false, false, false),
        };
        let level = self.style.elevation.select(hovered, focused_now, pressed, disabled);

        if let Some(controller) = &self.controller {
            let config = RippleConfig {
                focus,
                radius: self.state_radius,
                state_center: self.state_center,
                ..RippleConfig::new(colors.content, true)
            };
            controller.update(cx, |state, cx| {
                state.apply_config(config);
                state.sync_focus(focused_now, cx);
            });
        }

        let (container, content) = if disabled {
            (colors.disabled_container, colors.disabled_content)
        } else {
            (colors.container, colors.content)
        };

        let radii = self.style.shape.radii();
        let mut element = div()
            .id(self.id)
            .relative()
            .rounded_tl(px(radii.top_left))
            .rounded_tr(px(radii.top_right))
            .rounded_br(px(radii.bottom_right))
            .rounded_bl(px(radii.bottom_left))
            .overflow_hidden()
            .bg(container)
            .text_color(content)
            .shadow(shadow_for(level, shadow_color));

        if self.style.outlined {
            element = element.border_1().border_color(colors.outline);
        }

        if let Some(controller) = &self.controller {
            let focus_handle = controller.read(cx).focus_handle().clone();
            let hover = controller.clone();
            let press = controller.clone();
            let release = controller.clone();
            let key_down = controller.clone();
            let key_up = controller.clone();
            let activate_mouse = self.activate.clone();
            let activate_key = self.activate.clone();
            element = element
                .track_focus(&focus_handle)
                .on_hover(move |hovered, _window, cx| {
                    let event = if *hovered {
                        InteractionEvent::HoverEnter
                    } else {
                        InteractionEvent::HoverExit
                    };
                    hover.update(cx, |state, cx| state.handle(event, cx));
                })
                .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                    let position = event.position;
                    press.update(cx, |state, cx| {
                        state.handle(InteractionEvent::PressStart(Some(position)), cx)
                    });
                })
                .on_mouse_up(MouseButton::Left, move |_event, window, cx| {
                    let completed =
                        release.read(cx).is_pressed() && !release.read(cx).is_disabled();
                    release.update(cx, |state, cx| {
                        state.handle(InteractionEvent::PressRelease, cx)
                    });
                    if completed && let Some(activate) = &activate_mouse {
                        (activate.as_ref())(window, cx);
                    }
                })
                .on_key_down(move |event, _window, cx| {
                    if is_activation_key(&event.keystroke.key) {
                        key_down.update(cx, |state, cx| {
                            state.handle(InteractionEvent::PressStart(None), cx)
                        });
                    }
                })
                .on_key_up(move |event, window, cx| {
                    if is_activation_key(&event.keystroke.key) {
                        let completed =
                            key_up.read(cx).is_pressed() && !key_up.read(cx).is_disabled();
                        key_up.update(cx, |state, cx| {
                            state.handle(InteractionEvent::PressRelease, cx)
                        });
                        if completed && let Some(activate) = &activate_key {
                            (activate.as_ref())(window, cx);
                        }
                    }
                });
        }

        element = element.children(self.children);

        match self.controller {
            Some(controller) => element.child(indication_layer(controller, corners)),
            None => element,
        }
    }
}
