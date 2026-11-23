use gpui::{Hsla, Pixels, SharedString, TextStyle, px};

use crate::systems::text::text_style_for;
use crate::tokens::typography::{FontRole, TypeStyle};

#[derive(Clone, Debug, PartialEq)]
pub struct Typography {
    brand: SharedString,
    plain: SharedString,
}

impl Typography {
    pub fn new(brand: impl Into<SharedString>, plain: impl Into<SharedString>) -> Self {
        Self {
            brand: brand.into(),
            plain: plain.into(),
        }
    }

    pub fn platform_default() -> Self {
        let family = if cfg!(target_os = "windows") {
            "Segoe UI"
        } else if cfg!(target_os = "macos") {
            "Helvetica Neue"
        } else {
            "Noto Sans"
        };
        Self::new(family, family)
    }

    pub fn font_family(&self, role: FontRole) -> &SharedString {
        match role {
            FontRole::Brand => &self.brand,
            FontRole::Plain => &self.plain,
        }
    }

    pub fn text_style(&self, style: TypeStyle, color: Hsla) -> TextStyle {
        text_style_for(style, color, self.font_family(style.font()).clone())
    }

    pub fn letter_spacing(&self, style: TypeStyle) -> Pixels {
        px(style.tracking())
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self::platform_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::FontWeight;

    #[test]
    fn default_uses_the_platform_family_for_both_roles() {
        let typography = Typography::default();
        assert_eq!(typography, Typography::platform_default());
        assert_eq!(
            typography.font_family(FontRole::Brand),
            typography.font_family(FontRole::Plain)
        );
    }

    #[test]
    fn text_style_uses_mapped_family_and_metrics() {
        let typography = Typography::default();
        let style = typography.text_style(TypeStyle::TitleLarge, Hsla::default());
        assert_eq!(style.font_family, typography.font_family(FontRole::Brand).clone());
        assert_eq!(style.font_weight, FontWeight::NORMAL);
    }

    #[test]
    fn letter_spacing_is_the_reference_tracking() {
        assert_eq!(Typography::default().letter_spacing(TypeStyle::BodyLarge), px(0.5));
    }
}
