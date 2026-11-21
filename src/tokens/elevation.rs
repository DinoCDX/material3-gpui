use gpui::{Pixels, px};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub alpha: f32,
}

const KEY_SHADOW_ALPHA: f32 = 0.3;
const AMBIENT_SHADOW_ALPHA: f32 = 0.15;

const fn layer(offset_y: f32, blur: f32, spread: f32, alpha: f32) -> ShadowLayer {
    ShadowLayer {
        offset_y,
        blur,
        spread,
        alpha,
    }
}

const LEVEL0_SHADOWS: &[ShadowLayer] = &[];
const LEVEL1_SHADOWS: &[ShadowLayer] = &[
    layer(1.0, 2.0, 0.0, KEY_SHADOW_ALPHA),
    layer(1.0, 3.0, 1.0, AMBIENT_SHADOW_ALPHA),
];
const LEVEL2_SHADOWS: &[ShadowLayer] = &[
    layer(1.0, 2.0, 0.0, KEY_SHADOW_ALPHA),
    layer(2.0, 6.0, 2.0, AMBIENT_SHADOW_ALPHA),
];
const LEVEL3_SHADOWS: &[ShadowLayer] = &[
    layer(1.0, 3.0, 0.0, KEY_SHADOW_ALPHA),
    layer(4.0, 8.0, 3.0, AMBIENT_SHADOW_ALPHA),
];
const LEVEL4_SHADOWS: &[ShadowLayer] = &[
    layer(2.0, 3.0, 0.0, KEY_SHADOW_ALPHA),
    layer(6.0, 10.0, 4.0, AMBIENT_SHADOW_ALPHA),
];
const LEVEL5_SHADOWS: &[ShadowLayer] = &[
    layer(4.0, 4.0, 0.0, KEY_SHADOW_ALPHA),
    layer(8.0, 12.0, 6.0, AMBIENT_SHADOW_ALPHA),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ElevationLevel {
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl ElevationLevel {
    pub const fn dp(self) -> f32 {
        match self {
            Self::Level0 => 0.0,
            Self::Level1 => 1.0,
            Self::Level2 => 3.0,
            Self::Level3 => 6.0,
            Self::Level4 => 8.0,
            Self::Level5 => 12.0,
        }
    }

    pub fn pixels(self) -> Pixels {
        px(self.dp())
    }

    pub const fn shadow_layers(self) -> &'static [ShadowLayer] {
        match self {
            Self::Level0 => LEVEL0_SHADOWS,
            Self::Level1 => LEVEL1_SHADOWS,
            Self::Level2 => LEVEL2_SHADOWS,
            Self::Level3 => LEVEL3_SHADOWS,
            Self::Level4 => LEVEL4_SHADOWS,
            Self::Level5 => LEVEL5_SHADOWS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level0_casts_no_shadow() {
        assert!(ElevationLevel::Level0.shadow_layers().is_empty());
    }

    #[test]
    fn raised_levels_have_key_and_ambient_layers() {
        for level in [
            ElevationLevel::Level1,
            ElevationLevel::Level2,
            ElevationLevel::Level3,
            ElevationLevel::Level4,
            ElevationLevel::Level5,
        ] {
            assert_eq!(level.shadow_layers().len(), 2);
        }
    }

    #[test]
    fn level5_shadow_values_match_m3() {
        let layers = ElevationLevel::Level5.shadow_layers();
        assert_eq!(layers[0], layer(4.0, 4.0, 0.0, 0.3));
        assert_eq!(layers[1], layer(8.0, 12.0, 6.0, 0.15));
    }

    #[test]
    fn elevation_levels_match_m3_values() {
        assert_eq!(ElevationLevel::Level0.dp(), 0.0);
        assert_eq!(ElevationLevel::Level2.dp(), 3.0);
        assert_eq!(ElevationLevel::Level5.dp(), 12.0);
    }
}
