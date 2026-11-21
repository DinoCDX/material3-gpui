use gpui::{Hsla, Rgba};
use material_color_utils::{MaterializedScheme, utils::color_utils::Argb};

use crate::tokens::color::ColorRole;

#[derive(Clone, Debug)]
pub struct ColorScheme {
    colors: [Hsla; ColorRole::COUNT],
}

impl ColorScheme {
    pub fn from_materialized(scheme: &MaterializedScheme) -> Self {
        Self {
            colors: ColorRole::ALL.map(|role| argb_to_hsla(argb_of(scheme, role))),
        }
    }

    pub fn get(&self, role: ColorRole) -> Hsla {
        self.colors[role as usize]
    }
}

fn argb_to_hsla(argb: Argb) -> Hsla {
    let [a, r, g, b] = argb
        .0
        .to_be_bytes()
        .map(|channel| f32::from(channel) / 255.0);
    Hsla::from(Rgba { r, g, b, a })
}

fn argb_of(scheme: &MaterializedScheme, role: ColorRole) -> Argb {
    match role {
        ColorRole::Primary => scheme.primary,
        ColorRole::OnPrimary => scheme.on_primary,
        ColorRole::PrimaryContainer => scheme.primary_container,
        ColorRole::OnPrimaryContainer => scheme.on_primary_container,
        ColorRole::InversePrimary => scheme.inverse_primary,
        ColorRole::Secondary => scheme.secondary,
        ColorRole::OnSecondary => scheme.on_secondary,
        ColorRole::SecondaryContainer => scheme.secondary_container,
        ColorRole::OnSecondaryContainer => scheme.on_secondary_container,
        ColorRole::Tertiary => scheme.tertiary,
        ColorRole::OnTertiary => scheme.on_tertiary,
        ColorRole::TertiaryContainer => scheme.tertiary_container,
        ColorRole::OnTertiaryContainer => scheme.on_tertiary_container,
        ColorRole::Error => scheme.error,
        ColorRole::OnError => scheme.on_error,
        ColorRole::ErrorContainer => scheme.error_container,
        ColorRole::OnErrorContainer => scheme.on_error_container,
        ColorRole::Background => scheme.background,
        ColorRole::OnBackground => scheme.on_background,
        ColorRole::Surface => scheme.surface,
        ColorRole::SurfaceDim => scheme.surface_dim,
        ColorRole::SurfaceBright => scheme.surface_bright,
        ColorRole::SurfaceTint => scheme.surface_tint,
        ColorRole::SurfaceContainerLowest => scheme.surface_container_lowest,
        ColorRole::SurfaceContainerLow => scheme.surface_container_low,
        ColorRole::SurfaceContainer => scheme.surface_container,
        ColorRole::SurfaceContainerHigh => scheme.surface_container_high,
        ColorRole::SurfaceContainerHighest => scheme.surface_container_highest,
        ColorRole::OnSurface => scheme.on_surface,
        ColorRole::SurfaceVariant => scheme.surface_variant,
        ColorRole::OnSurfaceVariant => scheme.on_surface_variant,
        ColorRole::Outline => scheme.outline,
        ColorRole::OutlineVariant => scheme.outline_variant,
        ColorRole::InverseSurface => scheme.inverse_surface,
        ColorRole::InverseOnSurface => scheme.inverse_on_surface,
        ColorRole::Shadow => scheme.shadow,
        ColorRole::Scrim => scheme.scrim,
    }
}
