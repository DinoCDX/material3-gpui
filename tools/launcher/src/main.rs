use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Application, Bounds, BorderStyle, Context, Corners, Edges, Entity, Hsla,
    IntoElement, Render, Window, WindowBounds, WindowOptions, canvas, div, point, px, quad, size,
};
use gpui::AppContext;
use gpui::{SharedString, TitlebarOptions};
use material_color_utils::utils::color_utils::Argb;
use material3_gpui::prelude::*;
use serde_json::{Value, json};

#[cfg(unix)]
const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);
const HISTORY_LEN: usize = 120;
const CHART_WIDTH: f32 = 300.0;
const CHART_HEIGHT: f32 = 90.0;
const FRAME_BUDGET_MS: f32 = 16.67;
const FRAME_CHART_FLOOR_MS: f32 = 40.0;
const CPU_CHART_FLOOR: f32 = 100.0;
const SPEC_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/showcase.json");
const KOTLIN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../showcase-kotlin");

#[derive(Default, Clone, Copy)]
struct Sample {
    rss_mb: f32,
    cpu: f32,
    frame_avg_ms: f32,
}

#[derive(Default, Clone)]
struct ChildStats {
    name: String,
    pid: u32,
    rss_kb: u64,
    cpu_percent: f32,
    startup_ms: f64,
    build_ms: f64,
    exited: bool,
    frame_avg_ms: f64,
    frame_min_ms: f64,
    frame_p95_ms: f64,
    frame_max_ms: f64,
    frames_seen: u64,
    last_sample: Option<(f64, Instant)>,
    history: VecDeque<Sample>,
    peak_rss_mb: f32,
}

#[derive(Default)]
struct Shared {
    stats: Vec<ChildStats>,
    stdins: Vec<(String, ChildStdin)>,
    processes: Vec<(String, Child)>,
}

type SharedState = Arc<Mutex<Shared>>;

fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn sibling_binary(name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("launcher path");
    exe.parent()
        .expect("target dir")
        .join(executable_name(name))
}

fn rust_command() -> Command {
    let mut command = Command::new(sibling_binary("showcase"));
    command.arg(SPEC_PATH);
    command
}

fn java_executable() -> PathBuf {
    let java = executable_name("java");
    std::env::var("JAVA_HOME")
        .map(|home| PathBuf::from(home).join("bin").join(&java))
        .unwrap_or_else(|_| PathBuf::from(java))
}

fn kotlin_command() -> Command {
    let lib_dir = PathBuf::from(KOTLIN_DIR).join("build/install/showcase-kotlin/lib");
    let classpath = format!("{}{}*", lib_dir.display(), std::path::MAIN_SEPARATOR);
    let mut command = Command::new(java_executable());
    command
        .arg("-cp")
        .arg(classpath)
        .arg("MainKt")
        .env("SHOWCASE_SPEC", SPEC_PATH);
    command
}

fn number(event: &Value, key: &str) -> f64 {
    event[key].as_f64().unwrap_or(0.0)
}

