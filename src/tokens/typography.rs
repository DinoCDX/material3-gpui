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
    DisplayLargeEmphasized,
    DisplayMediumEmphasized,
    DisplaySmallEmphasized,
    HeadlineLargeEmphasized,
    HeadlineMediumEmphasized,
    HeadlineSmallEmphasized,
    TitleLargeEmphasized,
    TitleMediumEmphasized,
    TitleSmallEmphasized,
    BodyLargeEmphasized,
    BodyMediumEmphasized,
    BodySmallEmphasized,
    LabelLargeEmphasized,
    LabelMediumEmphasized,
    LabelSmallEmphasized,
}

impl TypeStyle {
    pub const ALL: [TypeStyle; 30] = [
        TypeStyle::DisplayLarge,
        TypeStyle::DisplayMedium,
        TypeStyle::DisplaySmall,
        TypeStyle::HeadlineLarge,
        TypeStyle::HeadlineMedium,
        TypeStyle::HeadlineSmall,
        TypeStyle::TitleLarge,
        TypeStyle::TitleMedium,
        TypeStyle::TitleSmall,
        TypeStyle::BodyLarge,
        TypeStyle::BodyMedium,
        TypeStyle::BodySmall,
        TypeStyle::LabelLarge,
        TypeStyle::LabelMedium,
        TypeStyle::LabelSmall,
        TypeStyle::DisplayLargeEmphasized,
        TypeStyle::DisplayMediumEmphasized,
        TypeStyle::DisplaySmallEmphasized,
        TypeStyle::HeadlineLargeEmphasized,
        TypeStyle::HeadlineMediumEmphasized,
        TypeStyle::HeadlineSmallEmphasized,
        TypeStyle::TitleLargeEmphasized,
        TypeStyle::TitleMediumEmphasized,
        TypeStyle::TitleSmallEmphasized,
        TypeStyle::BodyLargeEmphasized,
        TypeStyle::BodyMediumEmphasized,
        TypeStyle::BodySmallEmphasized,
        TypeStyle::LabelLargeEmphasized,
        TypeStyle::LabelMediumEmphasized,
        TypeStyle::LabelSmallEmphasized,
    ];

    pub const fn font(self) -> FontRole {
        match self {
            Self::DisplayLarge => FontRole::Brand,
            Self::DisplayMedium => FontRole::Brand,
            Self::DisplaySmall => FontRole::Brand,
            Self::HeadlineLarge => FontRole::Brand,
            Self::HeadlineMedium => FontRole::Brand,
            Self::HeadlineSmall => FontRole::Brand,
            Self::TitleLarge => FontRole::Brand,
            Self::TitleMedium => FontRole::Plain,
            Self::TitleSmall => FontRole::Plain,
            Self::BodyLarge => FontRole::Plain,
            Self::BodyMedium => FontRole::Plain,
            Self::BodySmall => FontRole::Plain,
            Self::LabelLarge => FontRole::Plain,
            Self::LabelMedium => FontRole::Plain,
            Self::LabelSmall => FontRole::Plain,
            Self::DisplayLargeEmphasized => FontRole::Brand,
            Self::DisplayMediumEmphasized => FontRole::Brand,
            Self::DisplaySmallEmphasized => FontRole::Brand,
            Self::HeadlineLargeEmphasized => FontRole::Brand,
            Self::HeadlineMediumEmphasized => FontRole::Brand,
            Self::HeadlineSmallEmphasized => FontRole::Brand,
            Self::TitleLargeEmphasized => FontRole::Brand,
            Self::TitleMediumEmphasized => FontRole::Plain,
            Self::TitleSmallEmphasized => FontRole::Plain,
            Self::BodyLargeEmphasized => FontRole::Plain,
            Self::BodyMediumEmphasized => FontRole::Plain,
            Self::BodySmallEmphasized => FontRole::Plain,
            Self::LabelLargeEmphasized => FontRole::Plain,
            Self::LabelMediumEmphasized => FontRole::Plain,
            Self::LabelSmallEmphasized => FontRole::Plain,
        }
    }

