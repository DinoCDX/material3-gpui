use gpui::{
    AnyElement, App, AppContext, Context, ElementId, Entity, IntoElement, ParentElement, Render,
    Subscription, Window,
};

use super::{Surface, SurfaceStyle};
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;

pub struct InteractiveSurface {
    id: ElementId,
    style: SurfaceStyle,
    controller: Entity<InteractionController>,
    content: Box<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    _observation: Subscription,
}

impl InteractiveSurface {
    pub fn new(
        id: impl Into<ElementId>,
        style: SurfaceStyle,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            style,
            controller,
            content: Box::new(content),
            _observation: observation,
        }
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }

    pub fn set_style(&mut self, style: SurfaceStyle, cx: &mut Context<Self>) {
        self.style = style;
        cx.notify();
    }
}

impl Render for InteractiveSurface {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let children = (self.content)(window, cx);
        Surface::new(self.id.clone())
            .style(self.style)
            .interactive(self.controller.clone())
            .child(children)
    }
}
