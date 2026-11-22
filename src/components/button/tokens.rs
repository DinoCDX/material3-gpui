use crate::tokens::shape::ShapeToken;
use crate::tokens::typography::TypeStyle;

pub const DISABLED_FILLED_CONTAINER_OPACITY: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonSize {
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonSizeTokens {
    pub container_height: f32,
    pub leading_space: f32,
    pub trailing_space: f32,
    pub icon_size: f32,
    pub icon_label_space: f32,
    pub outline_width: f32,
    pub square_shape: ShapeToken,
    pub pressed_square_shape: ShapeToken,
    pub label_style: TypeStyle,
}

impl ButtonSize {
    pub const fn tokens(self) -> ButtonSizeTokens {
        match self {
            Self::XSmall => ButtonSizeTokens {
                container_height: 32.0,
                leading_space: 16.0,
                trailing_space: 16.0,
                icon_size: 20.0,
                icon_label_space: 8.0,
                outline_width: 1.0,
                square_shape: ShapeToken::CornerMedium,
                pressed_square_shape: ShapeToken::CornerSmall,
                label_style: TypeStyle::LabelLarge,
            },
            Self::Small => ButtonSizeTokens {
                container_height: 40.0,
                leading_space: 16.0,
                trailing_space: 16.0,
                icon_size: 20.0,
                icon_label_space: 8.0,
                outline_width: 1.0,
                square_shape: ShapeToken::CornerMedium,
                pressed_square_shape: ShapeToken::CornerSmall,
                label_style: TypeStyle::LabelLarge,
            },
            Self::Medium => ButtonSizeTokens {
                container_height: 56.0,
                leading_space: 24.0,
                trailing_space: 24.0,
                icon_size: 24.0,
                icon_label_space: 8.0,
                outline_width: 1.0,
                square_shape: ShapeToken::CornerLarge,
                pressed_square_shape: ShapeToken::CornerMedium,
                label_style: TypeStyle::TitleMedium,
            },
            Self::Large => ButtonSizeTokens {
                container_height: 96.0,
                leading_space: 48.0,
                trailing_space: 48.0,
                icon_size: 32.0,
                icon_label_space: 12.0,
                outline_width: 2.0,
                square_shape: ShapeToken::CornerExtraLarge,
                pressed_square_shape: ShapeToken::CornerLarge,
                label_style: TypeStyle::HeadlineSmall,
            },
            Self::XLarge => ButtonSizeTokens {
                container_height: 136.0,
                leading_space: 64.0,
                trailing_space: 64.0,
                icon_size: 40.0,
                icon_label_space: 16.0,
                outline_width: 3.0,
                square_shape: ShapeToken::CornerExtraLarge,
                pressed_square_shape: ShapeToken::CornerLarge,
                label_style: TypeStyle::HeadlineLarge,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonShape {
    Round,
    Square,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_heights_match_the_reference() {
        assert_eq!(ButtonSize::XSmall.tokens().container_height, 32.0);
        assert_eq!(ButtonSize::Small.tokens().container_height, 40.0);
        assert_eq!(ButtonSize::Medium.tokens().container_height, 56.0);
        assert_eq!(ButtonSize::Large.tokens().container_height, 96.0);
        assert_eq!(ButtonSize::XLarge.tokens().container_height, 136.0);
    }

    #[test]
    fn label_style_steps_up_with_size() {
        assert_eq!(ButtonSize::Small.tokens().label_style, TypeStyle::LabelLarge);
        assert_eq!(ButtonSize::Medium.tokens().label_style, TypeStyle::TitleMedium);
        assert_eq!(ButtonSize::XLarge.tokens().label_style, TypeStyle::HeadlineLarge);
    }

    #[test]
    fn outline_width_grows_with_size() {
        assert_eq!(ButtonSize::Small.tokens().outline_width, 1.0);
        assert_eq!(ButtonSize::Large.tokens().outline_width, 2.0);
        assert_eq!(ButtonSize::XLarge.tokens().outline_width, 3.0);
    }
}