    pub fn weight(self) -> FontWeight {
        match self {
            Self::DisplayLarge => FontWeight::NORMAL,
            Self::DisplayMedium => FontWeight::NORMAL,
            Self::DisplaySmall => FontWeight::NORMAL,
            Self::HeadlineLarge => FontWeight::NORMAL,
            Self::HeadlineMedium => FontWeight::NORMAL,
            Self::HeadlineSmall => FontWeight::NORMAL,
            Self::TitleLarge => FontWeight::NORMAL,
            Self::TitleMedium => FontWeight::MEDIUM,
            Self::TitleSmall => FontWeight::MEDIUM,
            Self::BodyLarge => FontWeight::NORMAL,
            Self::BodyMedium => FontWeight::NORMAL,
            Self::BodySmall => FontWeight::NORMAL,
            Self::LabelLarge => FontWeight::MEDIUM,
            Self::LabelMedium => FontWeight::MEDIUM,
            Self::LabelSmall => FontWeight::MEDIUM,
            Self::DisplayLargeEmphasized => FontWeight::MEDIUM,
            Self::DisplayMediumEmphasized => FontWeight::MEDIUM,
            Self::DisplaySmallEmphasized => FontWeight::MEDIUM,
            Self::HeadlineLargeEmphasized => FontWeight::MEDIUM,
            Self::HeadlineMediumEmphasized => FontWeight::MEDIUM,
            Self::HeadlineSmallEmphasized => FontWeight::MEDIUM,
            Self::TitleLargeEmphasized => FontWeight::MEDIUM,
            Self::TitleMediumEmphasized => FontWeight::BOLD,
            Self::TitleSmallEmphasized => FontWeight::BOLD,
            Self::BodyLargeEmphasized => FontWeight::MEDIUM,
            Self::BodyMediumEmphasized => FontWeight::MEDIUM,
            Self::BodySmallEmphasized => FontWeight::MEDIUM,
            Self::LabelLargeEmphasized => FontWeight::BOLD,
            Self::LabelMediumEmphasized => FontWeight::BOLD,
            Self::LabelSmallEmphasized => FontWeight::BOLD,
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
            Self::DisplayLargeEmphasized => 57.0,
            Self::DisplayMediumEmphasized => 45.0,
            Self::DisplaySmallEmphasized => 36.0,
            Self::HeadlineLargeEmphasized => 32.0,
            Self::HeadlineMediumEmphasized => 28.0,
            Self::HeadlineSmallEmphasized => 24.0,
            Self::TitleLargeEmphasized => 22.0,
            Self::TitleMediumEmphasized => 16.0,
            Self::TitleSmallEmphasized => 14.0,
            Self::BodyLargeEmphasized => 16.0,
            Self::BodyMediumEmphasized => 14.0,
            Self::BodySmallEmphasized => 12.0,
            Self::LabelLargeEmphasized => 14.0,
            Self::LabelMediumEmphasized => 12.0,
            Self::LabelSmallEmphasized => 11.0,
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
            Self::DisplayLargeEmphasized => 64.0,
            Self::DisplayMediumEmphasized => 52.0,
            Self::DisplaySmallEmphasized => 44.0,
            Self::HeadlineLargeEmphasized => 40.0,
            Self::HeadlineMediumEmphasized => 36.0,
            Self::HeadlineSmallEmphasized => 32.0,
            Self::TitleLargeEmphasized => 28.0,
            Self::TitleMediumEmphasized => 24.0,
            Self::TitleSmallEmphasized => 20.0,
            Self::BodyLargeEmphasized => 24.0,
            Self::BodyMediumEmphasized => 20.0,
            Self::BodySmallEmphasized => 16.0,
            Self::LabelLargeEmphasized => 20.0,
            Self::LabelMediumEmphasized => 16.0,
            Self::LabelSmallEmphasized => 16.0,
        }
    }

    pub const fn tracking(self) -> f32 {
        match self {
            Self::DisplayLarge => -0.25,
            Self::DisplayMedium => 0.0,
            Self::DisplaySmall => 0.0,
            Self::HeadlineLarge => 0.0,
            Self::HeadlineMedium => 0.0,
            Self::HeadlineSmall => 0.0,
            Self::TitleLarge => 0.0,
            Self::TitleMedium => 0.15,
            Self::TitleSmall => 0.1,
            Self::BodyLarge => 0.5,
            Self::BodyMedium => 0.25,
            Self::BodySmall => 0.4,
            Self::LabelLarge => 0.1,
            Self::LabelMedium => 0.5,
            Self::LabelSmall => 0.5,
            Self::DisplayLargeEmphasized => -0.25,
            Self::DisplayMediumEmphasized => 0.0,
            Self::DisplaySmallEmphasized => 0.0,
            Self::HeadlineLargeEmphasized => 0.0,
            Self::HeadlineMediumEmphasized => 0.0,
            Self::HeadlineSmallEmphasized => 0.0,
            Self::TitleLargeEmphasized => 0.0,
            Self::TitleMediumEmphasized => 0.15,
            Self::TitleSmallEmphasized => 0.1,
            Self::BodyLargeEmphasized => 0.5,
            Self::BodyMediumEmphasized => 0.25,
            Self::BodySmallEmphasized => 0.4,
            Self::LabelLargeEmphasized => 0.1,
            Self::LabelMediumEmphasized => 0.5,
            Self::LabelSmallEmphasized => 0.5,
        }
    }

    pub const fn is_emphasized(self) -> bool {
        matches!(
            self,
            Self::DisplayLargeEmphasized
                | Self::DisplayMediumEmphasized
                | Self::DisplaySmallEmphasized
                | Self::HeadlineLargeEmphasized
                | Self::HeadlineMediumEmphasized
                | Self::HeadlineSmallEmphasized
                | Self::TitleLargeEmphasized
                | Self::TitleMediumEmphasized
                | Self::TitleSmallEmphasized
                | Self::BodyLargeEmphasized
                | Self::BodyMediumEmphasized
                | Self::BodySmallEmphasized
                | Self::LabelLargeEmphasized
                | Self::LabelMediumEmphasized
                | Self::LabelSmallEmphasized
        )
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
        assert!(!TypeStyle::BodyLarge.is_emphasized());
    }

    #[test]
    fn emphasized_body_large_uses_medium_weight() {
        assert_eq!(TypeStyle::BodyLargeEmphasized.weight(), FontWeight::MEDIUM);
        assert_eq!(TypeStyle::BodyLargeEmphasized.size_sp(), 16.0);
        assert!(TypeStyle::BodyLargeEmphasized.is_emphasized());
    }

    #[test]
    fn emphasized_labels_use_bold() {
        assert_eq!(TypeStyle::LabelSmallEmphasized.weight(), FontWeight::BOLD);
        assert_eq!(TypeStyle::TitleMediumEmphasized.weight(), FontWeight::BOLD);
    }

    #[test]
    fn display_styles_use_brand_font() {
        assert_eq!(TypeStyle::DisplayLarge.font(), FontRole::Brand);
        assert_eq!(TypeStyle::DisplayLarge.size_sp(), 57.0);
    }

    #[test]
    fn full_scale_has_thirty_styles() {
        assert_eq!(TypeStyle::ALL.len(), 30);
    }
}
