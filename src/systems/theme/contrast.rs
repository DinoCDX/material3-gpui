use gpui::{Hsla, Rgba};

const LINEAR_THRESHOLD: f32 = 0.03928;
const LINEAR_DIVISOR: f32 = 12.92;
const GAMMA_OFFSET: f32 = 0.055;
const GAMMA_SCALE: f32 = 1.055;
const GAMMA: f32 = 2.4;
const RED_WEIGHT: f32 = 0.2126;
const GREEN_WEIGHT: f32 = 0.7152;
const BLUE_WEIGHT: f32 = 0.0722;
const FLARE: f32 = 0.05;

pub const AA_TEXT_CONTRAST: f32 = 4.5;

fn linear_channel(channel: f32) -> f32 {
    if channel <= LINEAR_THRESHOLD {
        channel / LINEAR_DIVISOR
    } else {
        ((channel + GAMMA_OFFSET) / GAMMA_SCALE).powf(GAMMA)
    }
}

pub fn relative_luminance(color: Hsla) -> f32 {
    let rgba = Rgba::from(color);
    RED_WEIGHT * linear_channel(rgba.r)
        + GREEN_WEIGHT * linear_channel(rgba.g)
        + BLUE_WEIGHT * linear_channel(rgba.b)
}

pub fn contrast_ratio(a: Hsla, b: Hsla) -> f32 {
    let luminance_a = relative_luminance(a);
    let luminance_b = relative_luminance(b);
    let (lighter, darker) = if luminance_a > luminance_b {
        (luminance_a, luminance_b)
    } else {
        (luminance_b, luminance_a)
    };
    (lighter + FLARE) / (darker + FLARE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::{MaterialTheme, ThemeMode};
    use crate::tokens::color::ColorRole;
    use gpui::{black, white};
    use material_color_utils::utils::color_utils::Argb;

    const SEEDS: [u32; 4] = [0xFF4285F4, 0xFF34A853, 0xFFEA4335, 0xFF9C27B0];

    const TEXT_PAIRS: [(ColorRole, ColorRole); 12] = [
        (ColorRole::Primary, ColorRole::OnPrimary),
        (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer),
        (ColorRole::Secondary, ColorRole::OnSecondary),
        (ColorRole::SecondaryContainer, ColorRole::OnSecondaryContainer),
        (ColorRole::Tertiary, ColorRole::OnTertiary),
        (ColorRole::TertiaryContainer, ColorRole::OnTertiaryContainer),
        (ColorRole::Error, ColorRole::OnError),
        (ColorRole::ErrorContainer, ColorRole::OnErrorContainer),
        (ColorRole::Background, ColorRole::OnBackground),
        (ColorRole::Surface, ColorRole::OnSurface),
        (ColorRole::SurfaceVariant, ColorRole::OnSurfaceVariant),
        (ColorRole::InverseSurface, ColorRole::InverseOnSurface),
    ];

    #[test]
    fn black_on_white_is_twenty_one_to_one() {
        assert!((contrast_ratio(black(), white()) - 21.0).abs() < 0.01);
    }

    #[test]
    fn contrast_is_symmetric() {
        let a = black();
        let b = white();
        assert_eq!(contrast_ratio(a, b), contrast_ratio(b, a));
    }

    #[test]
    fn every_text_pair_meets_aa_in_both_modes_for_every_seed() {
        for seed in SEEDS {
            for mode in [ThemeMode::Light, ThemeMode::Dark] {
                let theme = MaterialTheme::from_seed(Argb(seed), mode);
                for (background, text) in TEXT_PAIRS {
                    let ratio = contrast_ratio(theme.color(background), theme.color(text));
                    assert!(
                        ratio >= AA_TEXT_CONTRAST,
                        "seed {seed:#x} {mode:?}: {background:?} on {text:?} is {ratio:.2}:1"
                    );
                }
            }
        }
    }
}
