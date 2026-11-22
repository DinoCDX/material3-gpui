use gpui::{Hsla, transparent_black};

use super::tokens::DISABLED_FILLED_CONTAINER_OPACITY;
use crate::systems::blend::with_alpha;
use crate::systems::surface::{StateElevation, SurfaceColors};
use crate::systems::theme::MaterialTheme;
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::state::{DISABLED_CONTAINER_OPACITY, DISABLED_CONTENT_OPACITY};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    Filled,
    FilledTonal,
    Elevated,
    Outlined,
    Text,
}

impl ButtonVariant {
    pub fn elevation(self) -> StateElevation {
        match self {
            Self::Filled => StateElevation {
                default: ElevationLevel::Level0,
                hovered: ElevationLevel::Level1,
                focused: ElevationLevel::Level0,
                pressed: ElevationLevel::Level0,
                disabled: ElevationLevel::Level0,
            },
            Self::FilledTonal => StateElevation {
                default: ElevationLevel::Level0,
                hovered: ElevationLevel::Level1,
                focused: ElevationLevel::Level0,
                pressed: ElevationLevel::Level0,
                disabled: ElevationLevel::Level0,
            },
            Self::Elevated => StateElevation {
                default: ElevationLevel::Level1,
                hovered: ElevationLevel::Level2,
                focused: ElevationLevel::Level1,
                pressed: ElevationLevel::Level1,
                disabled: ElevationLevel::Level0,
            },
            Self::Outlined | Self::Text => {
                StateElevation::constant(ElevationLevel::Level0)
            }
        }
    }

    pub fn is_outlined(self) -> bool {
        matches!(self, Self::Outlined)
    }

    pub fn colors(self, theme: &MaterialTheme) -> SurfaceColors {
        let on_surface = theme.color(ColorRole::OnSurface);
        let on_surface_variant = theme.color(ColorRole::OnSurfaceVariant);
        let outline = theme.color(ColorRole::OutlineVariant);
        let transparent: Hsla = transparent_black();
        match self {
            Self::Filled => filled(theme, on_surface, on_surface_variant),
            Self::FilledTonal => SurfaceColors {
                container: theme.color(ColorRole::SecondaryContainer),
                content: theme.color(ColorRole::OnSecondaryContainer),
                disabled_container: with_alpha(on_surface, DISABLED_CONTAINER_OPACITY),
                disabled_content: with_alpha(on_surface, DISABLED_CONTENT_OPACITY),
                outline,
            },
            Self::Elevated => SurfaceColors {
                container: theme.color(ColorRole::SurfaceContainerLow),
                content: theme.color(ColorRole::Primary),
                disabled_container: with_alpha(on_surface, DISABLED_FILLED_CONTAINER_OPACITY),
                disabled_content: with_alpha(on_surface_variant, DISABLED_CONTENT_OPACITY),
                outline,
            },
            Self::Outlined => SurfaceColors {
                container: transparent,
                content: on_surface_variant,
                disabled_container: transparent,
                disabled_content: with_alpha(on_surface_variant, DISABLED_CONTENT_OPACITY),
                outline,
            },
            Self::Text => SurfaceColors {
                container: transparent,
                content: on_surface_variant,
                disabled_container: transparent,
                disabled_content: with_alpha(on_surface_variant, DISABLED_CONTENT_OPACITY),
                outline,
            },
        }
    }
}

fn filled(theme: &MaterialTheme, on_surface: Hsla, on_surface_variant: Hsla) -> SurfaceColors {
    SurfaceColors {
        container: theme.color(ColorRole::Primary),
        content: theme.color(ColorRole::OnPrimary),
        disabled_container: with_alpha(on_surface, DISABLED_FILLED_CONTAINER_OPACITY),
        disabled_content: with_alpha(on_surface_variant, DISABLED_CONTENT_OPACITY),
        outline: theme.color(ColorRole::OutlineVariant),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::MaterialTheme;
    use crate::systems::theme::ThemeMode;
    use material_color_utils::utils::color_utils::Argb;

    fn theme() -> MaterialTheme {
        MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light)
    }

    #[test]
    fn filled_uses_primary_container() {
        let theme = theme();
        let colors = ButtonVariant::Filled.colors(&theme);
        assert_eq!(colors.container.h, theme.color(ColorRole::Primary).h);
    }

    #[test]
    fn filled_disabled_container_uses_ten_percent_on_surface() {
        let theme = theme();
        let colors = ButtonVariant::Filled.colors(&theme);
        assert_eq!(colors.disabled_container.a, DISABLED_FILLED_CONTAINER_OPACITY);
    }

    #[test]
    fn only_outlined_is_outlined() {
        assert!(ButtonVariant::Outlined.is_outlined());
        assert!(!ButtonVariant::Filled.is_outlined());
    }

    #[test]
    fn elevated_rises_on_hover() {
        let elevation = ButtonVariant::Elevated.elevation();
        assert_eq!(elevation.default, ElevationLevel::Level1);
        assert_eq!(elevation.hovered, ElevationLevel::Level2);
    }
}
