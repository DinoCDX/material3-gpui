use crate::tokens::shape::ShapeToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExtendedFabSize {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtendedFabTokens {
    pub height: f32,
    pub icon_size: f32,
    pub icon_label_space: f32,
    pub leading_space: f32,
    pub trailing_space: f32,
    pub shape: ShapeToken,
}

impl ExtendedFabSize {
    pub const fn tokens(self) -> ExtendedFabTokens {
        match self {
            Self::Small => ExtendedFabTokens {
                height: 56.0,
                icon_size: 24.0,
                icon_label_space: 8.0,
                leading_space: 16.0,
                trailing_space: 16.0,
                shape: ShapeToken::CornerLarge,
            },
            Self::Medium => ExtendedFabTokens {
                height: 80.0,
                icon_size: 28.0,
                icon_label_space: 16.0,
                leading_space: 26.0,
                trailing_space: 26.0,
                shape: ShapeToken::CornerLarge,
            },
            Self::Large => ExtendedFabTokens {
                height: 96.0,
                icon_size: 32.0,
                icon_label_space: 20.0,
                leading_space: 28.0,
                trailing_space: 28.0,
                shape: ShapeToken::CornerExtraLarge,
            },
        }
    }
}

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
    fn extended_fab_sizes_match_the_reference() {
        assert_eq!(ExtendedFabSize::Small.tokens().height, 56.0);
        assert_eq!(ExtendedFabSize::Medium.tokens().height, 80.0);
        assert_eq!(ExtendedFabSize::Large.tokens().height, 96.0);
        assert_eq!(ExtendedFabSize::Large.tokens().shape, ShapeToken::CornerExtraLarge);
    }
}
