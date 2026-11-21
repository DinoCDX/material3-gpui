use gpui::{Pixels, px};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CornerRadius {
    None,
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    ExtraExtraLarge,
    Full,
}

impl CornerRadius {
    pub const FULL_DP: f32 = 9999.0;

    pub const fn dp(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::ExtraSmall => 4.0,
            Self::Small => 8.0,
            Self::Medium => 12.0,
            Self::Large => 16.0,
            Self::ExtraLarge => 28.0,
            Self::ExtraExtraLarge => 48.0,
            Self::Full => Self::FULL_DP,
        }
    }

    pub fn pixels(self) -> Pixels {
        px(self.dp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_scale_matches_m3_values() {
        assert_eq!(CornerRadius::None.dp(), 0.0);
        assert_eq!(CornerRadius::Medium.dp(), 12.0);
        assert_eq!(CornerRadius::ExtraExtraLarge.dp(), 48.0);
        assert_eq!(CornerRadius::Full.dp(), CornerRadius::FULL_DP);
    }
}
