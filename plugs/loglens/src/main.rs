// src/main.rs
use regex::Regex;
use serde_json::Value;
use web_sys::window;
use yew::prelude::*;

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
        .map_err(|_| "Clipboard write failed (HTTPS + user gesture required in some browsers)".to_string())?;
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

/// Render a line with regex matches highlighted.
/// - Works with any UTF-8 text (uses regex match byte ranges and slices safely on boundaries
///   because regex match indices align with UTF-8 boundaries).
/// - Non-overlapping matches (regex crate guarantees this).
fn highlight_line(line: &str, re: &Regex) -> Html {
    let mut out: Vec<Html> = Vec::new();
    let mut cursor = 0usize;

    for m in re.find_iter(line) {
        let s = m.start();
        let e = m.end();
        if s > cursor {
            out.push(html! { <span>{ &line[cursor..s] }</span> });
        }
        out.push(html! { <span class="hl">{ &line[s..e] }</span> });
        cursor = e;
    }

    if cursor < line.len() {
        out.push(html! { <span>{ &line[cursor..] }</span> });
    }

    html! { <>{ for out }</> }
}

#[function_component(App)]
fn app() -> Html {
    let tab = use_state(|| Tab::Explore);

    let log_in = use_state(|| String::new());
    let parsed = use_state(|| Vec::<Entry>::new());

    // filters
    let want_level = use_state(|| "ANY".to_string());
    let needle = use_state(|| String::new());
    let show_json_only = use_state(|| false);

    // extracted fields
    let field_list = use_state(|| "request_id\ntraceId\nuserId\nspanId".to_string());
    let extracted_out = use_state(|| String::new());

    // Regex Highlight Mode (preview)
    let hl_pat = use_state(|| String::new());
    let hl_enabled = use_state(|| false);
    let hl_case_insensitive = use_state(|| true);
    let hl_msg = use_state(|| String::new());

    // messages
    let msg = use_state(|| String::new());

    let set_tab = {
        let tab = tab.clone();
        Callback::from(move |t: Tab| tab.set(t))
    };

    let msg_view = |s: &str| -> Html {
        if s.trim().is_empty() {
            html! { <div class="smallnote">{ " " }</div> }
        } else if s.to_lowercase().contains("error") || s.to_lowercase().contains("failed") {
            html! { <div class="alert">{ s }</div> }
        } else {
            html! { <div class="ok">{ s }</div> }
        }
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
            msg.set(format!("Extracted {rows} JSON entries into TSV (copy/paste into Excel/Sheets)."));
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

    // Build highlight regex (optional)
    let hl_regex: Option<Regex> = {
        if !*hl_enabled {
            None
        } else {
            let pat = (*hl_pat).trim().to_string();
            if pat.is_empty() {
                None
            } else {
                let final_pat = if *hl_case_insensitive {
                    // Prefix inline flag; keeps Cargo.toml untouched
                    format!("(?i:{pat})")
                } else {
                    pat
                };
                match Regex::new(&final_pat) {
                    Ok(re) => {
                        if !hl_msg.trim().is_empty() {
                            // don't spam; only clear if previously errored
                        }
                        Some(re)
                    }
                    Err(e) => {
                        hl_msg.set(format!("Regex highlight error: {e}"));
                        None
                    }
                }
            }
        }
    };

    // Pretty preview rendered as Html (so we can highlight spans)
    let pretty_preview_html = {
        let mut rows: Vec<Html> = Vec::new();
        let mut shown = 0usize;
        let mut total_matches = 0usize;

        for e in &filtered_entries {
            if shown >= 200 {
                rows.push(html! {
                    <>
                      <span>{ "\n… (preview truncated; export/copy for full output)\n" }</span>
                    </>
                });
                break;
            }

            // choose display payload
            let payload = if e.is_json {
                e.json_pretty.clone().unwrap_or_else(|| e.raw.clone())
            } else {
                e.raw.clone()
            };

            // quick line header
            let level_tag = e.level.clone().unwrap_or_else(|| "-".to_string());
            rows.push(html! {
                <>
                  <span>{ format!("— #{:04}  {}\n", e.idx, level_tag) }</span>
                </>
            });

            // content lines (preserve formatting)
            if let Some(re) = &hl_regex {
                // highlight line-by-line to keep it fast and preserve newlines
                for line in payload.lines() {
                    total_matches += re.find_iter(line).count();
                    rows.push(html! { <>{ highlight_line(line, re) }<span>{ "\n" }</span></> });
                }
                rows.push(html! { <span>{ "\n" }</span> });
            } else {
                // no highlighting; dump as text
                rows.push(html! { <span>{ payload }</span> });
                rows.push(html! { <span>{ "\n\n" }</span> });
            }

            shown += 1;
        }

        // if highlight is enabled and valid, show a small status line (non-blocking)
        if *hl_enabled && hl_regex.is_some() {
            hl_msg.set(format!("Highlight ON • matches in preview: {total_matches}"));
        } else if !*hl_enabled {
            if hl_msg.starts_with("Highlight ON") {
                hl_msg.set(String::new());
            }
        }

        html! { <>{ for rows }</> }
    };

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
          </div>
        </div>

        <div class="panel two-col">
          <div class="block">
            <div class="block-head">
              <div class="block-title">{ "Filters + Highlight" }</div>
              <div class="btnrow">
                <button class="btn" onclick={{
                  let show_json_only = show_json_only.clone();
                  Callback::from(move |_| show_json_only.set(!*show_json_only))
                }}>
                  { if *show_json_only { "JSON Only: ON" } else { "JSON Only: OFF" } }
                </button>

                <button class="btn" onclick={{
                  let hl_enabled = hl_enabled.clone();
                  let hl_msg = hl_msg.clone();
                  let hl_pat = hl_pat.clone();
                  Callback::from(move |_| {
                    let next = !*hl_enabled;
                    hl_enabled.set(next);
                    if next && (*hl_pat).trim().is_empty() {
                      hl_msg.set("Highlight ON — enter a regex pattern below.".to_string());
                    } else if !next {
                      hl_msg.set(String::new());
                    }
                  })
                }}>
                  { if *hl_enabled { "Highlight: ON" } else { "Highlight: OFF" } }
                </button>

                <button class="btn" onclick={{
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
                  let hl_msg = hl_msg.clone();
                  Callback::from(move |e: InputEvent| {
                    let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                    hl_pat.set(v);
                    // clear previous regex error while typing
                    if hl_msg.starts_with("Regex highlight error:") {
                      hl_msg.set(String::new());
                    }
                  })
                }}
                placeholder="Regex highlight pattern (e.g. traceId|request_id|ERROR)"
              />
            </div>

            <div class="kv">
              <span class="tag">{ format!("Showing: {}", filtered_entries.len()) }</span>
              <span class="tag">{ "Try: (?i)error|warn|traceId" }</span>
              <span class="tag">{ "Highlight wraps matches in preview" }</span>
            </div>

            { msg_view(&hl_msg) }
          </div>

          <div class="block">
            <div class="block-head">
              <div class="block-title">{ "Preview (first 200)" }</div>
              <div class="btnrow">
                <button class="btn" onclick={{
                  let msg = msg.clone();
                  let log_in = log_in.clone();
                  let parsed = parsed.clone();
                  let hl_msg = hl_msg.clone();
                  Callback::from(move |_| {
                    log_in.set(String::new());
                    parsed.set(Vec::new());
                    msg.set("Cleared input.".to_string());
                    hl_msg.set(String::new());
                  })
                }}>{ "Clear" }</button>
              </div>
            </div>
            <pre class="mono">{ pretty_preview_html }</pre>
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
    let root = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("app")
        .unwrap();
    yew::Renderer::<App>::with_root(root).render();
}