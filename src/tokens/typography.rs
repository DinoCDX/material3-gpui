use gpui::FontWeight;

pub const SANS_SERIF_FONT_FAMILY: &str = "sans-serif";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontRole {
    Brand,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeStyle {
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
    BodyLarge,
    BodyMedium,
    BodySmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
}

impl TypeStyle {
    pub const fn font(self) -> FontRole {
        match self {
            Self::DisplayLarge
            | Self::DisplayMedium
            | Self::DisplaySmall
            | Self::HeadlineLarge
            | Self::HeadlineMedium
            | Self::HeadlineSmall
            | Self::TitleLarge => FontRole::Brand,
            Self::TitleMedium
            | Self::TitleSmall
            | Self::BodyLarge
            | Self::BodyMedium
            | Self::BodySmall
            | Self::LabelLarge
            | Self::LabelMedium
            | Self::LabelSmall => FontRole::Plain,
        }
    }

    pub fn weight(self) -> FontWeight {
        match self {
            Self::TitleMedium
            | Self::TitleSmall
            | Self::LabelLarge
            | Self::LabelMedium
            | Self::LabelSmall => FontWeight::MEDIUM,
            _ => FontWeight::NORMAL,
        }
    }

    pub const fn size_sp(self) -> f32 {
        match self {
            Self::DisplayLarge => 57.0,
            Self::DisplayMedium => 45.0,
            Self::DisplaySmall => 36.0,
            Self::HeadlineLarge => 32.0,
            Self::HeadlineMedium => 28.0,
            Self::HeadlineSmall => 24.0,
            Self::TitleLarge => 22.0,
            Self::TitleMedium => 16.0,
            Self::TitleSmall => 14.0,
            Self::BodyLarge => 16.0,
            Self::BodyMedium => 14.0,
            Self::BodySmall => 12.0,
            Self::LabelLarge => 14.0,
            Self::LabelMedium => 12.0,
            Self::LabelSmall => 11.0,
        }
    }

    pub const fn line_height(self) -> f32 {
        match self {
            Self::DisplayLarge => 64.0,
            Self::DisplayMedium => 52.0,
            Self::DisplaySmall => 44.0,
            Self::HeadlineLarge => 40.0,
            Self::HeadlineMedium => 36.0,
            Self::HeadlineSmall => 32.0,
            Self::TitleLarge => 28.0,
            Self::TitleMedium => 24.0,
            Self::TitleSmall => 20.0,
            Self::BodyLarge => 24.0,
            Self::BodyMedium => 20.0,
            Self::BodySmall => 16.0,
            Self::LabelLarge => 20.0,
            Self::LabelMedium => 16.0,
            Self::LabelSmall => 16.0,
        }
    }

    pub const fn tracking(self) -> f32 {
        match self {
            Self::DisplayLarge => -0.25,
            Self::DisplayMedium | Self::DisplaySmall => 0.0,
            Self::HeadlineLarge | Self::HeadlineMedium | Self::HeadlineSmall => 0.0,
            Self::TitleLarge => 0.0,
            Self::TitleMedium => 0.15,
            Self::TitleSmall => 0.1,
            Self::BodyLarge => 0.5,
            Self::BodyMedium => 0.25,
            Self::BodySmall => 0.4,
            Self::LabelLarge => 0.1,
            Self::LabelMedium => 0.5,
            Self::LabelSmall => 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_large_matches_m3_scale() {
        assert_eq!(TypeStyle::BodyLarge.size_sp(), 16.0);
        assert_eq!(TypeStyle::BodyLarge.line_height(), 24.0);
        assert_eq!(TypeStyle::BodyLarge.tracking(), 0.5);
        assert_eq!(TypeStyle::BodyLarge.font(), FontRole::Plain);
    }

    #[test]
    fn display_styles_use_brand_font() {
        assert_eq!(TypeStyle::DisplayLarge.font(), FontRole::Brand);
        assert_eq!(TypeStyle::DisplayLarge.size_sp(), 57.0);
    }
}
