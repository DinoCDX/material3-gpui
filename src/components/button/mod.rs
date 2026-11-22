pub mod style;
pub mod tokens;

use std::rc::Rc;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, Context, DefiniteLength, ElementId, Entity, IntoElement, Render,
    SharedString, Subscription, Window, div, px,
};

use super::{ElementBuilder, sync_enabled};
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::surface::{ActivateFn, Surface, SurfaceStyle};
use crate::systems::theme::ActiveTheme;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::ShapeToken;
use style::ButtonVariant;
use tokens::{ButtonShape, ButtonSize};

pub struct Button {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    enabled: bool,
    leading_icon: Option<ElementBuilder>,
    trailing_icon: Option<ElementBuilder>,
    on_click: Option<ActivateFn>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl Button {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Filled,
            size: ButtonSize::Small,
            shape: ButtonShape::Round,
            enabled: true,
            leading_icon: None,
            trailing_icon: None,
            on_click: None,
            controller,
            _observation: observation,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn leading_icon(
        mut self,
        icon: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.leading_icon = Some(Box::new(icon));
        self
    }

    pub fn trailing_icon(
        mut self,
        icon: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trailing_icon = Some(Box::new(icon));
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

impl Render for Button {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);

        let size = self.size.tokens();
        let variant = self.variant;
        let shape_kind = self.shape;
        let (pressed, disabled) = {
            let state = self.controller.read(cx);
            (state.is_pressed(), state.is_disabled())
        };
        let shape = match (shape_kind, pressed) {
            (ButtonShape::Round, _) => ShapeToken::CornerFull,
            (ButtonShape::Square, true) => size.pressed_square_shape,
            (ButtonShape::Square, false) => size.square_shape,
        };

        let theme = cx.theme();
        let colors = variant.colors(theme);
        let label_family = theme
            .typography()
            .font_family(size.label_style.font())
            .clone();
        let label_color = if disabled {
            colors.disabled_content
        } else {
            colors.content
        };

        let icon_slot = |element: AnyElement| {
            div()
                .size(px(size.icon_size))
                .flex()
                .items_center()
                .justify_center()
                .child(element)
        };
        let leading = self.leading_icon.as_ref().map(|icon| icon_slot(icon(window, cx)));
        let trailing = self.trailing_icon.as_ref().map(|icon| icon_slot(icon(window, cx)));
        let label = div()
            .font_family(label_family)
            .font_weight(size.label_style.weight())
            .text_size(px(size.label_style.size_sp()))
            .line_height(DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(
                size.label_style.line_height(),
            ))))
            .text_color(label_color)
            .child(self.label.clone());

        let row = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(size.icon_label_space))
            .pl(px(size.leading_space))
            .pr(px(size.trailing_space))
            .children(leading)
            .child(label)
            .children(trailing);

        let on_click = self.on_click.clone();
        Surface::new(self.id.clone())
            .style(SurfaceStyle {
                shape,
                tonal_elevation: ElevationLevel::Level0,
                elevation: variant.elevation(),
                outlined: variant.is_outlined(),
            })
            .colors(colors)
            .interactive(self.controller.clone())
            .on_activate(move |window, cx| {
                if let Some(handler) = &on_click {
                    (handler.as_ref())(window, cx);
                }
            })
            .child(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_tokens_are_available_for_every_size() {
        for size in [
            ButtonSize::XSmall,
            ButtonSize::Small,
            ButtonSize::Medium,
            ButtonSize::Large,
            ButtonSize::XLarge,
        ] {
            assert!(size.tokens().container_height > 0.0);
        }
    }
}
