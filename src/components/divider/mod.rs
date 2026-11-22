pub mod tokens;

use gpui::prelude::*;
use gpui::{App, Hsla, IntoElement, RenderOnce, Window, div, px};

use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;
use tokens::THICKNESS;

#[derive(IntoElement)]
pub struct HorizontalDivider {
    thickness: f32,
    color: Option<Hsla>,
}

impl HorizontalDivider {
    pub fn new() -> Self {
        Self {
            thickness: THICKNESS,
            color: None,
        }
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for HorizontalDivider {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for HorizontalDivider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self
            .color
            .unwrap_or_else(|| cx.theme().color(ColorRole::OutlineVariant));
        div().w_full().h(px(self.thickness)).bg(color)
    }
}

#[derive(IntoElement)]
pub struct VerticalDivider {
    thickness: f32,
    color: Option<Hsla>,
}

impl VerticalDivider {
    pub fn new() -> Self {
        Self {
            thickness: THICKNESS,
            color: None,
        }
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for VerticalDivider {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for VerticalDivider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self
            .color
            .unwrap_or_else(|| cx.theme().color(ColorRole::OutlineVariant));
        div().h_full().w(px(self.thickness)).bg(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_thickness_matches_the_reference() {
        assert_eq!(HorizontalDivider::new().thickness, 1.0);
        assert_eq!(VerticalDivider::new().thickness, 1.0);
    }
}
