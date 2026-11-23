pub mod state;
pub mod style;
pub mod tokens;

use std::rc::Rc;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    App, Context, ElementId, Entity, IntoElement, Render, Subscription, Window, div, px,
};

use super::{IconSource, IntoIconSource, sync_enabled};
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::surface::{ActivateFn, Surface, SurfaceStyle};
use crate::systems::surface::StateElevation;
use crate::systems::theme::ActiveTheme;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::ShapeToken;
use state::next_selected;
use style::IconButtonVariant;
use tokens::{IconButtonSize, IconButtonWidth};

pub struct IconButton {
    id: ElementId,
    icon: IconSource,
    variant: IconButtonVariant,
    size: IconButtonSize,
    width: IconButtonWidth,
    shape_round: bool,
    toggleable: bool,
    selected: bool,
    enabled: bool,
    on_click: Option<ActivateFn>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl IconButton {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl IntoIconSource,
        cx: &mut Context<Self>,
    ) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            icon: icon.into_icon_source(),
            variant: IconButtonVariant::Standard,
            size: IconButtonSize::Small,
            width: IconButtonWidth::Default,
            shape_round: true,
            toggleable: false,
            selected: false,
            enabled: true,
            on_click: None,
            controller,
            _observation: observation,
        }
    }

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }

    pub fn square(mut self) -> Self {
        self.shape_round = false;
        self
    }

    pub fn toggleable(mut self, toggleable: bool) -> Self {
        self.toggleable = toggleable;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }
}

impl Render for IconButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);

        let size = self.size.tokens();
        let variant = self.variant;
        let selected = self.selected;
        let shape = if self.shape_round {
            ShapeToken::CornerFull
        } else {
            size.square_shape
        };
        let colors = variant.colors(cx.theme(), selected);
        let icon_color = colors.content;
        let icon = self.icon.element(size.icon_size, window, cx);

        let view = cx.entity();
        let on_click = self.on_click.clone();
        let toggleable = self.toggleable;
        let elevation = StateElevation::constant(ElevationLevel::Level0);

        let slot = div()
            .w(px(size.container_width(self.width)))
            .h(px(size.container_size))
            .flex()
            .items_center()
            .justify_center()
            .text_color(icon_color)
            .child(icon);

        Surface::new(self.id.clone())
            .style(SurfaceStyle {
                shape,
                tonal_elevation: ElevationLevel::Level0,
                elevation,
                outlined: variant.is_outlined(),
            })
            .colors(colors)
            .interactive(self.controller.clone())
            .on_activate(move |window, cx| {
                if toggleable {
                    view.update(cx, |this, cx| {
                        this.selected = next_selected(this.selected);
                        cx.notify();
                    });
                }
                if let Some(handler) = &on_click {
                    (handler.as_ref())(window, cx);
                }
            })
            .child(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_target_matches_container_size() {
        assert_eq!(IconButtonSize::Small.tokens().container_size, 40.0);
    }
}
