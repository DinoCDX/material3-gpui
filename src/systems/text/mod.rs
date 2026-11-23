pub mod layout;

use gpui::{
    AbsoluteLength, App, DefiniteLength, Hsla, IntoElement, Pixels, RenderOnce, SharedString,
    ShapedLine, TextStyle, Window, canvas, div, point, prelude::*, px,
};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::tokens::typography::TypeStyle;
use layout::glyph_positions;

const CACHE_LIMIT: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ShapeKey {
    text: String,
    style: TypeStyle,
    family: String,
    color: [u32; 4],
}

thread_local! {
    static SHAPE_CACHE: RefCell<HashMap<ShapeKey, Rc<ShapedText>>> = RefCell::new(HashMap::new());
}

pub struct ShapedText {
    pub glyphs: Vec<ShapedLine>,
    pub positions: Vec<f32>,
    pub width: f32,
}

pub fn text_style_for(style: TypeStyle, color: Hsla, family: SharedString) -> TextStyle {
    TextStyle {
        color,
        font_family: family,
        font_size: px(style.size_sp()).into(),
        line_height: DefiniteLength::Absolute(AbsoluteLength::Pixels(px(style.line_height()))),
        font_weight: style.weight(),
        ..Default::default()
    }
}

fn shape_piece(window: &Window, text: &str, font_size: Pixels, style: &TextStyle) -> ShapedLine {
    let run = style.to_run(text.len());
    window
        .text_system()
        .shape_line(SharedString::from(text.to_string()), font_size, &[run], None)
}

pub fn shape_tracked(
    window: &Window,
    text: &str,
    font_size: Pixels,
    style: &TextStyle,
    tracking: f32,
) -> ShapedText {
    let characters: Vec<String> = text.chars().map(|character| character.to_string()).collect();
    let glyphs: Vec<ShapedLine> = characters
        .iter()
        .map(|character| shape_piece(window, character, font_size, style))
        .collect();
    let advances: Vec<f32> = glyphs.iter().map(|glyph| f32::from(&glyph.width)).collect();
    let kerning: Vec<f32> = characters
        .windows(2)
        .enumerate()
        .map(|(index, pair)| {
            let joined = format!("{}{}", pair[0], pair[1]);
            let joined_width = f32::from(&shape_piece(window, &joined, font_size, style).width);
            joined_width - advances[index] - advances[index + 1]
        })
        .collect();
    let (positions, width) = glyph_positions(&advances, &kerning, tracking);
    ShapedText {
        glyphs,
        positions,
        width,
    }
}

pub fn shape_cached(
    window: &Window,
    text: &str,
    style: TypeStyle,
    family: &SharedString,
    color: Hsla,
) -> Rc<ShapedText> {
    let key = ShapeKey {
        text: text.to_string(),
        style,
        family: family.to_string(),
        color: [color.h, color.s, color.l, color.a].map(f32::to_bits),
    };
    SHAPE_CACHE.with(|cache| {
        if let Some(hit) = cache.borrow().get(&key) {
            return Rc::clone(hit);
        }
        let text_style = text_style_for(style, color, family.clone());
        let shaped = Rc::new(shape_tracked(
            window,
            text,
            px(style.size_sp()),
            &text_style,
            style.tracking(),
        ));
        let mut cache = cache.borrow_mut();
        if cache.len() >= CACHE_LIMIT {
            cache.clear();
        }
        cache.insert(key, Rc::clone(&shaped));
        shaped
    })
}

pub fn measure_text(window: &Window, text: &str, style: TypeStyle, family: SharedString) -> f32 {
    shape_cached(window, text, style, &family, gpui::black()).width
}

#[derive(IntoElement)]
pub struct TrackedText {
    text: SharedString,
    style: TypeStyle,
    color: Hsla,
    family: SharedString,
}

impl TrackedText {
    pub fn new(
        text: impl Into<SharedString>,
        style: TypeStyle,
        color: Hsla,
        family: SharedString,
    ) -> Self {
        Self {
            text: text.into(),
            style,
            color,
            family,
        }
    }
}

impl RenderOnce for TrackedText {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let shaped = shape_cached(window, &self.text, self.style, &self.family, self.color);
        let width = shaped.width;
        let line_height = self.style.line_height();
        let paint_source = Rc::clone(&shaped);
        div().relative().w(px(width)).h(px(line_height)).child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, cx| {
                    for (glyph, x) in paint_source.glyphs.iter().zip(&paint_source.positions) {
                        let origin = point(bounds.origin.x + px(*x), bounds.origin.y);
                        let _ = glyph.paint(origin, px(line_height), window, cx);
                    }
                },
            )
            .absolute()
            .size_full(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::typography::Typography;

    #[test]
    fn style_carries_the_reference_metrics() {
        let style = text_style_for(TypeStyle::BodyLarge, gpui::black(), SharedString::from("x"));
        assert_eq!(style.font_weight, TypeStyle::BodyLarge.weight());
        assert!(!Typography::default().font_family(TypeStyle::BodyLarge.font()).is_empty());
    }

    #[test]
    fn cache_key_separates_text_style_and_family() {
        let a = ShapeKey {
            text: "A".into(),
            style: TypeStyle::LabelLarge,
            family: "f".into(),
            color: [0; 4],
        };
        let b = ShapeKey {
            text: "A".into(),
            style: TypeStyle::BodyLarge,
            family: "f".into(),
            color: [0; 4],
        };
        assert_ne!(a, b);
    }
}
