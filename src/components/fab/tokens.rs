use crate::tokens::shape::ShapeToken;

pub const EXTENDED_HEIGHT: f32 = 56.0;
pub const EXTENDED_ICON_SIZE: f32 = 24.0;
pub const EXTENDED_LEADING_SPACE: f32 = 16.0;
pub const EXTENDED_ICON_LABEL_SPACE: f32 = 12.0;
pub const EXTENDED_TRAILING_SPACE: f32 = 20.0;
pub const EXTENDED_SHAPE: ShapeToken = ShapeToken::CornerLarge;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FabSize {
    Small,
    Standard,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FabSizeTokens {
    pub container_size: f32,
    pub icon_size: f32,
    pub shape: ShapeToken,
}

impl FabSize {
    pub const fn tokens(self) -> FabSizeTokens {
        match self {
            Self::Small => FabSizeTokens {
                container_size: 40.0,
                icon_size: 24.0,
                shape: ShapeToken::CornerMedium,
            },
            Self::Standard => FabSizeTokens {
                container_size: 56.0,
                icon_size: 24.0,
                shape: ShapeToken::CornerLarge,
            },
            Self::Large => FabSizeTokens {
                container_size: 96.0,
                icon_size: 36.0,
                shape: ShapeToken::CornerExtraLarge,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_sizes_match_the_reference() {
        assert_eq!(FabSize::Small.tokens().container_size, 40.0);
        assert_eq!(FabSize::Standard.tokens().container_size, 56.0);
        assert_eq!(FabSize::Large.tokens().container_size, 96.0);
    }

    #[test]
    fn large_icon_uses_the_component_value_not_the_token() {
        assert_eq!(FabSize::Large.tokens().icon_size, 36.0);
    }

    #[test]
    fn extended_fab_matches_the_reference() {
        assert_eq!(EXTENDED_HEIGHT, 56.0);
        assert_eq!(EXTENDED_SHAPE, ShapeToken::CornerLarge);
    }
}
