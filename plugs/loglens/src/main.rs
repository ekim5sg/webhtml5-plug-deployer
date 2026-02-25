// src/main.rs — LogLens (Rust + Yew + WASM)
// Features added:
// - Live streaming log tail simulator (Interval timer)
// - Saved filter presets (localStorage)
// - Highlighted match navigation (next/prev match) with scrollIntoView
//
// Fixes applied (to keep this working version compiling on Yew 0.21):
// - TailMode now derives Debug (required by format!("{:?}", *tail_mode))
// - Live tail use_effect_with teardown now returns a single closure type (no mismatched closures)

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
        .map_err(|_| {
            "Clipboard write failed (HTTPS + user gesture required in some browsers)".to_string()
        })?;
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
            return Some(if lv == "WARNING" {
                "WARN".to_string()
            } else {
                lv.to_string()
            });
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
    for seg in path
        .split('.')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
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

// ---------- presets (localStorage) ----------

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

// ---------- live tail simulator ----------

// FIX: derive Debug so format!("{:?}", *tail_mode) compiles
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
    // deterministic, no rand dependency
    match mode {
        TailMode::Off => "".to_string(),
        TailMode::DemoMixed => match n % 5 {
            0 => format!("2026-02-24T20:11:{:02}Z INFO Server health check OK", (n % 60)),
            1 => format!(
                r#"{{"timestamp":"2026-02-24T20:11:{:02}Z","level":"INFO","service":"gateway","request_id":"req-{:06x}","traceId":"tr-{:04x}","spanId":"sp-{:02}","path":"/api/v1/products","status":200,"duration_ms":{}}}"#,
                (n % 60),
                (n * 97) & 0xffffff,
                (n * 13) & 0xffff,
                (n % 50),
                40 + (n % 260)
            ),
            2 => format!(
                "2026-02-24T20:11:{:02}Z DEBUG Cache miss for key=session:u{}",
                (n % 60),
                1000 + (n % 50)
            ),
            3 => format!(
                r#"{{"timestamp":"2026-02-24T20:11:{:02}Z","level":"WARN","service":"auth-api","request_id":"req-{:06x}","traceId":"tr-{:04x}","userId":"u{}","message":"Password retry count high"}}"#,
                (n % 60),
                (n * 37) & 0xffffff,
                (n * 11) & 0xffff,
                1000 + (n % 50)
            ),
            _ => "java.sql.SQLException: Timeout while waiting for connection".to_string(),
        },
        TailMode::DemoJsonl => format!(
            r#"{{"timestamp":"2026-02-24T20:11:{:02}Z","level":"INFO","service":"orders","request_id":"req-{:06x}","traceId":"tr-{:04x}","userId":"u{}","spanId":"sp-{:02}","message":"Order lookup started"}}"#,
            (n % 60),
            (n * 71) & 0xffffff,
            (n * 19) & 0xffff,
            2000 + (n % 50),
            (n % 50)
        ),
        TailMode::DemoErrors => match n % 3 {
            0 => format!(
                r#"{{"timestamp":"2026-02-24T20:11:{:02}Z","level":"ERROR","service":"orders","request_id":"req-{:06x}","traceId":"tr-{:04x}","spanId":"sp-{:02}","error":"Database timeout","duration_ms":{}}}"#,
                (n % 60),
                (n * 71) & 0xffffff,
                (n * 19) & 0xffff,
                (n % 50),
                1200 + (n % 2500)
            ),
            1 => "ERROR Failed to process request after timeout".to_string(),
            _ => "Traceback (most recent call last):".to_string(),
        },
    }
}

// ---------- highlight rendering with match ids ----------

