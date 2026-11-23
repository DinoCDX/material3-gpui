use std::f32::consts::{PI, TAU};

use super::polygon::{Corner, rounded_outline};
use crate::systems::geometry::Point2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Preset {
    Circle,
    Square,
    Pill,
    Triangle,
    Diamond,
    Sunny,
    Cookie9,
    Clover4,
    Burst,
}

impl Preset {
    pub const ALL: [Preset; 9] = [
        Preset::Circle,
        Preset::Square,
        Preset::Pill,
        Preset::Triangle,
        Preset::Diamond,
        Preset::Sunny,
        Preset::Cookie9,
        Preset::Clover4,
        Preset::Burst,
    ];

    pub fn outline(self) -> Vec<Point2> {
        match self {
            Self::Circle => circle(64),
            Self::Square => rounded_outline(&[
                Corner::new(-1.0, -1.0, 0.6),
                Corner::new(1.0, -1.0, 0.6),
                Corner::new(1.0, 1.0, 0.6),
                Corner::new(-1.0, 1.0, 0.6),
            ]),
            Self::Pill => rounded_outline(&[
                Corner::new(-1.0, -0.5, 0.5),
                Corner::new(1.0, -0.5, 0.5),
                Corner::new(1.0, 0.5, 0.5),
                Corner::new(-1.0, 0.5, 0.5),
            ]),
            Self::Triangle => rounded_outline(&regular(3, 1.0, 0.2, -PI / 2.0)),
            Self::Diamond => rounded_outline(&regular(4, 1.0, 0.3, PI / 4.0 - PI / 2.0)),
            Self::Sunny => rounded_outline(&star(8, 1.0, 0.8, 0.15)),
            Self::Cookie9 => rounded_outline(&star(9, 1.0, 0.8, 0.5)),
            Self::Clover4 => clover(4, 128),
            Self::Burst => rounded_outline(&star(12, 1.0, 0.75, 0.05)),
        }
    }
}

fn circle(count: usize) -> Vec<Point2> {
    (0..count)
        .map(|index| {
            let angle = TAU * index as f32 / count as f32;
            (angle.cos(), angle.sin())
        })
        .collect()
}

fn regular(sides: usize, radius: f32, corner: f32, rotation: f32) -> Vec<Corner> {
    (0..sides)
        .map(|index| {
            let angle = rotation + TAU * index as f32 / sides as f32;
            Corner::new(radius * angle.cos(), radius * angle.sin(), corner)
        })
        .collect()
}

fn star(points: usize, outer: f32, inner_fraction: f32, corner: f32) -> Vec<Corner> {
    let vertices = points * 2;
    (0..vertices)
        .map(|index| {
            let radius = if index % 2 == 0 { outer } else { outer * inner_fraction };
            let angle = -PI / 2.0 + PI * index as f32 / points as f32;
            Corner::new(radius * angle.cos(), radius * angle.sin(), corner)
        })
        .collect()
}

fn clover(leaves: usize, count: usize) -> Vec<Point2> {
    (0..count)
        .map(|index| {
            let angle = TAU * index as f32 / count as f32;
            let radius = 0.55 + 0.45 * (leaves as f32 * angle / 2.0).cos().powi(2);
            (radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_has_an_outline() {
        for preset in Preset::ALL {
            assert!(preset.outline().len() > 8, "{preset:?}");
        }
    }

    #[test]
    fn circle_points_sit_on_the_unit_radius() {
        for (x, y) in Preset::Circle.outline() {
            assert!(((x * x + y * y).sqrt() - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn star_alternates_outer_and_inner_radius() {
        let corners = star(8, 1.0, 0.8, 0.0);
        let outer = (corners[0].at.0.powi(2) + corners[0].at.1.powi(2)).sqrt();
        let inner = (corners[1].at.0.powi(2) + corners[1].at.1.powi(2)).sqrt();
        assert!((outer - 1.0).abs() < 1e-5);
        assert!((inner - 0.8).abs() < 1e-5);
    }
}