fn apply_event(state: &SharedState, name: &str, event: &Value) {
    let mut shared = state.lock().expect("state lock");
    let Some(stats) = shared.stats.iter_mut().find(|stats| stats.name == name) else {
        return;
    };
    match event["event"].as_str() {
        Some("ready") => {
            stats.startup_ms = number(event, "startup_ms");
            stats.build_ms = number(event, "build_ms");
        }
        Some("frames") => {
            stats.frame_avg_ms = number(event, "avg_ms");
            stats.frame_min_ms = number(event, "min_ms");
            stats.frame_p95_ms = number(event, "p95_ms");
            stats.frame_max_ms = number(event, "max_ms");
            stats.frames_seen += number(event, "count") as u64;
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
        shared.processes.push((name.to_string(), child));
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

#[cfg(unix)]
fn read_process(pid: u32, _stats: &mut ChildStats) -> Option<(u64, f32)> {
    let output = Command::new("ps")
        .args(["-o", "rss=,%cpu=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut fields = text.split_whitespace();
    let rss = fields.next()?.parse().ok()?;
    let cpu = fields.next()?.parse().ok()?;
    Some((rss, cpu))
}

#[cfg(windows)]
mod win {
    use std::mem::{size_of, zeroed};

    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME};
    use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

    fn ticks(time: FILETIME) -> u64 {
        (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
    }

    pub fn sample(pid: u32) -> Option<(u64, f64)> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
            if handle.is_null() {
                return None;
            }
            let mut counters: PROCESS_MEMORY_COUNTERS = zeroed();
            let cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            counters.cb = cb;
            let memory_ok = GetProcessMemoryInfo(handle, &mut counters, cb) != 0;
            let mut created: FILETIME = zeroed();
            let mut exited: FILETIME = zeroed();
            let mut kernel: FILETIME = zeroed();
            let mut user: FILETIME = zeroed();
            let times_ok =
                GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) != 0;
            CloseHandle(handle);
            if !memory_ok || !times_ok {
                return None;
            }
            let rss_kb = counters.WorkingSetSize as u64 / 1024;
            let cpu_seconds = (ticks(kernel) + ticks(user)) as f64 / 10_000_000.0;
            Some((rss_kb, cpu_seconds))
        }
    }
}

#[cfg(windows)]
fn read_process(pid: u32, stats: &mut ChildStats) -> Option<(u64, f32)> {
    let (rss_kb, cpu_seconds) = win::sample(pid)?;
    let now = Instant::now();
    let cores = std::thread::available_parallelism().map_or(1.0, |n| n.get() as f64);
    let percent = match stats.last_sample {
        Some((last_seconds, last_time)) => {
            let wall = (now - last_time).as_secs_f64();
            if wall > 0.0 {
                ((cpu_seconds - last_seconds) / wall / cores * 100.0) as f32
            } else {
                stats.cpu_percent
            }
        }
        None => 0.0,
    };
    stats.last_sample = Some((cpu_seconds, now));
    Some((rss_kb, percent))
}

fn sample_processes(state: SharedState) {
    thread::spawn(move || {
        loop {
            thread::sleep(SAMPLE_INTERVAL);
            let mut shared = state.lock().expect("state lock");
            let Shared {
                stats, processes, ..
            } = &mut *shared;
            for (name, child) in processes.iter_mut() {
                if let Ok(Some(_)) = child.try_wait()
                    && let Some(entry) = stats.iter_mut().find(|entry| &entry.name == name)
                {
                    entry.exited = true;
                }
            }
            for entry in stats.iter_mut().filter(|entry| !entry.exited) {
                let pid = entry.pid;
                let Some((rss, cpu)) = read_process(pid, entry) else {
                    continue;
                };
                entry.rss_kb = rss;
                entry.cpu_percent = cpu;
                let sample = Sample {
                    rss_mb: rss as f32 / 1024.0,
                    cpu,
                    frame_avg_ms: entry.frame_avg_ms as f32,
                };
                entry.peak_rss_mb = entry.peak_rss_mb.max(sample.rss_mb);
                entry.history.push_back(sample);
                if entry.history.len() > HISTORY_LEN {
                    entry.history.pop_front();
                }
            }
        }
    });
}

fn stop_all(state: &SharedState) {
    let mut shared = state.lock().expect("state lock");
    for (_, child) in shared.processes.iter_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
    shared.processes.clear();
    shared.stdins.clear();
    shared.stats.clear();
}

fn send_command(state: &SharedState, command: Value) {
    let mut shared = state.lock().expect("state lock");
    for (_, stdin) in shared.stdins.iter_mut() {
        let _ = writeln!(stdin, "{command}");
        let _ = stdin.flush();
    }
}

#[derive(Clone, Copy)]
struct ChartStyle {
    accent: Hsla,
    surface: Hsla,
    text: Hsla,
}

fn chart(
    values: Vec<f32>,
    max: f32,
    color: Hsla,
    background: Hsla,
    reference: Option<f32>,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let origin_x = f32::from(&bounds.origin.x);
            let origin_y = f32::from(&bounds.origin.y);
            let width = f32::from(&bounds.size.width);
            let height = f32::from(&bounds.size.height);
            window.paint_quad(quad(
                bounds,
                Corners::all(px(0.0)),
                background,
                Edges::all(px(0.0)),
                background,
                BorderStyle::default(),
            ));
            if max <= 0.0 {
                return;
            }
            let slot = width / HISTORY_LEN as f32;
            let bar_width = (slot - 1.0).max(1.0);
            for (index, value) in values.iter().enumerate() {
                let fraction = (value / max).clamp(0.0, 1.0);
                let bar_height = fraction * height;
                let x = origin_x + width - (values.len() - index) as f32 * slot;
                let bar = Bounds::new(
                    point(px(x), px(origin_y + height - bar_height)),
                    size(px(bar_width), px(bar_height)),
                );
                window.paint_quad(quad(
                    bar,
                    Corners::all(px(0.0)),
                    color,
                    Edges::all(px(0.0)),
                    color,
                    BorderStyle::default(),
                ));
            }
            if let Some(reference) = reference {
                let fraction = (reference / max).clamp(0.0, 1.0);
                let line_y = origin_y + height - fraction * height;
                let line = Bounds::new(
                    point(px(origin_x), px(line_y)),
                    size(px(width), px(1.0)),
                );
                window.paint_quad(quad(
                    line,
                    Corners::all(px(0.0)),
                    color,
                    Edges::all(px(0.0)),
                    color,
                    BorderStyle::default(),
                ));
            }
        },
    )
    .w(px(CHART_WIDTH))
    .h(px(CHART_HEIGHT))
}