fn highlight_line(
    line: &str,
    re: &Regex,
    next_match_idx: &mut usize,
    current: Option<usize>,
) -> (Html, usize) {
    let mut out: Vec<Html> = Vec::new();
    let mut cursor = 0usize;
    let mut count = 0usize;

    for m in re.find_iter(line) {
        let s = m.start();
        let e = m.end();
        if s > cursor {
            out.push(html! { <span>{ &line[cursor..s] }</span> });
        }

        let idx = *next_match_idx;
        *next_match_idx += 1;
        count += 1;

        let id = format!("m{idx}");
        let is_current = current == Some(idx);
        let cls = if is_current { "hl current" } else { "hl" };

        out.push(html! { <span id={id} class={cls}>{ &line[s..e] }</span> });
        cursor = e;
    }

    if cursor < line.len() {
        out.push(html! { <span>{ &line[cursor..] }</span> });
    }

    (html! { <>{ for out }</> }, count)
}

fn scroll_to_match(idx: usize) {
    let Some(w) = window() else { return; };
    let Some(doc) = w.document() else { return; };
    let id = format!("m{idx}");
    if let Some(el) = doc.get_element_by_id(&id) {
        el.scroll_into_view();
    }
}

#[function_component(App)]
fn app() -> Html {
    let tab = use_state(|| Tab::Explore);

    // input + parsed
    let log_in = use_state(|| String::new());
    let parsed = use_state(|| Vec::<Entry>::new());

    // filters
    let want_level = use_state(|| "ANY".to_string());
    let needle = use_state(|| String::new());
    let show_json_only = use_state(|| false);

    // extract tab
    let field_list = use_state(|| "request_id\ntraceId\nuserId\nspanId".to_string());
    let extracted_out = use_state(|| String::new());

    // highlight
    let hl_pat = use_state(|| String::new());
    let hl_enabled = use_state(|| false);
    let hl_case_insensitive = use_state(|| true);

    // match navigation
    let current_match = use_state(|| None::<usize>);
    let total_matches = use_state(|| 0usize);

    // presets
    let presets = use_state(|| load_presets());
    let preset_name = use_state(|| "My preset".to_string());

    // live tail
    let tail_mode = use_state(|| TailMode::Off);
    let tail_rate_ms = use_state(|| 650u32);
    let tail_counter = use_state(|| 0u64);

    // status msg
    let msg = use_state(|| String::new());

    // --- effects ---

    // Reset current match when filters/highlight inputs change
    {
        let current_match = current_match.clone();
        let total_matches = total_matches.clone();
        let deps = (
            (*want_level).clone(),
            (*needle).clone(),
            *show_json_only,
            (*hl_pat).clone(),
            *hl_enabled,
            *hl_case_insensitive,
            (*parsed).len(),
        );
        use_effect_with(deps, move |_| {
            current_match.set(None);
            total_matches.set(0);
            || ()
        });
    }

    // Live tail simulator interval
    {
        let tail_mode = tail_mode.clone();
        let tail_rate_ms = tail_rate_ms.clone();
        let tail_counter = tail_counter.clone();
        let log_in = log_in.clone();
        let parsed = parsed.clone();
        let msg = msg.clone();

        let deps = (*tail_mode, *tail_rate_ms);

        // FIX: single teardown closure type (no early return with a different closure)
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

                    // auto-parse on each tick for “live” feel
                    let entries = parse_entries(&log_in);
                    parsed.set(entries);
                    msg.set(format!("Live tail: {} @ {}ms", tail_mode_label(m), r));
                }));
            }

            move || drop(interval)
        });
    }

    // --- helpers ---

    let msg_view = |s: &str| -> Html {
        if s.trim().is_empty() {
            html! { <div class="smallnote">{ " " }</div> }
        } else if s.to_lowercase().contains("error") || s.to_lowercase().contains("failed") {
            html! { <div class="alert">{ s }</div> }
        } else {
            html! { <div class="ok">{ s }</div> }
        }
    };

    let set_tab = {
        let tab = tab.clone();
        Callback::from(move |t: Tab| tab.set(t))
    };

    let on_parse = {
        let log_in = log_in.clone();
        let parsed = parsed.clone();
        let msg = msg.clone();
        Callback::from(move |_| {
            let entries = parse_entries(&log_in);
            let json_count = entries.iter().filter(|e| e.is_json).count();
            let total = entries.len();
            parsed.set(entries);
            msg.set(format!("Parsed {total} entries ({json_count} JSON lines detected)."));
        })
    };

    let filtered_entries = {
        let entries = (*parsed).clone();
        let lv = (*want_level).clone();
        let n = needle.trim().to_lowercase();
        let json_only = *show_json_only;

        entries
            .into_iter()
            .filter(|e| {
                if json_only && !e.is_json {
                    return false;
                }
                if lv != "ANY" {
                    match &e.level {
                        Some(elv) => {
                            if elv != &lv {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                if !n.is_empty() && !e.raw.to_lowercase().contains(&n) {
                    return false;
                }
                true
            })
            .collect::<Vec<_>>()
    };

    // compile regex
    let hl_regex: Result<Option<Regex>, String> = {
        if !*hl_enabled {
            Ok(None)
        } else {
            let pat = (*hl_pat).trim().to_string();
            if pat.is_empty() {
                Ok(None)
            } else {
                let final_pat = if *hl_case_insensitive {
                    format!("(?i:{pat})")
                } else {
                    pat
                };
                Regex::new(&final_pat)
                    .map(Some)
                    .map_err(|e| format!("Regex highlight error: {e}"))
            }
        }
    };

    let on_copy_filtered = {
        let msg = msg.clone();
        let lines = filtered_entries
            .iter()
            .map(|e| e.raw.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        Callback::from(move |_| {
            let txt = lines.clone();
            let msg2 = msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => msg2.set("Copied filtered raw lines.".to_string()),
                    Err(e) => msg2.set(e),
                }
            });
        })
    };

    let on_export_jsonl = {
        let msg = msg.clone();
        let lines = filtered_entries
            .iter()
            .filter_map(|e| if e.is_json { Some(e.raw.as_str()) } else { None })
            .collect::<Vec<_>>()
            .join("\n");
        Callback::from(move |_| {
            let txt = lines.clone();
            let msg2 = msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => msg2.set("Copied filtered JSONL lines.".to_string()),
                    Err(e) => msg2.set(e),
                }
            });
        })
    };

    let on_extract = {
        let msg = msg.clone();
        let extracted_out = extracted_out.clone();
        let field_list = field_list.clone();
        let filtered = filtered_entries.clone();
        Callback::from(move |_| {
            let fields = field_list
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            if fields.is_empty() {
                msg.set("Add one field per line to extract.".to_string());
                extracted_out.set(String::new());
                return;
            }

            let mut out = String::new();
            out.push_str("idx\tlevel");
            for f in &fields {
                out.push('\t');
                out.push_str(f);
            }
            out.push('\n');

            let mut rows = 0usize;

            for e in &filtered {
                if !e.is_json {
                    continue;
                }
                let v: Value = match serde_json::from_str(&e.raw) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                out.push_str(&format!(
                    "{}\t{}",
                    e.idx,
                    e.level.clone().unwrap_or_else(|| "-".to_string())
                ));

                for f in &fields {
                    let val = extract_field(&v, f).unwrap_or_else(|| "".to_string());
                    out.push('\t');
                    out.push_str(&val.replace('\t', " ").replace('\n', " "));
                }
                out.push('\n');
                rows += 1;
            }

            extracted_out.set(out);
            msg.set(format!(
                "Extracted {rows} JSON entries into TSV (copy/paste into Excel/Sheets)."
            ));
        })
    };

    let on_copy_extracted = {
        let extracted_out = extracted_out.clone();
        let msg = msg.clone();
        Callback::from(move |_| {
            let txt = (*extracted_out).clone();
            let msg2 = msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => msg2.set("Copied extracted TSV.".to_string()),
                    Err(e) => msg2.set(e),
                }
            });
        })
    };

    // presets handlers
    let on_save_preset = {
        let presets = presets.clone();
        let preset_name = preset_name.clone();
        let want_level = want_level.clone();
        let needle = needle.clone();
        let show_json_only = show_json_only.clone();
        let hl_enabled = hl_enabled.clone();
        let hl_case_insensitive = hl_case_insensitive.clone();
        let hl_pat = hl_pat.clone();
        let msg = msg.clone();

        Callback::from(move |_| {
            let mut list = (*presets).clone();
            let name = (*preset_name).trim().to_string();
            if name.is_empty() {
                msg.set("Preset name required.".to_string());
                return;
            }

            let p = Preset {
                name: name.clone(),
                level: (*want_level).clone(),
                needle: (*needle).clone(),
                json_only: *show_json_only,
                hl_enabled: *hl_enabled,
                hl_case_insensitive: *hl_case_insensitive,
                hl_pat: (*hl_pat).clone(),
            };

            // upsert by name
            if let Some(ix) = list.iter().position(|x| x.name == name) {
                list[ix] = p;
            } else {
                list.push(p);
            }

            save_presets(&list);
            presets.set(list);
            msg.set("Preset saved to localStorage.".to_string());
        })
    };

    let on_delete_preset = {
        let presets = presets.clone();
        let preset_name = preset_name.clone();
        let msg = msg.clone();

        Callback::from(move |_| {
            let name = (*preset_name).trim().to_string();
            if name.is_empty() {
                msg.set("Enter preset name to delete.".to_string());
                return;
            }
            let mut list = (*presets).clone();
            let before = list.len();
            list.retain(|p| p.name != name);
            if list.len() == before {
                msg.set("Preset not found.".to_string());
                return;
            }
            save_presets(&list);
            presets.set(list);
            msg.set("Preset deleted.".to_string());
        })
    };

    let on_apply_preset = {
        let presets = presets.clone();
        let preset_name = preset_name.clone();
        let want_level = want_level.clone();
        let needle = needle.clone();
        let show_json_only = show_json_only.clone();
        let hl_enabled = hl_enabled.clone();
        let hl_case_insensitive = hl_case_insensitive.clone();
        let hl_pat = hl_pat.clone();
        let msg = msg.clone();

        Callback::from(move |_| {
            let name = (*preset_name).trim().to_string();
            if let Some(p) = (*presets).iter().find(|x| x.name == name) {
                want_level.set(p.level.clone());
                needle.set(p.needle.clone());
                show_json_only.set(p.json_only);
                hl_enabled.set(p.hl_enabled);
                hl_case_insensitive.set(p.hl_case_insensitive);
                hl_pat.set(p.hl_pat.clone());
                msg.set("Preset applied.".to_string());
            } else {
                msg.set("Preset not found.".to_string());
            }
        })
    };

    // preview rendering + match counting
    let (preview_html, highlight_status_line, matches_found) = {
        let mut rows: Vec<Html> = Vec::new();
        let mut shown = 0usize;
        let mut match_total = 0usize;
        let mut next_match_idx = 0usize;

        let current = *current_match;

        let status_base = match &hl_regex {
            Ok(Some(_)) if *hl_enabled => "Highlight ON".to_string(),
            Ok(None) if *hl_enabled => "Highlight ON — enter a regex pattern.".to_string(),
            Ok(_) => String::new(),
            Err(e) => e.clone(),
        };

        for e in &filtered_entries {
            if shown >= 200 {
                rows.push(html! { <span>{ "\n… (preview truncated; export/copy for full output)\n" }</span> });
                break;
            }

            let level_tag = e.level.clone().unwrap_or_else(|| "-".to_string());
            rows.push(html! { <span>{ format!("— #{:04}  {}\n", e.idx, level_tag) }</span> });

            let payload = if e.is_json {
                e.json_pretty.clone().unwrap_or_else(|| e.raw.clone())
            } else {
                e.raw.clone()
            };

            match &hl_regex {
                Ok(Some(re)) if *hl_enabled => {
                    for line in payload.lines() {
                        let (h, c) = highlight_line(line, re, &mut next_match_idx, current);
                        match_total += c;
                        rows.push(html! { <>{ h }<span>{ "\n" }</span></> });
                    }
                    rows.push(html! { <span>{ "\n" }</span> });
                }
                _ => {
                    rows.push(html! { <span>{ payload }</span> });
                    rows.push(html! { <span>{ "\n\n" }</span> });
                }
            }

            shown += 1;
        }

        let status = if *hl_enabled {
            if match_total > 0 && hl_regex.as_ref().ok().and_then(|x| x.as_ref()).is_some() {
                format!("{status_base} • matches in preview: {match_total}")
            } else {
                status_base
            }
        } else {
            String::new()
        };

        (html! { <>{ for rows }</> }, status, match_total)
    };

    // update total_matches state (safe: derived value; keep in sync)
    {
        let total_matches = total_matches.clone();
        use_effect_with(matches_found, move |m| {
            total_matches.set(*m);
            || ()
        });
    }

    // match nav handlers
    let on_match_prev = {
        let current_match = current_match.clone();
        let total_matches = total_matches.clone();
        Callback::from(move |_| {
            let total = *total_matches;
            if total == 0 {
                current_match.set(None);
                return;
            }
            let next = match *current_match {
                None => total - 1,
                Some(0) => total - 1,
                Some(i) => i - 1,
            };
            current_match.set(Some(next));
            scroll_to_match(next);
        })
    };

    let on_match_next = {
        let current_match = current_match.clone();
        let total_matches = total_matches.clone();
        Callback::from(move |_| {
            let total = *total_matches;
            if total == 0 {
                current_match.set(None);
                return;
            }
            let next = match *current_match {
                None => 0,
                Some(i) => (i + 1) % total,
            };
            current_match.set(Some(next));
            scroll_to_match(next);
        })
    };

    // Live tail controls
    let on_tail_toggle = {
        let tail_mode = tail_mode.clone();
        let msg = msg.clone();
        Callback::from(move |_| {
            let next = if *tail_mode == TailMode::Off {
                TailMode::DemoMixed
            } else {
                TailMode::Off
            };
            tail_mode.set(next);
            msg.set(if next == TailMode::Off {
                "Live tail stopped.".to_string()
            } else {
                "Live tail started.".to_string()
            });
        })
    };

    // Views
    let explore_view = html! {
      <div class="panel">
        <div class="block">
          <div class="block-head">
            <div class="block-title">{ "Paste Logs" }</div>
            <div class="btnrow">
              <button class="btn" onclick={on_parse.clone()}>{ "Parse" }</button>
              <button class="btn" onclick={on_copy_filtered.clone()}>{ "Copy Filtered (raw)" }</button>
              <button class="btn" onclick={on_export_jsonl.clone()}>{ "Copy Filtered (JSONL)" }</button>
            </div>
          </div>

          <textarea
            value={(*log_in).clone()}
            oninput={{
              let log_in = log_in.clone();
              Callback::from(move |e: InputEvent| {
                let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                log_in.set(v);
              })
            }}
            placeholder="Paste JSONL logs or plain text logs here…"
          />

          <div class="kv">
            <span class="tag">{ "Tip: JSONL = one JSON object per line" }</span>
            <span class="tag">{ "Everything stays local in the browser" }</span>
            <span class="tag">{ "Live tail simulator available below" }</span>
          </div>
        </div>

        <div class="panel two-col">
          <div class="block">
            <div class="block-head">
              <div class="block-title">{ "Filters • Presets • Highlight" }</div>
              <div class="btnrow">
                <button class="btn small" onclick={{
                  let show_json_only = show_json_only.clone();
                  Callback::from(move |_| show_json_only.set(!*show_json_only))
                }}>
                  { if *show_json_only { "JSON Only: ON" } else { "JSON Only: OFF" } }
                </button>

                <button class="btn small" onclick={{
                  let hl_enabled = hl_enabled.clone();
                  Callback::from(move |_| hl_enabled.set(!*hl_enabled))
                }}>
                  { if *hl_enabled { "Highlight: ON" } else { "Highlight: OFF" } }
                </button>

                <button class="btn small" onclick={{
                  let hl_case_insensitive = hl_case_insensitive.clone();
                  Callback::from(move |_| hl_case_insensitive.set(!*hl_case_insensitive))
                }}>
                  { if *hl_case_insensitive { "Case: i" } else { "Case: exact" } }
                </button>
              </div>
            </div>

            <div class="textline">
              <input
                type="text"
                value={(*needle).clone()}
                oninput={{
                  let needle = needle.clone();
                  Callback::from(move |e: InputEvent| {
                    let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                    needle.set(v);
                  })
                }}
                placeholder="Search keyword (case-insensitive)"
              />
            </div>

            <div class="textline">
              <input
                type="text"
                value={(*want_level).clone()}
                oninput={{
                  let want_level = want_level.clone();
                  Callback::from(move |e: InputEvent| {
                    let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value().to_uppercase();
                    want_level.set(if v.trim().is_empty() { "ANY".to_string() } else { v });
                  })
                }}
                placeholder="Level filter (ANY / INFO / WARN / ERROR / DEBUG / TRACE / FATAL)"
              />
            </div>

            <div class="textline">
              <input
                type="text"
                value={(*hl_pat).clone()}
                oninput={{
                  let hl_pat = hl_pat.clone();
                  Callback::from(move |e: InputEvent| {
                    let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                    hl_pat.set(v);
                  })
                }}
                placeholder="Regex highlight pattern (e.g. traceId|request_id|ERROR)"
              />
            </div>

            <div class="kv">
              <span class="tag">{ format!("Showing: {}", filtered_entries.len()) }</span>
              <span class="tag">{ "Try: error|warn|traceId" }</span>
              <span class="tag">{ "Highlight wraps matches in preview" }</span>
            </div>

            <div class="textline">
              <div class="row">
                <input
                  type="text"
                  value={(*preset_name).clone()}
                  oninput={{
                    let preset_name = preset_name.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                      preset_name.set(v);
                    })
                  }}
                  placeholder="Preset name"
                />
              </div>
              <div class="btnrow" style="padding-top:10px;">
                <button class="btn small" onclick={on_save_preset.clone()}>{ "Save Preset" }</button>
                <button class="btn small" onclick={on_apply_preset.clone()}>{ "Apply Preset" }</button>
                <button class="btn small" onclick={on_delete_preset.clone()}>{ "Delete Preset" }</button>
              </div>
              <div class="smallnote" style="padding-top:8px;">
                { format!("Saved presets: {}", presets.len()) }
              </div>
            </div>

            <div class="textline">
              <div class="row">
                <select
                  value={format!("{:?}", *tail_mode)}
                  onchange={{
                    let tail_mode = tail_mode.clone();
                    Callback::from(move |e: Event| {
                      let v = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value();
                      let m = match v.as_str() {
                        "DemoMixed" => TailMode::DemoMixed,
                        "DemoJsonl" => TailMode::DemoJsonl,
                        "DemoErrors" => TailMode::DemoErrors,
                        _ => TailMode::Off,
                      };
                      tail_mode.set(m);
                    })
                  }}
                >
                  <option value="Off">{ tail_mode_label(TailMode::Off) }</option>
                  <option value="DemoMixed">{ tail_mode_label(TailMode::DemoMixed) }</option>
                  <option value="DemoJsonl">{ tail_mode_label(TailMode::DemoJsonl) }</option>
                  <option value="DemoErrors">{ tail_mode_label(TailMode::DemoErrors) }</option>
                </select>

                <input
                  type="number"
                  value={tail_rate_ms.to_string()}
                  oninput={{
                    let tail_rate_ms = tail_rate_ms.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                      if let Ok(n) = v.parse::<u32>() {
                        let clamped = n.clamp(120, 5000);
                        tail_rate_ms.set(clamped);
                      }
                    })
                  }}
                  placeholder="Tail interval (ms)"
                />
              </div>

              <div class="btnrow" style="padding-top:10px;">
                <button class="btn small" onclick={on_tail_toggle.clone()}>
                  { if *tail_mode == TailMode::Off { "Start Live Tail" } else { "Stop Live Tail" } }
                </button>
              </div>

              <div class="smallnote" style="padding-top:8px;">
                { "Live tail is simulated locally (no network). It appends new lines into the textarea." }
              </div>
            </div>

            { msg_view(&highlight_status_line) }
          </div>

          <div class="block">
            <div class="block-head">
              <div class="block-title">{ "Preview (first 200)" }</div>
              <div class="btnrow">
                <button class="btn small" onclick={on_match_prev.clone()}>{ "◀ Prev match" }</button>
                <button class="btn small" onclick={on_match_next.clone()}>{ "Next match ▶" }</button>

                <button class="btn small" onclick={{
                  let msg = msg.clone();
                  let log_in = log_in.clone();
                  let parsed = parsed.clone();
                  let tail_mode = tail_mode.clone();
                  let tail_counter = tail_counter.clone();
                  Callback::from(move |_| {
                    tail_mode.set(TailMode::Off);
                    tail_counter.set(0);
                    log_in.set(String::new());
                    parsed.set(Vec::new());
                    msg.set("Cleared input.".to_string());
                  })
                }}>{ "Clear" }</button>
              </div>
            </div>

            <pre class="mono">{ preview_html }</pre>

            <div class="kv">
              <span class="tag">{ format!("Matches: {}", *total_matches) }</span>
              <span class="tag">
                {
                  match *current_match {
                    None => "Current: -".to_string(),
                    Some(i) => format!("Current: {}", i + 1),
                  }
                }
              </span>
              <span class="tag">{ "Tip: Next/Prev scrolls to highlighted chips" }</span>
            </div>
          </div>
        </div>
      </div>
    };

    let extract_view = html! {
      <div class="panel">
        <div class="panel two-col">
          <div class="block">
            <div class="block-head">
              <div class="block-title">{ "Fields to Extract (one per line)" }</div>
              <div class="btnrow">
                <button class="btn" onclick={on_extract.clone()}>{ "Extract" }</button>
                <button class="btn" onclick={on_copy_extracted.clone()}>{ "Copy TSV" }</button>
              </div>
            </div>

            <textarea
              value={(*field_list).clone()}
              oninput={{
                let field_list = field_list.clone();
                Callback::from(move |e: InputEvent| {
                  let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                  field_list.set(v);
                })
              }}
              placeholder="traceId\nrequest_id\nuser.id"
            />

            <div class="kv">
              <span class="tag">{ "Supports dotted paths: user.id, request.id" }</span>
              <span class="tag">{ "Only JSON entries produce rows" }</span>
            </div>
          </div>

          <div class="block">
            <div class="block-head">
              <div class="block-title">{ "Extracted TSV (paste into Excel/Sheets)" }</div>
            </div>
            <textarea value={(*extracted_out).clone()} placeholder="Click Extract…" />
          </div>
        </div>

        <div class="smallnote">
          { "Tip: Parse first, apply filters, then Extract — it only uses the currently filtered set." }
        </div>
      </div>
    };

    let body = match *tab {
        Tab::Explore => explore_view,
        Tab::Extract => extract_view,
    };

    html! {
      <div class="app">
        <div class="tabs" role="tablist" aria-label="LogLens Tabs">
          {
            for [Tab::Explore, Tab::Extract].into_iter().map(|t| {
              let is_active = *tab == t;
              let cls = if is_active { "tab active" } else { "tab" };
              let set_tab = set_tab.clone();
              html!{
                <button
                  class={cls}
                  role="tab"
                  aria-selected={is_active.to_string()}
                  onclick={Callback::from(move |_| set_tab.emit(t))}
                >
                  { tab_label(t) }
                </button>
              }
            })
          }
        </div>

        { msg_view(&msg) }
        { body }
      </div>
    }
}

fn main() {
    // main.rs expects: <div id="app"></div>
    let root = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("app")
        .unwrap();

    yew::Renderer::<App>::with_root(root).render();
}