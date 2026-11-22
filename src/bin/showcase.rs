use std::io::{BufRead, Write, stdin, stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Application, Bounds, Context, Entity, IntoElement, Render, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use material_color_utils::utils::color_utils::Argb;
use material3_gpui::prelude::*;
use serde_json::{Value, json};

const FRAMES_PER_REPORT: usize = 60;
const DEFAULT_SPEC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/spec/showcase.json");

enum Item {
    Text { style: TypeStyle, text: String },
    Swatch { role: ColorRole, name: String },
    Button(Entity<Button>),
    IconButton(Entity<IconButton>),
    Fab(Entity<Fab>),
    Divider,
    Badge(Option<String>),
}

struct Showcase {
    screens: Vec<(String, Vec<Item>)>,
    toggle: Arc<AtomicBool>,
    started: Instant,
    last_frame: Option<Instant>,
    frame_times: Vec<f64>,
    reported_ready: bool,
}

fn report(event: Value) {
    println!("{event}");
    let _ = stdout().flush();
}

fn text_field<'a>(item: &'a Value, key: &str) -> &'a str {
    item[key].as_str().unwrap_or("")
}

fn button_variant(name: &str) -> ButtonVariant {
    match name {
        "tonal" => ButtonVariant::FilledTonal,
        "elevated" => ButtonVariant::Elevated,
        "outlined" => ButtonVariant::Outlined,
        "text" => ButtonVariant::Text,
        _ => ButtonVariant::Filled,
    }
}

fn button_size(name: &str) -> ButtonSize {
    match name {
        "xsmall" => ButtonSize::XSmall,
        "medium" => ButtonSize::Medium,
        "large" => ButtonSize::Large,
        "xlarge" => ButtonSize::XLarge,
        _ => ButtonSize::Small,
    }
}

fn icon_variant(name: &str) -> IconButtonVariant {
    match name {
        "filled" => IconButtonVariant::Filled,
        "tonal" => IconButtonVariant::FilledTonal,
        "outlined" => IconButtonVariant::Outlined,
        _ => IconButtonVariant::Standard,
    }
}

fn fab_size(name: &str) -> FabSize {
    match name {
        "small" => FabSize::Small,
        "large" => FabSize::Large,
        _ => FabSize::Standard,
    }
}

fn fab_color(name: &str) -> FabColor {
    match name {
        "secondary" => FabColor::Secondary,
        _ => FabColor::Primary,
    }
}

fn find_style(name: &str) -> TypeStyle {
    TypeStyle::ALL
        .iter()
        .copied()
        .find(|style| format!("{style:?}").eq_ignore_ascii_case(name))
        .unwrap_or(TypeStyle::BodyLarge)
}

fn find_role(name: &str) -> ColorRole {
    ColorRole::ALL
        .iter()
        .copied()
        .find(|role| format!("{role:?}").eq_ignore_ascii_case(name))
        .unwrap_or(ColorRole::OnSurface)
}

fn glyph() -> impl Fn(&mut Window, &mut App) -> AnyElement + 'static {
    |_, _| div().child("+").into_any_element()
}

impl Showcase {
    fn new(cx: &mut Context<Self>, spec: &Value, toggle: Arc<AtomicBool>) -> Self {
        let mut screens = Vec::new();
        for (screen_index, screen) in spec["screens"].as_array().into_iter().flatten().enumerate() {
            let mut items = Vec::new();
            for (index, item) in screen["items"].as_array().into_iter().flatten().enumerate() {
                let id = format!("{screen_index}-{index}");
                let built = match text_field(item, "kind") {
                    "text" => Item::Text {
                        style: find_style(text_field(item, "style")),
                        text: text_field(item, "text").to_string(),
                    },
                    "swatch" => Item::Swatch {
                        role: find_role(text_field(item, "role")),
                        name: text_field(item, "name").to_string(),
                    },
                    "button" => {
                        let label = text_field(item, "label").to_string();
                        let variant = button_variant(text_field(item, "variant"));
                        let size = button_size(text_field(item, "size"));
                        let enabled = item["enabled"].as_bool().unwrap_or(true);
                        let leading = item["leading"].as_bool().unwrap_or(false);
                        Item::Button(cx.new(|cx| {
                            let button = Button::new(id, label, cx)
                                .variant(variant)
                                .size(size)
                                .enabled(enabled);
                            if leading { button.leading_icon(glyph()) } else { button }
                        }))
                    }
                    "icon_button" => {
                        let variant = icon_variant(text_field(item, "variant"));
                        let size = IconButtonSize::Small;
                        Item::IconButton(cx.new(|cx| {
                            IconButton::new(id, glyph(), cx).variant(variant).size(size)
                        }))
                    }
                    "fab" => {
                        let size = fab_size(text_field(item, "size"));
                        let color = fab_color(text_field(item, "color"));
                        let extended = item["extended"].as_str().map(str::to_string);
                        Item::Fab(cx.new(|cx| {
                            let fab = Fab::new(id, glyph(), cx).size(size).color(color);
                            match extended {
                                Some(label) => fab.extended(label),
                                None => fab,
                            }
                        }))
                    }
                    "divider" => Item::Divider,
                    "badge" => match text_field(item, "content") {
                        "dot" => Item::Badge(None),
                        count => Item::Badge(Some(count.to_string())),
                    },
                    _ => continue,
                };
                items.push(built);
            }
            screens.push((text_field(screen, "title").to_string(), items));
        }
        Self {
            screens,
            toggle,
            started: Instant::now(),
            last_frame: None,
            frame_times: Vec::new(),
            reported_ready: false,
        }
    }

