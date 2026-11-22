use gpui::{Corners, Pixels, px};

pub const FULL_CORNER_DP: f32 = 9999.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub const fn as_array(self) -> [f32; 4] {
        [self.top_left, self.top_right, self.bottom_right, self.bottom_left]
    }

    pub fn pixels(self) -> Corners<Pixels> {
        Corners {
            top_left: px(self.top_left),
            top_right: px(self.top_right),
            bottom_right: px(self.bottom_right),
            bottom_left: px(self.bottom_left),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeToken {
    CornerNone,
    CornerExtraSmall,
    CornerExtraSmallTop,
    CornerSmall,
    CornerMedium,
    CornerLarge,
    CornerLargeIncreased,
    CornerLargeTop,
    CornerLargeStart,
    CornerLargeEnd,
    CornerExtraLarge,
    CornerExtraLargeIncreased,
    CornerExtraLargeTop,
    CornerExtraExtraLarge,
    CornerFull,
}

impl ShapeToken {
    pub const ALL: [ShapeToken; 15] = [
        ShapeToken::CornerNone,
        ShapeToken::CornerExtraSmall,
        ShapeToken::CornerExtraSmallTop,
        ShapeToken::CornerSmall,
        ShapeToken::CornerMedium,
        ShapeToken::CornerLarge,
        ShapeToken::CornerLargeIncreased,
        ShapeToken::CornerLargeTop,
        ShapeToken::CornerLargeStart,
        ShapeToken::CornerLargeEnd,
        ShapeToken::CornerExtraLarge,
        ShapeToken::CornerExtraLargeIncreased,
        ShapeToken::CornerExtraLargeTop,
        ShapeToken::CornerExtraExtraLarge,
        ShapeToken::CornerFull,
    ];

    pub const fn radii(self) -> CornerRadii {
        match self {
            Self::CornerNone => CornerRadii::all(0.0),
            Self::CornerExtraSmall => CornerRadii::all(4.0),
            Self::CornerExtraSmallTop => CornerRadii {
                top_left: 4.0,
                top_right: 4.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            Self::CornerSmall => CornerRadii::all(8.0),
            Self::CornerMedium => CornerRadii::all(12.0),
            Self::CornerLarge => CornerRadii::all(16.0),
            Self::CornerLargeIncreased => CornerRadii::all(20.0),
            Self::CornerLargeTop => CornerRadii {
                top_left: 16.0,
                top_right: 16.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            Self::CornerLargeStart => CornerRadii {
                top_left: 16.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 16.0,
            },
            Self::CornerLargeEnd => CornerRadii {
                top_left: 0.0,
                top_right: 16.0,
                bottom_right: 16.0,
                bottom_left: 0.0,
            },
            Self::CornerExtraLarge => CornerRadii::all(28.0),
            Self::CornerExtraLargeIncreased => CornerRadii::all(32.0),
            Self::CornerExtraLargeTop => CornerRadii {
                top_left: 28.0,
                top_right: 28.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            Self::CornerExtraExtraLarge => CornerRadii::all(48.0),
            Self::CornerFull => CornerRadii::all(FULL_CORNER_DP),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_scale_matches_m3_values() {
        assert_eq!(ShapeToken::CornerNone.radii(), CornerRadii::all(0.0));
        assert_eq!(ShapeToken::CornerMedium.radii(), CornerRadii::all(12.0));
        assert_eq!(ShapeToken::CornerExtraExtraLarge.radii(), CornerRadii::all(48.0));
    }

    #[test]
    fn increased_sizes_match_m3_values() {
        assert_eq!(ShapeToken::CornerLargeIncreased.radii(), CornerRadii::all(20.0));
        assert_eq!(ShapeToken::CornerExtraLargeIncreased.radii(), CornerRadii::all(32.0));
    }

    #[test]
    fn directional_shapes_round_only_their_named_corners() {
        let start = ShapeToken::CornerLargeStart.radii();
        assert_eq!((start.top_left, start.top_right), (16.0, 0.0));
        assert_eq!((start.bottom_right, start.bottom_left), (0.0, 16.0));
        let end = ShapeToken::CornerLargeEnd.radii();
        assert_eq!((end.top_left, end.top_right), (0.0, 16.0));
        assert_eq!((end.bottom_right, end.bottom_left), (16.0, 0.0));
        let top = ShapeToken::CornerExtraLargeTop.radii();
        assert_eq!((top.bottom_right, top.bottom_left), (0.0, 0.0));
    }

    #[test]
    fn every_token_is_listed_once() {
        assert_eq!(ShapeToken::ALL.len(), 15);
    }
}
