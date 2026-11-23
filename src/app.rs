use gpui::{
    App, Application, Bounds, Entity, Render, Window, WindowBounds, WindowOptions, px, size,
};
use material_color_utils::utils::color_utils::Argb;

use crate::systems::theme::{MaterialTheme, ThemeMode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppConfig {
    pub seed: u32,
    pub mode: ThemeMode,
    pub width: f32,
    pub height: f32,
}

impl AppConfig {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            mode: ThemeMode::Light,
            width: 1280.0,
            height: 800.0,
        }
    }

    pub fn mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

pub fn run_app<V: Render + 'static>(
    config: AppConfig,
    root: impl FnOnce(&mut Window, &mut App) -> Entity<V> + 'static,
) {
    Application::new().run(move |cx: &mut App| {
        MaterialTheme::install(cx, Argb(config.seed), config.mode);
        let bounds = Bounds::centered(None, size(px(config.width), px(config.height)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            root,
        )
        .unwrap();
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_mode_and_size() {
        let config = AppConfig::new(0xFF4285F4)
            .mode(ThemeMode::Dark)
            .size(900.0, 700.0);
        assert_eq!(config.mode, ThemeMode::Dark);
        assert_eq!(config.width, 900.0);
        assert_eq!(config.height, 700.0);
    }
}
