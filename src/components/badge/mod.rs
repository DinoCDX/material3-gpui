pub mod tokens;

use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, px};

use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;
use crate::tokens::shape::FULL_CORNER_DP;
use tokens::{COUNT_LABEL_STYLE, COUNT_SIZE, DOT_SIZE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BadgeContent {
    Dot,
    Count(SharedString),
}

#[derive(IntoElement)]
pub struct Badge {
    content: BadgeContent,
}

impl Badge {
    pub fn dot() -> Self {
        Self {
            content: BadgeContent::Dot,
        }
    }

    pub fn count(text: impl Into<SharedString>) -> Self {
        Self {
            content: BadgeContent::Count(text.into()),
        }
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let container = theme.color(ColorRole::Error);
        match self.content {
            BadgeContent::Dot => div()
                .size(px(DOT_SIZE))
                .rounded(px(FULL_CORNER_DP))
                .bg(container),
            BadgeContent::Count(text) => {
                let style = COUNT_LABEL_STYLE;
                let family = theme.typography().font_family(style.font()).clone();
                div()
                    .min_w(px(COUNT_SIZE))
                    .h(px(COUNT_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(FULL_CORNER_DP))
                    .bg(container)
                    .text_color(theme.color(ColorRole::OnError))
                    .font_family(family)
                    .font_weight(style.weight())
                    .text_size(px(style.size_sp()))
                    .line_height(px(style.line_height()))
                    .child(text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_and_count_are_distinct_content() {
        assert_eq!(Badge::dot().content, BadgeContent::Dot);
        assert_eq!(
            Badge::count("3").content,
            BadgeContent::Count(SharedString::from("3"))
        );
    }

    #[test]
    fn badge_sizes_match_the_reference() {
        assert_eq!(DOT_SIZE, 6.0);
        assert_eq!(COUNT_SIZE, 16.0);
        assert_eq!(COUNT_LABEL_STYLE, TypeStyle::LabelSmall);
    }
}