    fn record_frame(&mut self) {
        let now = Instant::now();
        if !self.reported_ready {
            self.reported_ready = true;
            let startup = (now - self.started).as_secs_f64() * 1000.0;
            report(json!({"event": "ready", "startup_ms": startup}));
        }
        if let Some(last) = self.last_frame {
            self.frame_times.push((now - last).as_secs_f64() * 1000.0);
            if self.frame_times.len() >= FRAMES_PER_REPORT {
                let count = self.frame_times.len();
                let total: f64 = self.frame_times.iter().sum();
                let max = self.frame_times.iter().copied().fold(0.0, f64::max);
                report(json!({
                    "event": "frames",
                    "count": count,
                    "avg_ms": total / count as f64,
                    "max_ms": max,
                }));
                self.frame_times.clear();
            }
        }
        self.last_frame = Some(now);
    }
}

fn render_item(item: &Item, theme: &MaterialTheme) -> AnyElement {
    match item {
        Item::Text { style, text } => div()
            .font_family(theme.typography().font_family(style.font()).clone())
            .font_weight(style.weight())
            .text_size(px(style.size_sp()))
            .child(text.clone())
            .into_any_element(),
        Item::Swatch { role, name } => div()
            .w(px(140.0))
            .h(px(44.0))
            .bg(theme.color(*role))
            .text_size(px(11.0))
            .text_color(theme.color(ColorRole::OnSurface))
            .child(name.clone())
            .into_any_element(),
        Item::Button(button) => button.clone().into_any_element(),
        Item::IconButton(button) => button.clone().into_any_element(),
        Item::Fab(fab) => fab.clone().into_any_element(),
        Item::Divider => div()
            .w(px(160.0))
            .child(HorizontalDivider::new())
            .into_any_element(),
        Item::Badge(None) => Badge::dot().into_any_element(),
        Item::Badge(Some(text)) => Badge::count(text.clone()).into_any_element(),
    }
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.record_frame();
        if self.toggle.swap(false, Ordering::SeqCst) {
            MaterialTheme::toggle_mode(cx);
        }
        window.request_animation_frame();

        let theme = cx.theme();
        let mut column = div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(28.0))
            .p(px(24.0))
            .bg(theme.color(ColorRole::Background))
            .text_color(theme.color(ColorRole::OnBackground));
        for (title, items) in &self.screens {
            let mut row = div().flex().flex_wrap().items_center().gap(px(12.0));
            for item in items {
                row = row.child(render_item(item, theme));
            }
            column = column.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .child(div().text_size(px(16.0)).child(title.clone()))
                    .child(row),
            );
        }
        column
    }
}

fn parse_seed(hex: &str) -> u32 {
    0xFF00_0000 | u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0x4285F4)
}

fn spawn_command_reader(toggle: Arc<AtomicBool>) {
    thread::spawn(move || {
        for line in stdin().lock().lines().map_while(Result::ok) {
            if let Ok(command) = serde_json::from_str::<Value>(&line) {
                if command["cmd"] == "toggle_theme" {
                    toggle.store(true, Ordering::SeqCst);
                }
            }
        }
    });
}

fn main() {
    let spec_path = std::env::args().nth(1).unwrap_or_else(|| DEFAULT_SPEC.to_string());
    let spec_text = std::fs::read_to_string(&spec_path).expect("read showcase spec");
    let spec: Value = serde_json::from_str(&spec_text).expect("parse showcase spec");
    let seed = parse_seed(spec["seed"].as_str().unwrap_or("#4285F4"));
    let toggle = Arc::new(AtomicBool::new(false));
    spawn_command_reader(Arc::clone(&toggle));

    Application::new().run(move |cx: &mut App| {
        MaterialTheme::install(cx, Argb(seed), ThemeMode::Light);
        let bounds = Bounds::centered(None, size(px(1280.0), px(2000.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| cx.new(|cx| Showcase::new(cx, &spec, toggle)),
        )
        .unwrap();
        cx.activate(true);
    });
}
