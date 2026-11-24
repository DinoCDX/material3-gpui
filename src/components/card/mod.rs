pub mod tokens;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    App, Context, ElementId, Entity, IntoElement, Render, Subscription, Window, div, px,
};

use super::{ElementBuilder, sync_enabled};
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::surface::{
    ActivateFn, StateElevation, Surface, SurfaceColors, SurfaceStyle,
};
use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::ShapeToken;
use tokens::{CONTENT_GAP, PADDING};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardVariant {
    Elevated,
    Filled,
    Outlined,
}

pub struct Card {
    id: ElementId,
    variant: CardVariant,
    clickable: bool,
    enabled: bool,
    content: Option<ElementBuilder>,
    on_click: Option<ActivateFn>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl Card {
    pub fn new(id: impl Into<ElementId>, variant: CardVariant, cx: &mut Context<Self>) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            variant,
            clickable: false,
            enabled: true,
            content: None,
            on_click: None,
            controller,
            _observation: observation,
        }
    }

    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn content(
        mut self,
        builder: impl Fn(&mut Window, &mut App) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.content = Some(Box::new(builder));
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }
}

impl Render for Card {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);
        let theme = cx.theme();
        let surface = theme.color(ColorRole::Surface);
        let on_surface = theme.color(ColorRole::OnSurface);
        let outline = theme.color(ColorRole::OutlineVariant);
        let (container, disabled_container, elevation, outlined) = match self.variant {
            CardVariant::Elevated => (
                theme.color(ColorRole::SurfaceContainerLow),
                surface,
                if self.clickable {
                    StateElevation {
                        default: ElevationLevel::Level1,
                        hovered: ElevationLevel::Level2,
                        focused: ElevationLevel::Level1,
                        pressed: ElevationLevel::Level1,
                        disabled: ElevationLevel::Level0,
                    }
                } else {
                    StateElevation::constant(ElevationLevel::Level1)
                },
                false,
            ),
            CardVariant::Filled => (
                theme.color(ColorRole::SurfaceContainerHighest),
                theme.color(ColorRole::SurfaceVariant),
                StateElevation::constant(ElevationLevel::Level0),
                false,
            ),
            CardVariant::Outlined => (
                surface,
                surface,
                StateElevation::constant(ElevationLevel::Level0),
                true,
            ),
        };

        let mut card = Surface::new(self.id.clone())
            .style(SurfaceStyle {
                shape: ShapeToken::CornerMedium,
                tonal_elevation: ElevationLevel::Level0,
                elevation,
                outlined,
            })
            .colors(SurfaceColors {
                container,
                content: on_surface,
                disabled_container,
                disabled_content: on_surface,
                outline,
            });
        if self.clickable {
            card = card.interactive(self.controller.clone());
            if let Some(handler) = self.on_click.clone() {
                card = card.on_activate(move |window, cx| (handler.as_ref())(window, cx));
            }
        }
        let content = match &self.content {
            Some(builder) => builder(window, cx),
            None => div().into_any_element(),
        };
        card.child(
            div()
                .flex()
                .flex_col()
                .gap(px(CONTENT_GAP))
                .p(px(PADDING))
                .child(content),
        )
    }
}
