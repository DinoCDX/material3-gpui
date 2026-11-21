use gpui::Hsla;

use crate::systems::blend::{composite_over, with_alpha};
use crate::tokens::color::{TONAL_ELEVATION_BASE_PERCENT, TONAL_ELEVATION_LOG_SCALE};

pub fn tonal_elevation_alpha(elevation_dp: f32) -> f32 {
    if elevation_dp <= 0.0 {
        return 0.0;
    }
    (TONAL_ELEVATION_LOG_SCALE * (elevation_dp + 1.0).ln() + TONAL_ELEVATION_BASE_PERCENT) / 100.0
}

pub fn surface_color_at_elevation(surface: Hsla, tint: Hsla, elevation_dp: f32) -> Hsla {
    if elevation_dp <= 0.0 {
        return surface;
    }
    composite_over(with_alpha(tint, tonal_elevation_alpha(elevation_dp)), surface)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Rgba, black, white};

    #[test]
    fn no_tint_at_rest() {
        assert_eq!(tonal_elevation_alpha(0.0), 0.0);
    }

    #[test]
    fn tint_alpha_matches_m3_formula_at_three_dp() {
        let expected = (4.5 * 4.0_f32.ln() + 2.0) / 100.0;
        assert!((tonal_elevation_alpha(3.0) - expected).abs() < 1e-6);
    }

    #[test]
    fn rest_elevation_returns_surface_unchanged() {
        let surface = Rgba::from(surface_color_at_elevation(white(), black(), 0.0));
        assert!((surface.r - 1.0).abs() < 1e-3);
    }

    #[test]
    fn elevated_surface_darkens_toward_tint() {
        let elevated = Rgba::from(surface_color_at_elevation(white(), black(), 5.0));
        assert!(elevated.r < 1.0);
    }
}
