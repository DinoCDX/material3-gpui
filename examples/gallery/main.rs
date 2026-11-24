use gpui::prelude::*;
use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, Render, Window, div, px};
use material_color_utils::utils::color_utils::Argb;
use material3_gpui::prelude::*;

const SEEDS: [u32; 4] = [0xFF4285F4, 0xFF34A853, 0xFFEA4335, 0xFF9C27B0];

fn toggle_mode(cx: &mut App) {
    let next = match cx.theme().mode() {
        ThemeMode::Light => ThemeMode::Dark,
        ThemeMode::Dark => ThemeMode::Light,
    };
    MaterialTheme::set_mode(cx, next);
}

fn next_seed(cx: &mut App) {
    let current = cx.theme().seed().0;
    let index = SEEDS.iter().position(|seed| *seed == current).unwrap_or(0);
    let next = SEEDS[(index + 1) % SEEDS.len()];
    MaterialTheme::set_seed(cx, Argb(next));
}

fn glyph(text: &'static str) -> impl Fn(&mut Window, &mut App) -> AnyElement {
    move |_, _| div().child(text).into_any_element()
}

struct Gallery {
    mode_toggle: Entity<Button>,
    seed_toggle: Entity<Button>,
    buttons: Vec<Entity<Button>>,
    icon_buttons: Vec<Entity<IconButton>>,
    fabs: Vec<Entity<Fab>>,
    morph: Entity<MorphView>,
}

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        let mode_toggle = cx.new(|cx| {
            Button::new("mode", "Toggle light and dark", cx)
                .variant(ButtonVariant::FilledTonal)
                .on_click(|_, cx| toggle_mode(cx))
        });
        let seed_toggle = cx.new(|cx| {
            Button::new("seed", "Next seed colour", cx)
                .variant(ButtonVariant::Outlined)
                .on_click(|_, cx| next_seed(cx))
        });

        let variants = [
            ("filled", ButtonVariant::Filled),
            ("tonal", ButtonVariant::FilledTonal),
            ("elevated", ButtonVariant::Elevated),
            ("outlined", ButtonVariant::Outlined),
            ("text", ButtonVariant::Text),
        ];
        let mut buttons = Vec::new();
        for (id, variant) in variants {
            buttons.push(cx.new(|cx| {
                Button::new(id, "Label", cx)
                    .variant(variant)
                    .size(ButtonSize::Medium)
                    .leading_icon(glyph("+"))
            }));
        }
        buttons.push(cx.new(|cx| {
            Button::new("disabled", "Disabled", cx)
                .variant(ButtonVariant::Filled)
                .enabled(false)
        }));
        buttons.push(cx.new(|cx| {
            Button::new("square", "Square XL", cx)
                .variant(ButtonVariant::FilledTonal)
                .size(ButtonSize::XLarge)
                .shape(ButtonShape::Square)
        }));

        let icon_variants = [
            ("standard", IconButtonVariant::Standard),
            ("filled", IconButtonVariant::Filled),
            ("tonal", IconButtonVariant::FilledTonal),
            ("outlined", IconButtonVariant::Outlined),
        ];
        let mut icon_buttons = Vec::new();
        for (id, variant) in icon_variants {
            icon_buttons.push(cx.new(|cx| {
                IconButton::new(id, glyph("★"), cx)
                    .variant(variant)
                    .size(IconButtonSize::Medium)
                    .toggleable(true)
            }));
        }

        let fabs = vec![
            cx.new(|cx| Fab::new("fab-small", glyph("+"), cx).size(FabSize::Small)),
            cx.new(|cx| Fab::new("fab-standard", glyph("+"), cx)),
            cx.new(|cx| Fab::new("fab-large", glyph("+"), cx).size(FabSize::Large)),
            cx.new(|cx| {
                Fab::new("fab-extended", glyph("+"), cx)
                    .color(FabColor::Secondary)
                    .extended("Compose")
            }),
        ];

        let morph = cx.new(|_| MorphView::new(&Preset::ALL, 1.0, 0.8));
        Self {
            mode_toggle,
            seed_toggle,
            buttons,
            icon_buttons,
            fabs,
            morph,
        }
    }
}

