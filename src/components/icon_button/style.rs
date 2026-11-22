use gpui::transparent_black;

use crate::systems::blend::with_alpha;
use crate::systems::surface::SurfaceColors;
use crate::systems::theme::MaterialTheme;
use crate::tokens::color::ColorRole;
use crate::tokens::state::{DISABLED_CONTAINER_OPACITY, DISABLED_CONTENT_OPACITY};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconButtonVariant {
    Standard,
    Filled,
    FilledTonal,
    Outlined,
}

impl IconButtonVariant {
    pub fn colors(self, theme: &MaterialTheme, selected: bool) -> SurfaceColors {
        let on_surface = theme.color(ColorRole::OnSurface);
        let on_surface_variant = theme.color(ColorRole::OnSurfaceVariant);
        let outline = theme.color(ColorRole::OutlineVariant);
        let transparent = transparent_black();
        let disabled_content = with_alpha(on_surface, DISABLED_CONTENT_OPACITY);
        let disabled_tint = with_alpha(on_surface, DISABLED_CONTAINER_OPACITY);
        match (self, selected) {
            (Self::Standard, false) => SurfaceColors {
                container: transparent,
                content: on_surface_variant,
                disabled_container: transparent,
                disabled_content,
                outline,
            },
            (Self::Standard, true) => SurfaceColors {
                container: transparent,
                content: theme.color(ColorRole::Primary),
                disabled_container: transparent,
                disabled_content,
                outline,
            },
            (Self::Filled, false) => SurfaceColors {
                container: theme.color(ColorRole::SurfaceContainer),
                content: on_surface_variant,
                disabled_container: with_alpha(on_surface, DISABLED_CONTAINER_OPACITY),
                disabled_content,
                outline,
            },
            (Self::Filled, true) => SurfaceColors {
                container: theme.color(ColorRole::Primary),
                content: theme.color(ColorRole::OnPrimary),
                disabled_container: disabled_tint,
                disabled_content,
                outline,
            },
            (Self::FilledTonal, false) => SurfaceColors {
                container: theme.color(ColorRole::SecondaryContainer),
                content: theme.color(ColorRole::OnSecondaryContainer),
                disabled_container: disabled_tint,
                disabled_content,
                outline,
            },
            (Self::FilledTonal, true) => SurfaceColors {
                container: theme.color(ColorRole::Secondary),
                content: theme.color(ColorRole::OnSecondary),
                disabled_container: disabled_tint,
                disabled_content,
                outline,
            },
            (Self::Outlined, false) => SurfaceColors {
                container: transparent,
                content: on_surface_variant,
                disabled_container: transparent,
                disabled_content,
                outline,
            },
            (Self::Outlined, true) => SurfaceColors {
                container: theme.color(ColorRole::InverseSurface),
                content: theme.color(ColorRole::InverseOnSurface),
                disabled_container: disabled_tint,
                disabled_content,
                outline,
            },
        }
    }

    pub fn is_outlined(self) -> bool {
        matches!(self, Self::Outlined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::ThemeMode;
    use material_color_utils::utils::color_utils::Argb;

    fn theme() -> MaterialTheme {
        MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light)
    }

    #[test]
    fn selected_filled_uses_primary() {
        let theme = theme();
        let selected = IconButtonVariant::Filled.colors(&theme, true);
        assert_eq!(selected.container.h, theme.color(ColorRole::Primary).h);
    }

    #[test]
    fn standard_unselected_has_no_container() {
        let theme = theme();
        let colors = IconButtonVariant::Standard.colors(&theme, false);
        assert_eq!(colors.container.a, 0.0);
    }

    #[test]
    fn only_outlined_is_outlined() {
        assert!(IconButtonVariant::Outlined.is_outlined());
        assert!(!IconButtonVariant::Standard.is_outlined());
    }
}
