use gpui::{Pixels, px};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_levels_match_m3_values() {
        assert_eq!(ElevationLevel::Level0.dp(), 0.0);
        assert_eq!(ElevationLevel::Level2.dp(), 3.0);
        assert_eq!(ElevationLevel::Level5.dp(), 12.0);
    }
}
