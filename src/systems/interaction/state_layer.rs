use gpui::Hsla;

use crate::systems::blend::with_alpha;
use crate::tokens::state::StateLayer;

pub fn overlay_color(content: Hsla, layer: StateLayer) -> Hsla {
    with_alpha(content, layer.opacity())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::black;

    #[test]
    fn overlay_uses_content_color_at_layer_opacity() {
        let overlay = overlay_color(black(), StateLayer::Hover);
        assert_eq!(overlay.a, StateLayer::Hover.opacity());
        assert_eq!(overlay.l, black().l);
    }
}