fn labelled_chart(
    label: &str,
    current: String,
    values: Vec<f32>,
    max: f32,
    style: ChartStyle,
    reference: Option<f32>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(px(12.0))
                .text_color(style.text)
                .child(label.to_string())
                .child(current),
        )
        .child(chart(values, max, style.accent, style.surface, reference))
}

fn average(values: impl Iterator<Item = f32>, count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    values.sum::<f32>() / count as f32
}

fn child_panel(stats: &ChildStats, style: ChartStyle) -> impl IntoElement {
    let history: Vec<Sample> = stats.history.iter().copied().collect();
    let count = history.len();
    let rss_now = stats.rss_kb as f32 / 1024.0;
    let rss_avg = average(history.iter().map(|s| s.rss_mb), count);
    let cpu_avg = average(history.iter().map(|s| s.cpu), count);
    let rss_values: Vec<f32> = history.iter().map(|s| s.rss_mb).collect();
    let cpu_values: Vec<f32> = history.iter().map(|s| s.cpu).collect();
    let frame_values: Vec<f32> = history.iter().map(|s| s.frame_avg_ms).collect();
    let rss_max = stats.peak_rss_mb.max(1.0) * 1.1;
    let cpu_max = cpu_values.iter().copied().fold(CPU_CHART_FLOOR, f32::max);
    let frame_max = frame_values.iter().copied().fold(FRAME_CHART_FLOOR_MS, f32::max);

    let state = if stats.exited { " · exited" } else { "" };
    let heading = format!(
        "{} · pid {} · startup {:.0} ms (build {:.0} ms) · {} frames{}",
        stats.name, stats.pid, stats.startup_ms, stats.build_ms, stats.frames_seen, state
    );
    let frame_line = format!(
        "frame ms  avg {:.2}  min {:.2}  p95 {:.2}  max {:.2}",
        stats.frame_avg_ms, stats.frame_min_ms, stats.frame_p95_ms, stats.frame_max_ms
    );
    let memory_line = format!(
        "memory MB  now {:.1}  avg {:.1}  peak {:.1}",
        rss_now, rss_avg, stats.peak_rss_mb
    );
    let cpu_line = format!(
        "cpu %  now {:.1}  avg {:.1}  samples {}",
        stats.cpu_percent, cpu_avg, count
    );

    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .p(px(14.0))
        .rounded(px(12.0))
        .bg(style.surface)
        .text_color(style.text)
        .child(div().text_size(px(15.0)).child(heading))
        .child(div().text_size(px(12.0)).child(frame_line))
        .child(div().text_size(px(12.0)).child(memory_line))
        .child(div().text_size(px(12.0)).child(cpu_line))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(16.0))
                .child(labelled_chart(
                    "Memory (MB)",
                    format!("{rss_now:.1}"),
                    rss_values,
                    rss_max,
                    style,
                    None,
                ))
                .child(labelled_chart(
                    "CPU (%)",
                    format!("{:.1}", stats.cpu_percent),
                    cpu_values,
                    cpu_max,
                    style,
                    None,
                ))
                .child(labelled_chart(
                    "Frame time (ms, line = 60 fps budget)",
                    format!("{:.2}", stats.frame_avg_ms),
                    frame_values,
                    frame_max,
                    style,
                    Some(FRAME_BUDGET_MS),
                )),
        )
}

