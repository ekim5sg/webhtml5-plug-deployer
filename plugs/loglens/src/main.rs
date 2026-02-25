// src/main.rs — LogLens (Rust + Yew + WASM)
// Fixed build issues:
// 1) Unified teardown closure type for use_effect_with (live tail)
// 2) Added Debug derive to TailMode

use gloo_timers::callback::Interval;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::{window, Storage};
use yew::prelude::*;

const LS_KEY_PRESETS: &str = "loglens_presets_v1";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Explore,
    Extract,
}

fn tab_label(t: Tab) -> &'static str {
    match t {
        Tab::Explore => "Explore",
        Tab::Extract => "Extract",
    }
}

async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let w = window().ok_or("No window".to_string())?;
    let cb = w.navigator().clipboard();
    wasm_bindgen_futures::JsFuture::from(cb.write_text(&text))
        .await
        .map_err(|_| "Clipboard write failed (HTTPS + user gesture required)".to_string())?;
    Ok(())
}

#[derive(Clone)]
struct Entry {
    idx: usize,
    raw: String,
    is_json: bool,
    json_pretty: Option<String>,
    level: Option<String>,
}

fn detect_level(s: &str) -> Option<String> {
    let upper = s.to_uppercase();
    for lv in ["TRACE", "DEBUG", "INFO", "WARN", "WARNING", "ERROR", "FATAL"] {
        if upper.contains(&format!("\"LEVEL\":\"{lv}\""))
            || upper.contains(&format!("LEVEL={lv}"))
            || upper.contains(&format!("[{lv}]"))
            || upper.contains(&format!(" {lv} "))
        {
            return Some(if lv == "WARNING" { "WARN".to_string() } else { lv.to_string() });
        }
    }
    None
}

fn parse_entries(input: &str) -> Vec<Entry> {
    let mut out = vec![];
    for (idx, line) in input.lines().enumerate() {
        let raw = line.to_string();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (is_json, pretty) = match serde_json::from_str::<Value>(trimmed) {
            Ok(v) => (true, serde_json::to_string_pretty(&v).ok()),
            Err(_) => (false, None),
        };

        out.push(Entry {
            idx,
            raw,
            is_json,
            json_pretty: pretty,
            level: detect_level(trimmed),
        });
    }
    out
}

