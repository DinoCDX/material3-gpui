use gpui::{AnyElement, App, Hsla, IntoElement, RenderOnce, Window, canvas, div, px};

use crate::systems::geometry::{Point2, Stroke, paint_strokes};
use crate::systems::theme::ActiveTheme;
use crate::tokens::color::ColorRole;

const GRID: f32 = 24.0;
const STROKE: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconKind {
    Add,
    Remove,
    Close,
    Check,
}

impl IconKind {
    pub fn strokes(self) -> Vec<Stroke> {
        match self {
            Self::Add => vec![
                [(5.0, 12.0), (19.0, 12.0)],
                [(12.0, 5.0), (12.0, 19.0)],
            ],
            Self::Remove => vec![[(5.0, 12.0), (19.0, 12.0)]],
            Self::Close => vec![
                [(6.0, 6.0), (18.0, 18.0)],
                [(18.0, 6.0), (6.0, 18.0)],
            ],
            Self::Check => vec![
                [(4.5, 12.5), (9.5, 17.5)],
                [(9.5, 17.5), (19.5, 7.5)],
            ],
        }
    }
}

fn scaled(stroke: Stroke, factor: f32) -> Stroke {
    let scale = |point: Point2| (point.0 * factor, point.1 * factor);
    [scale(stroke[0]), scale(stroke[1])]
}

#[derive(IntoElement)]
pub struct Icon {
    kind: IconKind,
    size: f32,
    color: Option<Hsla>,
}

impl Icon {
    pub fn new(kind: IconKind) -> Self {
        Self {
            kind,
            size: GRID,
            color: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self
            .color
            .unwrap_or_else(|| cx.theme().color(ColorRole::OnSurface));
        let factor = self.size / GRID;
        let strokes: Vec<Stroke> = self
            .kind
            .strokes()
            .into_iter()
            .map(|stroke| scaled(stroke, factor))
            .collect();
        let width = STROKE * factor;
        div().size(px(self.size)).child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let origin = (f32::from(&bounds.origin.x), f32::from(&bounds.origin.y));
                    paint_strokes(window, origin, &strokes, width, color);
                },
            )
            .absolute()
            .size_full(),
        )
    }
}

pub fn icon_builder(
    kind: IconKind,
    size: f32,
) -> impl Fn(&mut Window, &mut App) -> AnyElement + 'static {
    move |_, _| Icon::new(kind).size(size).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_strokes_inside_the_grid() {
        for kind in [IconKind::Add, IconKind::Remove, IconKind::Close, IconKind::Check] {
            for stroke in kind.strokes() {
                for (x, y) in stroke {
                    assert!((0.0..=GRID).contains(&x) && (0.0..=GRID).contains(&y));
                }
            }
        }
    }

    #[test]
    fn add_is_two_strokes_and_remove_is_one() {
        assert_eq!(IconKind::Add.strokes().len(), 2);
        assert_eq!(IconKind::Remove.strokes().len(), 1);
    }
}