struct Launcher {
    state: SharedState,
    rust_button: Entity<Button>,
    kotlin_button: Entity<Button>,
    theme_button: Entity<Button>,
    stop_button: Entity<Button>,
}

impl Launcher {
    fn new(cx: &mut Context<Self>, state: SharedState) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(SAMPLE_INTERVAL).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();
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
        let stop_state = Arc::clone(&state);
        let stop_button = cx.new(|cx| {
            Button::new("stop-all", "Stop all", cx)
                .variant(ButtonVariant::Text)
                .on_click(move |_, _| stop_all(&stop_state))
        });
        Self {
            state,
            rust_button,
            kotlin_button,
            theme_button,
            stop_button,
        }
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let surface = theme.color(ColorRole::SurfaceContainer);
        let text = theme.color(ColorRole::OnSurface);
        let rust_style = ChartStyle {
            accent: theme.color(ColorRole::Primary),
            surface,
            text,
        };
        let kotlin_style = ChartStyle {
            accent: theme.color(ColorRole::Tertiary),
            surface,
            text,
        };

        let snapshot: Vec<ChildStats> = {
            let shared = self.state.lock().expect("state lock");
            shared.stats.clone()
        };

        let mut body = div().flex().flex_col().gap(px(16.0));
        if snapshot.is_empty() {
            body = body.child(
                div()
                    .text_size(px(14.0))
                    .text_color(text)
                    .child("No showcase running. Open one to see live stats."),
            );
        }
        for stats in &snapshot {
            let style = if stats.name == "kotlin" { kotlin_style } else { rust_style };
            body = body.child(child_panel(stats, style));
        }

        let content = div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .p(px(24.0))
            .bg(theme.color(ColorRole::Background))
            .child(
                div()
                    .flex()
                    .gap(px(12.0))
                    .child(self.rust_button.clone())
                    .child(self.kotlin_button.clone())
                    .child(self.theme_button.clone())
                    .child(self.stop_button.clone()),
            )
            .child(body);
        div()
            .id("launcher-scroll")
            .size_full()
            .overflow_y_scroll()
            .child(content)
    }
}

fn main() {
    let state: SharedState = Arc::new(Mutex::new(Shared::default()));
    sample_processes(Arc::clone(&state));
    Application::new().run(move |cx: &mut App| {
        MaterialTheme::install(cx, Argb(0xFF4285F4), ThemeMode::Light);
        let bounds = Bounds::centered(None, size(px(1280.0), px(1000.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("Material 3 harness")),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, cx| cx.new(|cx| Launcher::new(cx, state)),
        )
        .unwrap();
        cx.activate(true);
    });
}
