pub mod style;
pub mod tokens;

use std::rc::Rc;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, AbsoluteLength, Context, DefiniteLength, ElementId, Entity, IntoElement,
    Render, SharedString, Subscription, Window, div, px,
};

use super::{ElementBuilder, sync_enabled};
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::surface::{ActivateFn, Surface, SurfaceStyle};
use crate::systems::theme::ActiveTheme;
use crate::tokens::elevation::ElevationLevel;
use style::{FabColor, elevation};
use tokens::{
    EXTENDED_HEIGHT, EXTENDED_ICON_LABEL_SPACE, EXTENDED_ICON_SIZE, EXTENDED_LEADING_SPACE,
    EXTENDED_SHAPE, EXTENDED_TRAILING_SPACE, FabSize,
};

pub struct Fab {
    id: ElementId,
    icon: ElementBuilder,
    size: FabSize,
    color: FabColor,
    extended_label: Option<SharedString>,
    enabled: bool,
    on_click: Option<ActivateFn>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl Fab {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            icon: Box::new(icon),
            size: FabSize::Standard,
            color: FabColor::Primary,
            extended_label: None,
            enabled: true,
            on_click: None,
            controller,
            _observation: observation,
        }
    }

    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: FabColor) -> Self {
        self.color = color;
        self
    }

    pub fn extended(mut self, label: impl Into<SharedString>) -> Self {
        self.extended_label = Some(label.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }
}

impl Render for Fab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);

        let size = self.size.tokens();
        let colors = self.color.colors(cx.theme());
        let content = colors.content;
        let icon = (self.icon)(window, cx);
        let on_click = self.on_click.clone();

        match self.extended_label.clone() {
            None => {
                let slot = div()
                    .size(px(size.icon_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon);
                let mut surface = Surface::new(self.id.clone())
                    .style(SurfaceStyle {
                        shape: size.shape,
                        tonal_elevation: ElevationLevel::Level0,
                        elevation: elevation(),
                        outlined: false,
                    })
                    .colors(colors)
                    .interactive(self.controller.clone());
                surface = surface.on_activate(move |window, cx| {
                    if let Some(handler) = &on_click {
                        (handler.as_ref())(window, cx);
                    }
                });
                surface.child(
                    div()
                        .size(px(size.container_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(content)
                        .child(slot),
                )
            }
            Some(label) => {
                let style = crate::tokens::typography::TypeStyle::LabelLarge;
                let family = cx.theme().typography().font_family(style.font()).clone();
                let label_element = div()
                    .font_family(family)
                    .font_weight(style.weight())
                    .text_size(px(style.size_sp()))
                    .line_height(DefiniteLength::Absolute(AbsoluteLength::Pixels(px(
                        style.line_height(),
                    ))))
                    .text_color(content)
                    .child(label);
                let icon_slot = div()
                    .size(px(EXTENDED_ICON_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon);
                let row = div()
                    .h(px(EXTENDED_HEIGHT))
                    .flex()
                    .items_center()
                    .gap(px(EXTENDED_ICON_LABEL_SPACE))
                    .pl(px(EXTENDED_LEADING_SPACE))
                    .pr(px(EXTENDED_TRAILING_SPACE))
                    .text_color(content)
                    .child(icon_slot)
                    .child(label_element);
                let mut surface = Surface::new(self.id.clone())
                    .style(SurfaceStyle {
                        shape: EXTENDED_SHAPE,
                        tonal_elevation: ElevationLevel::Level0,
                        elevation: elevation(),
                        outlined: false,
                    })
                    .colors(colors)
                    .interactive(self.controller.clone());
                surface = surface.on_activate(move |window, cx| {
                    if let Some(handler) = &on_click {
                        (handler.as_ref())(window, cx);
                    }
                });
                surface.child(row)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_fab_uses_the_large_corner_shape() {
        assert_eq!(EXTENDED_SHAPE, ShapeToken::CornerLarge);
    }
}
