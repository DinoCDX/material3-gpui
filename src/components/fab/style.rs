use crate::systems::surface::{StateElevation, SurfaceColors};
use crate::systems::theme::MaterialTheme;
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FabColor {
    Primary,
    Secondary,
}

impl FabColor {
    pub fn colors(self, theme: &MaterialTheme) -> SurfaceColors {
        let (container, content) = match self {
            Self::Primary => (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer),
            Self::Secondary => (ColorRole::SecondaryContainer, ColorRole::OnSecondaryContainer),
        };
        let mut colors = SurfaceColors::default_for(theme, 0.0);
        colors.container = theme.color(container);
        colors.content = theme.color(content);
        colors
    }
}

pub fn elevation() -> StateElevation {
    StateElevation {
        default: ElevationLevel::Level3,
        hovered: ElevationLevel::Level4,
        focused: ElevationLevel::Level3,
        pressed: ElevationLevel::Level3,
        disabled: ElevationLevel::Level0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::ThemeMode;
    use material_color_utils::utils::color_utils::Argb;

    #[test]
    fn fab_rises_to_level_three_and_four_on_hover() {
        let elevation = elevation();
        assert_eq!(elevation.default, ElevationLevel::Level3);
        assert_eq!(elevation.hovered, ElevationLevel::Level4);
    }

    #[test]
    fn primary_fab_uses_primary_container() {
        let theme = MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light);
        let colors = FabColor::Primary.colors(&theme);
        assert_eq!(colors.container.h, theme.color(ColorRole::PrimaryContainer).h);
    }
}
