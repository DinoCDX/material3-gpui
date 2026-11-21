use gpui::{AbsoluteLength, DefiniteLength, Hsla, Pixels, SharedString, TextRun, TextStyle, px};

use crate::tokens::typography::{FontRole, SANS_SERIF_FONT_FAMILY, TypeStyle};

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

    pub fn font_family(&self, role: FontRole) -> &SharedString {
        match role {
            FontRole::Brand => &self.brand,
            FontRole::Plain => &self.plain,
        }
    }

    pub fn text_style(&self, style: TypeStyle, color: Hsla) -> TextStyle {
        TextStyle {
            color,
            font_family: self.font_family(style.font()).clone(),
            font_size: px(style.size_sp()).into(),
            line_height: DefiniteLength::Absolute(AbsoluteLength::Pixels(px(style.line_height()))),
            font_weight: style.weight(),
            ..Default::default()
        }
    }

    pub fn text_run(&self, style: TypeStyle, color: Hsla, len: usize) -> TextRun {
        self.text_style(style, color).to_run(len)
    }

    pub fn letter_spacing(&self, style: TypeStyle) -> Pixels {
        px(style.tracking())
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self::new(SANS_SERIF_FONT_FAMILY, SANS_SERIF_FONT_FAMILY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::FontWeight;

    #[test]
    fn text_style_uses_mapped_family_and_metrics() {
        let typography = Typography::default();
        let style = typography.text_style(TypeStyle::TitleLarge, Hsla::default());
        assert_eq!(style.font_family.as_ref(), SANS_SERIF_FONT_FAMILY);
        assert_eq!(style.font_weight, FontWeight::NORMAL);
    }

    #[test]
    fn text_run_covers_requested_length() {
        let typography = Typography::default();
        let run = typography.text_run(TypeStyle::BodyMedium, Hsla::default(), 7);
        assert_eq!(run.len, 7);
    }
}
