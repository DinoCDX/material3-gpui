pub const DISABLED_CONTAINER_OPACITY: f32 = 0.12;
pub const DISABLED_CONTENT_OPACITY: f32 = 0.38;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StateLayer {
    Hover,
    Focus,
    Press,
    Drag,
}

impl StateLayer {
    pub const fn opacity(self) -> f32 {
        match self {
            Self::Hover => 0.08,
            Self::Focus => 0.10,
            Self::Press => 0.10,
            Self::Drag => 0.16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_layer_opacities_match_m3() {
        assert_eq!(StateLayer::Hover.opacity(), 0.08);
        assert_eq!(StateLayer::Focus.opacity(), 0.10);
        assert_eq!(StateLayer::Press.opacity(), 0.10);
        assert_eq!(StateLayer::Drag.opacity(), 0.16);
    }

    #[test]
    fn disabled_opacities_match_m3() {
        assert_eq!(DISABLED_CONTAINER_OPACITY, 0.12);
        assert_eq!(DISABLED_CONTENT_OPACITY, 0.38);
    }
}
