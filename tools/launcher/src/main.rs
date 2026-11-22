use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    App, Application, Bounds, Context, Entity, IntoElement, Render, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use material_color_utils::utils::color_utils::Argb;
use material3_gpui::prelude::*;
use serde_json::{Value, json};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);
const SPEC_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/showcase.json");
const KOTLIN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../showcase-kotlin");

#[derive(Default, Clone)]
struct ChildStats {
    name: String,
    pid: u32,
    rss_kb: u64,
    cpu_percent: f32,
    startup_ms: f64,
    frame_avg_ms: f64,
    frame_max_ms: f64,
    frames_seen: u64,
}

#[derive(Default)]
struct Shared {
    stats: Vec<ChildStats>,
    stdins: Vec<(String, ChildStdin)>,
    processes: Vec<Child>,
}

type SharedState = Arc<Mutex<Shared>>;

fn sibling_binary(name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("launcher path");
    exe.parent().expect("target dir").join(name)
}

fn rust_command() -> Command {
    let mut command = Command::new(sibling_binary("showcase"));
    command.arg(SPEC_PATH);
    command
}

fn kotlin_command() -> Command {
    let script = PathBuf::from(KOTLIN_DIR)
        .join("build/install/showcase-kotlin/bin/showcase-kotlin");
    let mut command = Command::new(script);
    command.env("SHOWCASE_SPEC", SPEC_PATH);
    command
}

fn apply_event(state: &SharedState, name: &str, event: &Value) {
    let mut shared = state.lock().expect("state lock");
    let Some(stats) = shared.stats.iter_mut().find(|stats| stats.name == name) else {
        return;
    };
    match event["event"].as_str() {
        Some("ready") => stats.startup_ms = event["startup_ms"].as_f64().unwrap_or(0.0),
        Some("frames") => {
            stats.frame_avg_ms = event["avg_ms"].as_f64().unwrap_or(0.0);
            stats.frame_max_ms = event["max_ms"].as_f64().unwrap_or(0.0);
            stats.frames_seen += event["count"].as_u64().unwrap_or(0);
        }
        _ => {}
    }
}

fn spawn_child(state: &SharedState, name: &str, mut command: Command) {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            eprintln!("could not start {name}: {error}");
            return;
        }
    };
    let stdout = child.stdout.take().expect("piped stdout");
    let stdin = child.stdin.take().expect("piped stdin");
    {
        let mut shared = state.lock().expect("state lock");
        shared.stats.push(ChildStats {
            name: name.to_string(),
            pid: child.id(),
            ..Default::default()
        });
        shared.stdins.push((name.to_string(), stdin));
        shared.processes.push(child);
    }
    let reader_state = Arc::clone(state);
    let reader_name = name.to_string();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Ok(event) = serde_json::from_str::<Value>(&line) {
                apply_event(&reader_state, &reader_name, &event);
            }
        }
    });
}

fn sample_processes(state: SharedState) {
    thread::spawn(move || {
        loop {
            thread::sleep(SAMPLE_INTERVAL);
            let mut shared = state.lock().expect("state lock");
            for stats in shared.stats.iter_mut() {
                let output = Command::new("ps")
                    .args(["-o", "rss=,%cpu=", "-p", &stats.pid.to_string()])
                    .output();
                if let Ok(output) = output {
                    let text = String::from_utf8_lossy(&output.stdout);
                    let mut fields = text.split_whitespace();
                    stats.rss_kb = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                    stats.cpu_percent = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                }
            }
        }
    });
}

fn send_command(state: &SharedState, command: Value) {
    let mut shared = state.lock().expect("state lock");
    for (_, stdin) in shared.stdins.iter_mut() {
        let _ = writeln!(stdin, "{command}");
        let _ = stdin.flush();
    }
}

struct Launcher {
    state: SharedState,
    rust_button: Entity<Button>,
    kotlin_button: Entity<Button>,
    theme_button: Entity<Button>,
}

impl Launcher {
    fn new(cx: &mut Context<Self>, state: SharedState) -> Self {
        let rust_state = Arc::clone(&state);
        let rust_button = cx.new(|cx| {
            Button::new("open-rust", "Open Rust showcase", cx)
                .variant(ButtonVariant::Filled)
                .on_click(move |_, _| spawn_child(&rust_state, "rust", rust_command()))
        });
        let kotlin_state = Arc::clone(&state);
        let kotlin_button = cx.new(|cx| {
            Button::new("open-kotlin", "Open Kotlin showcase", cx)
                .variant(ButtonVariant::FilledTonal)
                .on_click(move |_, _| spawn_child(&kotlin_state, "kotlin", kotlin_command()))
        });
        let theme_state = Arc::clone(&state);
        let theme_button = cx.new(|cx| {
            Button::new("toggle-both", "Toggle theme on both", cx)
                .variant(ButtonVariant::Outlined)
                .on_click(move |_, cx| {
                    MaterialTheme::toggle_mode(cx);
                    send_command(&theme_state, json!({"cmd": "toggle_theme"}));
                })
        });
        Self {
            state,
            rust_button,
            kotlin_button,
            theme_button,
        }
    }
}

fn stat_line(stats: &ChildStats) -> String {
    format!(
        "{:<7} pid {:>6}  RSS {:>8.1} MB  CPU {:>5.1}%  startup {:>7.0} ms  frame avg {:>5.2} ms  max {:>6.2} ms  frames {}",
        stats.name,
        stats.pid,
        stats.rss_kb as f64 / 1024.0,
        stats.cpu_percent,
        stats.startup_ms,
        stats.frame_avg_ms,
        stats.frame_max_ms,
        stats.frames_seen,
    )
}

impl Render for Launcher {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();
        let theme = cx.theme();
        let lines: Vec<String> = {
            let shared = self.state.lock().expect("state lock");
            if shared.stats.is_empty() {
                vec!["No showcase running. Open one to see live stats.".to_string()]
            } else {
                shared.stats.iter().map(stat_line).collect()
            }
        };
        let mut stats_column = div().flex().flex_col().gap(px(6.0));
        for line in lines {
            stats_column = stats_column.child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.color(ColorRole::OnSurface))
                    .child(line),
            );
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(24.0))
            .p(px(24.0))
            .bg(theme.color(ColorRole::Background))
            .child(
                div()
                    .flex()
                    .gap(px(12.0))
                    .child(self.rust_button.clone())
                    .child(self.kotlin_button.clone())
                    .child(self.theme_button.clone()),
            )
            .child(stats_column)
    }
}

fn main() {
    let state: SharedState = Arc::new(Mutex::new(Shared::default()));
    sample_processes(Arc::clone(&state));
    Application::new().run(move |cx: &mut App| {
        MaterialTheme::install(cx, Argb(0xFF4285F4), ThemeMode::Light);
        let bounds = Bounds::centered(None, size(px(1100.0), px(520.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| cx.new(|cx| Launcher::new(cx, state)),
        )
        .unwrap();
        cx.activate(true);
    });
}