fn section(title: &'static str, theme: &MaterialTheme, body: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .text_size(px(16.0))
                .text_color(theme.color(ColorRole::OnSurface))
                .child(title),
        )
        .child(body)
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let on_surface = theme.color(ColorRole::OnSurface);

        let colours = ColorRole::ALL.iter().fold(
            div().flex().flex_wrap().gap(px(6.0)),
            |row, role| {
                row.child(
                    div()
                        .w(px(120.0))
                        .h(px(40.0))
                        .bg(theme.color(*role))
                        .text_size(px(10.0))
                        .text_color(theme.color(ColorRole::OnSurface))
                        .child(format!("{role:?}")),
                )
            },
        );

        let type_scale = TypeStyle::ALL.iter().fold(div().flex().flex_col(), |column, style| {
            let family = theme.typography().font_family(style.font()).clone();
            column.child(
                div()
                    .font_family(family)
                    .font_weight(style.weight())
                    .text_size(px(style.size_sp()))
                    .text_color(on_surface)
                    .child(format!("{style:?}")),
            )
        });

        let shapes = ShapeToken::ALL.iter().fold(
            div().flex().flex_wrap().gap(px(8.0)),
            |row, token| {
                let radii = token.radii();
                row.child(
                    div()
                        .w(px(96.0))
                        .h(px(64.0))
                        .rounded_tl(px(radii.top_left.min(48.0)))
                        .rounded_tr(px(radii.top_right.min(48.0)))
                        .rounded_br(px(radii.bottom_right.min(48.0)))
                        .rounded_bl(px(radii.bottom_left.min(48.0)))
                        .bg(theme.color(ColorRole::PrimaryContainer))
                        .text_size(px(10.0))
                        .text_color(theme.color(ColorRole::OnPrimaryContainer))
                        .child(format!("{token:?}")),
                )
            },
        );

        let levels = [
            ElevationLevel::Level0,
            ElevationLevel::Level1,
            ElevationLevel::Level2,
            ElevationLevel::Level3,
            ElevationLevel::Level4,
            ElevationLevel::Level5,
        ];
        let elevations = levels.iter().enumerate().fold(
            div().flex().flex_wrap().gap(px(24.0)),
            |row, (index, level)| {
                row.child(
                    Surface::new(("elevation", index as u64))
                        .style(SurfaceStyle {
                            shape: ShapeToken::CornerMedium,
                            tonal_elevation: *level,
                            elevation: StateElevation::constant(*level),
                            outlined: false,
                        })
                        .child(
                            div()
                                .w(px(120.0))
                                .h(px(80.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(on_surface)
                                .child(format!("{level:?}")),
                        ),
                )
            },
        );

        let mut button_row = div().flex().flex_wrap().gap(px(12.0));
        for button in &self.buttons {
            button_row = button_row.child(button.clone());
        }
        let mut icon_row = div().flex().flex_wrap().gap(px(12.0));
        for icon in &self.icon_buttons {
            icon_row = icon_row.child(icon.clone());
        }
        let mut fab_row = div().flex().flex_wrap().items_center().gap(px(16.0));
        for fab in &self.fabs {
            fab_row = fab_row.child(fab.clone());
        }

        let badges = div()
            .flex()
            .items_center()
            .gap(px(16.0))
            .child(Badge::dot())
            .child(Badge::count("3"))
            .child(Badge::count("12"));

        let dividers = div()
            .flex()
            .gap(px(16.0))
            .h(px(40.0))
            .child(div().flex_1().child(HorizontalDivider::new()))
            .child(VerticalDivider::new());

        let content = div()
            .flex()
            .flex_col()
            .gap(px(28.0))
            .p(px(24.0))
            .bg(theme.color(ColorRole::Background))
            .text_color(on_surface)
            .child(
                div()
                    .flex()
                    .gap(px(12.0))
                    .child(self.mode_toggle.clone())
                    .child(self.seed_toggle.clone()),
            )
            .child(section("Colour roles", theme, colours))
            .child(section("Type scale", theme, type_scale))
            .child(section("Shapes", theme, shapes))
            .child(section("Elevation", theme, elevations))
            .child(section(
                "Shapes",
                theme,
                div().w(px(260.0)).h(px(260.0)).child(self.morph.clone()),
            ))
            .child(section("Buttons", theme, button_row))
            .child(section("Icon buttons", theme, icon_row))
            .child(section("FABs", theme, fab_row))
            .child(section(
                "Badges and dividers",
                theme,
                div().flex().flex_col().gap(px(12.0)).child(badges).child(dividers),
            ));
        div()
            .id("gallery-scroll")
            .size_full()
            .overflow_y_scroll()
            .child(content)
    }
}

fn main() {
    run_app(
        AppConfig::new(SEEDS[0]).size(1280.0, 1800.0).title("Material 3 gallery"),
        |_, cx| cx.new(Gallery::new),
    );
}
