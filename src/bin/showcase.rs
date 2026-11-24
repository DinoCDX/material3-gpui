use std::io::{BufRead, Write, stdin, stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, Render, Window, div, px};
use material3_gpui::prelude::*;
use serde_json::{Value, json};

const FRAMES_PER_REPORT: usize = 60;
const COMMAND_POLL_MS: u64 = 50;
const DEFAULT_SPEC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/spec/showcase.json");

enum Item {
    Text { style: TypeStyle, text: String },
    Swatch { role: ColorRole, name: String },
    Button(Entity<Button>),
    IconButton(Entity<IconButton>),
    Fab(Entity<Fab>),
    Divider,
    Badge(Option<String>),
    Checkbox(Entity<Checkbox>),
    Radio(Entity<RadioButton>),
    Switch(Entity<Switch>),
    Slider(Entity<Slider>),
    RangeSlider(Entity<RangeSlider>),
    Chip(Entity<Chip>),
    TextField(Entity<TextField>),
    Card(Entity<Card>),
    ListItem(ListSpec),
    Progress(ProgressSpec),
}

#[derive(Clone)]
struct ListSpec {
    headline: String,
    supporting: Option<String>,
    third: Option<String>,
    leading: Option<IconKind>,
    trailing: Option<String>,
}

#[derive(Clone, Copy)]
struct ProgressSpec {
    value: Option<f32>,
    circular: bool,
}

fn card_variant(name: &str) -> CardVariant {
    match name {
        "filled" => CardVariant::Filled,
        "outlined" => CardVariant::Outlined,
        _ => CardVariant::Elevated,
    }
}

fn build_list_item(spec: &ListSpec) -> ListItem {
    let mut row = ListItem::new(spec.headline.clone());
    if let Some(text) = &spec.supporting {
        row = row.supporting(text.clone());
    }
    if let Some(text) = &spec.third {
        row = row.third_line(text.clone());
    }
    if let Some(kind) = spec.leading {
        row = row.leading(icon_builder(kind, 24.0));
    }
    if let Some(text) = &spec.trailing {
        let text = text.clone();
        row = row.trailing(move |_, _| div().child(text.clone()).into_any_element());
    }
    row
}

fn build_progress(spec: ProgressSpec) -> AnyElement {
    match (spec.circular, spec.value) {
        (true, Some(value)) => CircularProgress::determinate(value).into_any_element(),
        (true, None) => CircularProgress::indeterminate().into_any_element(),
        (false, Some(value)) => LinearProgress::determinate(value).into_any_element(),
        (false, None) => LinearProgress::indeterminate().into_any_element(),
    }
}

struct Showcase {
    screens: Vec<(String, Vec<Item>)>,
    started: Instant,
    built_at: Instant,
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

fn number(item: &Value, key: &str, default: f32) -> f32 {
    item[key].as_f64().map_or(default, |value| value as f32)
}

fn flag(item: &Value, key: &str, default: bool) -> bool {
    item[key].as_bool().unwrap_or(default)
}

fn checkbox_state(name: &str) -> CheckboxState {
    match name {
        "checked" => CheckboxState::Checked,
        "indeterminate" => CheckboxState::Indeterminate,
        _ => CheckboxState::Unchecked,
    }
}

fn icon_kind(name: &str) -> Option<IconKind> {
    match name {
        "add" => Some(IconKind::Add),
        "remove" => Some(IconKind::Remove),
        "close" => Some(IconKind::Close),
        "check" => Some(IconKind::Check),
        _ => None,
    }
}

fn chip_kind(name: &str) -> ChipKind {
    match name {
        "filter" => ChipKind::Filter,
        "input" => ChipKind::Input,
        "suggestion" => ChipKind::Suggestion,
        _ => ChipKind::Assist,
    }
}

fn glyph() -> impl Fn(&mut Window, &mut App) -> AnyElement + 'static {
    |_, _| div().child("+").into_any_element()
}

