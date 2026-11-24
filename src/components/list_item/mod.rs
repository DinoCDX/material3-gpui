use gpui::prelude::*;
use gpui::{AnyElement, App, IntoElement, RenderOnce, SharedString, Window, div, px};

use super::ElementBuilder;
use crate::systems::text::TrackedText;
use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;
use crate::tokens::typography::TypeStyle;

const PADDING: f32 = 16.0;
const SLOT_GAP: f32 = 16.0;
const ONE_LINE: f32 = 56.0;
const TWO_LINE: f32 = 72.0;
const THREE_LINE: f32 = 88.0;

#[derive(IntoElement)]
pub struct ListItem {
    headline: SharedString,
    supporting: Option<SharedString>,
    third_line: Option<SharedString>,
    leading: Option<ElementBuilder>,
    trailing: Option<ElementBuilder>,
}

impl ListItem {
    pub fn new(headline: impl Into<SharedString>) -> Self {
        Self {
            headline: headline.into(),
            supporting: None,
            third_line: None,
            leading: None,
            trailing: None,
        }
    }

    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    pub fn third_line(mut self, text: impl Into<SharedString>) -> Self {
        self.third_line = Some(text.into());
        self
    }

    pub fn leading(
        mut self,
        builder: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.leading = Some(Box::new(builder));
        self
    }

    pub fn trailing(
        mut self,
        builder: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trailing = Some(Box::new(builder));
        self
    }

    fn height(&self) -> f32 {
        match (self.supporting.is_some(), self.third_line.is_some()) {
            (_, true) => THREE_LINE,
            (true, false) => TWO_LINE,
            (false, false) => ONE_LINE,
        }
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let height = self.height();
        let theme = cx.theme();
        let family = theme.typography().font_family(TypeStyle::BodyLarge.font()).clone();
        let on_surface = theme.color(ColorRole::OnSurface);
        let variant = theme.color(ColorRole::OnSurfaceVariant);

        let mut text = div().flex().flex_col().flex_1().justify_center();
        text = text.child(TrackedText::new(
            self.headline.clone(),
            TypeStyle::BodyLarge,
            on_surface,
            family.clone(),
        ));
        if let Some(supporting) = self.supporting.clone() {
            text = text.child(TrackedText::new(
                supporting,
                TypeStyle::BodyMedium,
                variant,
                family.clone(),
            ));
        }
        if let Some(third) = self.third_line.clone() {
            text = text.child(TrackedText::new(
                third,
                TypeStyle::BodyMedium,
                variant,
                family,
            ));
        }

        let mut row = div()
            .h(px(height))
            .flex()
            .items_center()
            .gap(px(SLOT_GAP))
            .px(px(PADDING));
        if let Some(leading) = &self.leading {
            row = row.child(leading(window, cx));
        }
        row = row.child(text);
        if let Some(trailing) = &self.trailing {
            row = row.child(trailing(window, cx));
        }
        row
    }
}
