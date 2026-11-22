pub mod contrast;
pub mod scheme;
pub mod typography;

use gpui::{App, BorrowAppContext, Global, Hsla};
use material_color_utils::{theme_from_color, utils::color_utils::Argb};

use crate::systems::interaction::config::FocusIndication;
use crate::tokens::color::ColorRole;

use scheme::ColorScheme;
use typography::Typography;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub struct MaterialTheme {
    seed: Argb,
    mode: ThemeMode,
    light: ColorScheme,
    dark: ColorScheme,
    typography: Typography,
    ripple_focus: FocusIndication,
}

impl Global for MaterialTheme {}

impl MaterialTheme {
    pub fn from_seed(seed: Argb, mode: ThemeMode) -> Self {
        let generated = theme_from_color(seed).call();
        Self {
            seed,
            mode,
            light: ColorScheme::from_materialized(&generated.schemes.light),
            dark: ColorScheme::from_materialized(&generated.schemes.dark),
            typography: Typography::default(),
            ripple_focus: FocusIndication::default_opacity(),
        }
    }

    pub fn install(cx: &mut App, seed: Argb, mode: ThemeMode) {
        cx.set_global(Self::from_seed(seed, mode));
    }

    pub fn seed(&self) -> Argb {
        self.seed
    }

    pub fn mode(&self) -> ThemeMode {
        self.mode
    }

    pub fn scheme(&self) -> &ColorScheme {
        match self.mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    pub fn color(&self, role: ColorRole) -> Hsla {
        self.scheme().get(role)
    }

    pub fn typography(&self) -> &Typography {
        &self.typography
    }

    pub fn ripple_focus(&self) -> FocusIndication {
        self.ripple_focus
    }

    pub fn set_ripple_focus(cx: &mut App, indication: FocusIndication) {
        cx.update_global::<Self, _>(|theme, _| theme.ripple_focus = indication);
        cx.refresh_windows();
    }

    pub fn toggle_mode(cx: &mut App) {
        let next = match cx.global::<Self>().mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        Self::set_mode(cx, next);
    }

    pub fn set_mode(cx: &mut App, mode: ThemeMode) {
        cx.update_global::<Self, _>(|theme, _| theme.mode = mode);
        cx.refresh_windows();
    }

    pub fn set_seed(cx: &mut App, seed: Argb) {
        let mode = cx.global::<Self>().mode;
        cx.set_global(Self::from_seed(seed, mode));
        cx.refresh_windows();
    }
}

pub trait ActiveTheme {
    fn theme(&self) -> &MaterialTheme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &MaterialTheme {
        self.global::<MaterialTheme>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    const SEED: Argb = Argb(0xFF4285F4);

    #[test]
    fn light_and_dark_surfaces_differ_in_lightness() {
        let theme = MaterialTheme::from_seed(SEED, ThemeMode::Light);
        let light = theme.light.get(ColorRole::Surface);
        let dark = theme.dark.get(ColorRole::Surface);
        assert!(light.l > dark.l);
    }

    #[test]
    fn active_scheme_follows_mode() {
        let mut theme = MaterialTheme::from_seed(SEED, ThemeMode::Light);
        assert!(std::ptr::eq(theme.scheme(), &theme.light));
        theme.mode = ThemeMode::Dark;
        assert!(std::ptr::eq(theme.scheme(), &theme.dark));
    }

    #[test]
    fn seed_is_retained() {
        let theme = MaterialTheme::from_seed(SEED, ThemeMode::Dark);
        assert_eq!(theme.seed().0, SEED.0);
        assert_eq!(theme.mode(), ThemeMode::Dark);
    }

    #[gpui::test]
    fn set_mode_updates_active_mode(cx: &mut TestAppContext) {
        cx.update(|cx| MaterialTheme::install(cx, SEED, ThemeMode::Light));
        cx.update(|cx| MaterialTheme::set_mode(cx, ThemeMode::Dark));
        cx.update(|cx| assert_eq!(cx.theme().mode(), ThemeMode::Dark));
    }

    #[gpui::test]
    fn set_seed_keeps_active_mode(cx: &mut TestAppContext) {
        cx.update(|cx| MaterialTheme::install(cx, SEED, ThemeMode::Dark));
        cx.update(|cx| MaterialTheme::set_seed(cx, Argb(0xFF34A853)));
        cx.update(|cx| {
            assert_eq!(cx.theme().mode(), ThemeMode::Dark);
            assert_eq!(cx.theme().seed().0, 0xFF34A853);
        });
    }
}