impl Showcase {
    fn new(
        cx: &mut Context<Self>,
        spec: &Value,
        toggle: Arc<AtomicBool>,
        started: Instant,
    ) -> Self {
        let mut screens = Vec::new();
        for (screen_index, screen) in spec["screens"].as_array().into_iter().flatten().enumerate() {
            let mut items = Vec::new();
            for (index, item) in screen["items"].as_array().into_iter().flatten().enumerate() {
                let id = ("showcase-item", (screen_index * 1000 + index) as u64);
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
                    "checkbox" => {
                        let state = checkbox_state(text_field(item, "state"));
                        let enabled = flag(item, "enabled", true);
                        let error = flag(item, "error", false);
                        Item::Checkbox(cx.new(|cx| {
                            Checkbox::new(id, cx).state(state).enabled(enabled).error(error)
                        }))
                    }
                    "radio" => {
                        let selected = flag(item, "selected", false);
                        let enabled = flag(item, "enabled", true);
                        Item::Radio(cx.new(|cx| {
                            RadioButton::new(id, cx).selected(selected).enabled(enabled)
                        }))
                    }
                    "switch" => {
                        let selected = flag(item, "selected", false);
                        let enabled = flag(item, "enabled", true);
                        let icon = flag(item, "icon", false);
                        Item::Switch(cx.new(|cx| {
                            Switch::new(id, cx)
                                .selected(selected)
                                .enabled(enabled)
                                .show_icon(icon)
                        }))
                    }
                    "slider" => {
                        let min = number(item, "min", 0.0);
                        let max = number(item, "max", 100.0);
                        let value = number(item, "value", min);
                        let steps = number(item, "steps", 0.0) as usize;
                        let width = number(item, "width", 240.0);
                        let enabled = flag(item, "enabled", true);
                        Item::Slider(cx.new(|cx| {
                            Slider::new(id, min, max, cx)
                                .value(value)
                                .steps(steps)
                                .width(width)
                                .enabled(enabled)
                        }))
                    }
                    "range_slider" => {
                        let min = number(item, "min", 0.0);
                        let max = number(item, "max", 100.0);
                        let low = number(item, "low", min);
                        let high = number(item, "high", max);
                        let steps = number(item, "steps", 0.0) as usize;
                        let width = number(item, "width", 240.0);
                        let enabled = flag(item, "enabled", true);
                        Item::RangeSlider(cx.new(|cx| {
                            RangeSlider::new(id, min, max, cx)
                                .values(low, high)
                                .steps(steps)
                                .width(width)
                                .enabled(enabled)
                        }))
                    }
                    "chip" => {
                        let kind = chip_kind(text_field(item, "variant"));
                        let label = text_field(item, "label").to_string();
                        let selected = flag(item, "selected", false);
                        let elevated = flag(item, "elevated", false);
                        let leading = item["leading"].as_str().map(str::to_string);
                        let trailing = item["trailing"].as_str().map(str::to_string);
                        Item::Chip(cx.new(|cx| {
                            let chip = Chip::new(id, kind, label, cx)
                                .selected(selected)
                                .elevated(elevated);
                            let chip = match leading {
                                Some(glyph) => chip.leading_icon(glyph),
                                None => chip,
                            };
                            match trailing {
                                Some(glyph) => chip.trailing_icon(glyph),
                                None => chip,
                            }
                        }))
                    }
                    "text_field" => {
                        let label = text_field(item, "label").to_string();
                        let value = text_field(item, "value").to_string();
                        let supporting = item["supporting"].as_str().map(str::to_string);
                        let error = item["error"].as_str().map(str::to_string);
                        let leading = icon_kind(text_field(item, "leading"));
                        let trailing = icon_kind(text_field(item, "trailing"));
                        let filled = flag(item, "filled", true);
                        let max = item["max"].as_u64().map(|max| max as usize);
                        Item::TextField(cx.new(|cx| {
                            let mut field = TextField::new(id, label, cx)
                                .value(value)
                                .filled(filled);
                            if let Some(max) = max {
                                field = field.max_chars(max);
                            }
                            if let Some(text) = supporting {
                                field = field.supporting(text);
                            }
                            if let Some(text) = error {
                                field = field.error(text);
                            }
                            if let Some(kind) = leading {
                                field = field.leading_icon(kind);
                            }
                            if let Some(kind) = trailing {
                                field = field.trailing_icon(kind);
                            }
                            field
                        }))
                    }
                    "card" => {
                        let variant = card_variant(text_field(item, "variant"));
                        let clickable = flag(item, "clickable", false);
                        let body = text_field(item, "text").to_string();
                        Item::Card(cx.new(|cx| {
                            Card::new(id, variant, cx)
                                .clickable(clickable)
                                .content(move |_, _| div().child(body.clone()).into_any_element())
                        }))
                    }
                    "list_item" => Item::ListItem(ListSpec {
                        headline: text_field(item, "headline").to_string(),
                        supporting: item["supporting"].as_str().map(str::to_string),
                        third: item["third"].as_str().map(str::to_string),
                        leading: icon_kind(text_field(item, "leading")),
                        trailing: item["trailing"].as_str().map(str::to_string),
                    }),
                    "linear_progress" | "circular_progress" => Item::Progress(ProgressSpec {
                        value: if flag(item, "indeterminate", false) {
                            None
                        } else {
                            Some(number(item, "value", 0.0))
                        },
                        circular: text_field(item, "kind") == "circular_progress",
                    }),
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
        let flag = Arc::clone(&toggle);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(COMMAND_POLL_MS))
                    .await;
                let alive = this.update(cx, |_, cx| {
                    if flag.swap(false, Ordering::SeqCst) {
                        MaterialTheme::toggle_mode(cx);
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            screens,
            started,
            built_at: Instant::now(),
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
            let build = (self.built_at - self.started).as_secs_f64() * 1000.0;
            report(json!({"event": "ready", "startup_ms": startup, "build_ms": build}));
        }
        if let Some(last) = self.last_frame {
            self.frame_times.push((now - last).as_secs_f64() * 1000.0);
            if self.frame_times.len() >= FRAMES_PER_REPORT {
                let count = self.frame_times.len();
                let total: f64 = self.frame_times.iter().sum();
                let mut sorted = self.frame_times.clone();
                sorted.sort_by(f64::total_cmp);
                let p95 = sorted[((count - 1) as f64 * 0.95) as usize];
                report(json!({
                    "event": "frames",
                    "count": count,
                    "avg_ms": total / count as f64,
                    "min_ms": sorted[0],
                    "p95_ms": p95,
                    "max_ms": sorted[count - 1],
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
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.color(*role))
            .text_size(px(11.0))
            .text_color(theme.color(ColorRole::OnSurface))
            .child(name.clone())
            .into_any_element(),
        Item::Button(button) => button.clone().into_any_element(),
        Item::IconButton(button) => button.clone().into_any_element(),
        Item::Fab(fab) => fab.clone().into_any_element(),
        Item::Checkbox(view) => view.clone().into_any_element(),
        Item::Radio(view) => view.clone().into_any_element(),
        Item::Switch(view) => view.clone().into_any_element(),
        Item::Slider(view) => view.clone().into_any_element(),
        Item::RangeSlider(view) => view.clone().into_any_element(),
        Item::Chip(view) => view.clone().into_any_element(),
        Item::TextField(view) => view.clone().into_any_element(),
        Item::Card(view) => view.clone().into_any_element(),
        Item::ListItem(spec) => build_list_item(spec).into_any_element(),
        Item::Progress(spec) => build_progress(*spec),
        Item::Divider => div()
            .w(px(160.0))
            .child(HorizontalDivider::new())
            .into_any_element(),
        Item::Badge(None) => Badge::dot().into_any_element(),
        Item::Badge(Some(text)) => Badge::count(text.clone()).into_any_element(),
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.record_frame();

        let theme = cx.theme();
        let mut column = div()
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
        div()
            .id("showcase-scroll")
            .size_full()
            .overflow_y_scroll()
            .child(column)
    }
}

fn parse_seed(hex: &str) -> u32 {
    0xFF00_0000 | u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0x4285F4)
}

fn spawn_command_reader(toggle: Arc<AtomicBool>) {
    thread::spawn(move || {
        for line in stdin().lock().lines().map_while(Result::ok) {
            if let Ok(command) = serde_json::from_str::<Value>(&line)
                && command["cmd"] == "toggle_theme"
            {
                toggle.store(true, Ordering::SeqCst);
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

    let started = Instant::now();
    run_app(
        AppConfig::new(seed)
            .size(1280.0, 2000.0)
            .title("Material 3 showcase"),
        move |_, cx| cx.new(|cx| Showcase::new(cx, &spec, toggle, started)),
    );
}
