use crate::tokens::shape::ShapeToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconButtonSize {
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconButtonWidth {
    Default,
    Narrow,
    Wide,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconButtonSizeTokens {
    pub container_size: f32,
    pub icon_size: f32,
    pub outline_width: f32,
    pub square_shape: ShapeToken,
    pub narrow_space: f32,
    pub wide_space: f32,
}

impl IconButtonSizeTokens {
    pub fn container_width(self, width: IconButtonWidth) -> f32 {
        match width {
            IconButtonWidth::Default => self.container_size,
            IconButtonWidth::Narrow => self.icon_size + 2.0 * self.narrow_space,
            IconButtonWidth::Wide => self.icon_size + 2.0 * self.wide_space,
        }
    }
}

impl IconButtonSize {
    pub const fn tokens(self) -> IconButtonSizeTokens {
        match self {
            Self::XSmall => IconButtonSizeTokens {
                container_size: 32.0,
                icon_size: 20.0,
                outline_width: 1.0,
                square_shape: ShapeToken::CornerMedium,
                narrow_space: 4.0,
                wide_space: 10.0,
            },
            Self::Small => IconButtonSizeTokens {
                container_size: 40.0,
                icon_size: 24.0,
                outline_width: 1.0,
                square_shape: ShapeToken::CornerMedium,
                narrow_space: 4.0,
                wide_space: 14.0,
            },
            Self::Medium => IconButtonSizeTokens {
                container_size: 56.0,
                icon_size: 24.0,
                outline_width: 1.0,
                square_shape: ShapeToken::CornerLarge,
                narrow_space: 12.0,
                wide_space: 24.0,
            },
            Self::Large => IconButtonSizeTokens {
                container_size: 96.0,
                icon_size: 32.0,
                outline_width: 2.0,
                square_shape: ShapeToken::CornerExtraLarge,
                narrow_space: 16.0,
                wide_space: 48.0,
            },
            Self::XLarge => IconButtonSizeTokens {
                container_size: 136.0,
                icon_size: 40.0,
                outline_width: 3.0,
                square_shape: ShapeToken::CornerExtraLarge,
                narrow_space: 32.0,
                wide_space: 72.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_sizes_match_the_reference() {
        assert_eq!(IconButtonSize::XSmall.tokens().container_size, 32.0);
        assert_eq!(IconButtonSize::Small.tokens().container_size, 40.0);
        assert_eq!(IconButtonSize::Medium.tokens().container_size, 56.0);
        assert_eq!(IconButtonSize::Large.tokens().container_size, 96.0);
        assert_eq!(IconButtonSize::XLarge.tokens().container_size, 136.0);
    }

    #[test]
    fn icon_sizes_grow_with_container() {
        assert_eq!(IconButtonSize::Small.tokens().icon_size, 24.0);
        assert_eq!(IconButtonSize::XLarge.tokens().icon_size, 40.0);
    }
}