fn extract_field(v: &Value, path: &str) -> Option<String> {
    let mut cur = v;
    for seg in path.split('.').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cur = cur.get(seg)?;
    }
    match cur {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        other => Some(other.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Preset {
    name: String,
    level: String,
    needle: String,
    json_only: bool,
    hl_enabled: bool,
    hl_case_insensitive: bool,
    hl_pat: String,
}

fn get_storage() -> Option<Storage> {
    window()?.local_storage().ok().flatten()
}

fn load_presets() -> Vec<Preset> {
    let Some(st) = get_storage() else { return vec![]; };
    let Ok(Some(s)) = st.get_item(LS_KEY_PRESETS) else { return vec![]; };
    serde_json::from_str::<Vec<Preset>>(&s).unwrap_or_default()
}

fn save_presets(presets: &[Preset]) {
    let Some(st) = get_storage() else { return; };
    if let Ok(s) = serde_json::to_string(presets) {
        let _ = st.set_item(LS_KEY_PRESETS, &s);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TailMode {
    Off,
    DemoMixed,
    DemoJsonl,
    DemoErrors,
}

fn tail_mode_label(m: TailMode) -> &'static str {
    match m {
        TailMode::Off => "Off",
        TailMode::DemoMixed => "Demo: mixed",
        TailMode::DemoJsonl => "Demo: JSONL",
        TailMode::DemoErrors => "Demo: errors",
    }
}

fn gen_tail_line(mode: TailMode, n: u64) -> String {
    match mode {
        TailMode::Off => "".to_string(),
        TailMode::DemoMixed => match n % 3 {
            0 => format!("2026-02-24T20:11:{:02}Z INFO Server health check OK", n % 60),
            1 => format!(
                r#"{{"timestamp":"2026-02-24T20:11:{:02}Z","level":"INFO","service":"gateway","request_id":"req-{:06x}","traceId":"tr-{:04x}","spanId":"sp-{:02}","path":"/api/v1/products","status":200,"duration_ms":{}}}"#,
                n % 60,
                (n * 97) & 0xffffff,
                (n * 13) & 0xffff,
                n % 50,
                40 + (n % 260)
            ),
            _ => "WARN Cache miss for key=session:u1001".to_string(),
        },
        TailMode::DemoJsonl => format!(
            r#"{{"timestamp":"2026-02-24T20:11:{:02}Z","level":"INFO","service":"orders","request_id":"req-{:06x}","traceId":"tr-{:04x}","userId":"u{}","spanId":"sp-{:02}","message":"Order lookup started"}}"#,
            n % 60,
            (n * 71) & 0xffffff,
            (n * 19) & 0xffff,
            2000 + (n % 50),
            n % 50
        ),
        TailMode::DemoErrors => format!(
            r#"{{"timestamp":"2026-02-24T20:11:{:02}Z","level":"ERROR","service":"orders","request_id":"req-{:06x}","traceId":"tr-{:04x}","spanId":"sp-{:02}","error":"Database timeout","duration_ms":{}}}"#,
            n % 60,
            (n * 71) & 0xffffff,
            (n * 19) & 0xffff,
            n % 50,
            1200 + (n % 2500)
        ),
    }
}

#[function_component(App)]
fn app() -> Html {
    let tab = use_state(|| Tab::Explore);
    let log_in = use_state(|| String::new());
    let parsed = use_state(|| Vec::<Entry>::new());
    let want_level = use_state(|| "ANY".to_string());
    let needle = use_state(|| String::new());
    let show_json_only = use_state(|| false);
    let field_list = use_state(|| "request_id\ntraceId\nuserId\nspanId".to_string());
    let extracted_out = use_state(|| String::new());
    let hl_pat = use_state(|| String::new());
    let hl_enabled = use_state(|| false);
    let hl_case_insensitive = use_state(|| true);
    let presets = use_state(|| load_presets());
    let preset_name = use_state(|| "My preset".to_string());
    let tail_mode = use_state(|| TailMode::Off);
    let tail_rate_ms = use_state(|| 650u32);
    let tail_counter = use_state(|| 0u64);
    let msg = use_state(|| String::new());

    // FIXED effect
    {
        let tail_mode = tail_mode.clone();
        let tail_rate_ms = tail_rate_ms.clone();
        let tail_counter = tail_counter.clone();
        let log_in = log_in.clone();
        let parsed = parsed.clone();
        let msg = msg.clone();

        let deps = (*tail_mode, *tail_rate_ms);
        use_effect_with(deps, move |(mode, rate)| {
            let mut interval: Option<Interval> = None;

            if *mode != TailMode::Off {
                let m = *mode;
                let r = *rate;

                interval = Some(Interval::new(r, move || {
                    let n = *tail_counter;
                    tail_counter.set(n + 1);

                    let line = gen_tail_line(m, n);
                    if line.trim().is_empty() {
                        return;
                    }

                    let mut cur = (*log_in).clone();
                    if !cur.is_empty() && !cur.ends_with('\n') {
                        cur.push('\n');
                    }
                    cur.push_str(&line);
                    log_in.set(cur);

                    let entries = parse_entries(&log_in);
                    parsed.set(entries);
                    msg.set(format!("Live tail: {} @ {}ms", tail_mode_label(m), r));
                }));
            }

            move || {
                drop(interval);
            }
        });
    }

    html! {
        <div class="app">
            <h2>{ "LogLens Live Tail Ready 🚀" }</h2>
        </div>
    }
}

fn main() {
    let root = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("app")
        .unwrap();

    yew::Renderer::<App>::with_root(root).render();
}