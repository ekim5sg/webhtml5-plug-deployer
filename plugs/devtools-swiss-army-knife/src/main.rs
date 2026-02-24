// src/main.rs
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use regex::Regex;
use sha2::{Digest, Sha256};
use similar::TextDiff;
use urlencoding::{decode, encode};
use uuid::Uuid;
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Json,
    Jwt,
    Base64,
    Url,
    Uuid,
    Hash,
    Diff,
    Regex,
}

fn tab_label(t: Tab) -> &'static str {
    match t {
        Tab::Json => "JSON",
        Tab::Jwt => "JWT",
        Tab::Base64 => "Base64",
        Tab::Url => "URL",
        Tab::Uuid => "UUID",
        Tab::Hash => "Hash",
        Tab::Diff => "Diff",
        Tab::Regex => "Regex",
    }
}

async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let w = window().ok_or("No window".to_string())?;
    let nav = w.navigator();
    let cb = nav.clipboard();
    wasm_bindgen_futures::JsFuture::from(cb.write_text(&text))
        .await
        .map_err(|_| "Clipboard write failed (requires HTTPS + user gesture in many browsers)".to_string())?;
    Ok(())
}

/* ---------- JSON helpers ---------- */

fn pretty_json(input: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("JSON parse error: {e}"))?;
    serde_json::to_string_pretty(&v).map_err(|e| format!("JSON stringify error: {e}"))
}

fn minify_json(input: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("JSON parse error: {e}"))?;
    serde_json::to_string(&v).map_err(|e| format!("JSON stringify error: {e}"))
}

fn sort_json_value(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            // Recursively sort children first
            for (_k, child) in map.iter_mut() {
                sort_json_value(child);
            }
            // Rebuild as BTreeMap to get key order
            let mut bt = std::collections::BTreeMap::<String, serde_json::Value>::new();
            for (k, val) in std::mem::take(map).into_iter() {
                bt.insert(k, val);
            }
            let mut new_map = serde_json::Map::new();
            for (k, val) in bt.into_iter() {
                new_map.insert(k, val);
            }
            *map = new_map;
        }
        serde_json::Value::Array(arr) => {
            for child in arr.iter_mut() {
                sort_json_value(child);
            }
        }
        _ => {}
    }
}

fn normalize_json_for_diff(input: &str) -> Result<String, String> {
    let mut v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("JSON parse error: {e}"))?;
    sort_json_value(&mut v);
    serde_json::to_string_pretty(&v).map_err(|e| format!("JSON stringify error: {e}"))
}

/* ---------- JWT helpers ---------- */

fn decode_jwt_part(part: &str) -> Result<String, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(part.as_bytes())
        .map_err(|e| format!("base64url decode error: {e}"))?;
    let s = String::from_utf8(bytes).map_err(|e| format!("utf8 error: {e}"))?;
    match pretty_json(&s) {
        Ok(p) => Ok(p),
        Err(_) => Ok(s),
    }
}

/* ---------- Hash helpers ---------- */

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let out = hasher.finalize();
    hex_lower(&out)
}

fn md5_hex(input: &str) -> String {
    let digest = md5::compute(input.as_bytes());
    format!("{:x}", digest)
}

fn hex_lower(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(LUT[(b >> 4) as usize] as char);
        s.push(LUT[(b & 0x0f) as usize] as char);
    }
    s
}

/* ---------- Diff helpers ---------- */

fn unified_diff(a: &str, b: &str) -> String {
    let diff = TextDiff::from_lines(a, b);
    diff.unified_diff()
        .header("left", "right")
        .to_string()
}

#[derive(Clone)]
struct DiffLine {
    kind: DiffKind,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DiffKind {
    Meta,
    Add,
    Del,
    Ctx,
}

fn classify_unified_diff(diff: &str) -> Vec<DiffLine> {
    diff.lines()
        .map(|line| {
            let (kind, text) = if line.starts_with("+++") || line.starts_with("---") || line.starts_with("@@") {
                (DiffKind::Meta, line.to_string())
            } else if line.starts_with('+') {
                (DiffKind::Add, line.to_string())
            } else if line.starts_with('-') {
                (DiffKind::Del, line.to_string())
            } else {
                (DiffKind::Ctx, line.to_string())
            };
            DiffLine { kind, text }
        })
        .collect()
}

/* ---------- Regex helpers ---------- */

fn run_regex(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Regex error: {e}"))?;
    let mut out = vec![];

    for (i, caps) in re.captures_iter(text).enumerate() {
        let m0 = caps.get(0).unwrap();
        let mut line = format!(
            "#{i} match [{}..{}] = {:?}",
            m0.start(),
            m0.end(),
            m0.as_str()
        );
        // capture groups
        for gi in 1..caps.len() {
            if let Some(g) = caps.get(gi) {
                line.push_str(&format!(
                    "\n    g{gi} [{}..{}] = {:?}",
                    g.start(),
                    g.end(),
                    g.as_str()
                ));
            } else {
                line.push_str(&format!("\n    g{gi} = <none>"));
            }
        }
        out.push(line);
    }

    if out.is_empty() {
        out.push("No matches.".to_string());
    }

    Ok(out)
}

#[function_component(App)]
fn app() -> Html {
    let tab = use_state(|| Tab::Json);

    // JSON
    let json_in = use_state(|| String::new());
    let json_out = use_state(|| String::new());
    let json_msg = use_state(|| String::new());

    // JWT
    let jwt_in = use_state(|| String::new());
    let jwt_header = use_state(|| String::new());
    let jwt_payload = use_state(|| String::new());
    let jwt_msg = use_state(|| String::new());

    // Base64
    let b64_in = use_state(|| String::new());
    let b64_out = use_state(|| String::new());
    let b64_msg = use_state(|| String::new());

    // URL
    let url_in = use_state(|| String::new());
    let url_out = use_state(|| String::new());
    let url_msg = use_state(|| String::new());

    // UUID
    let uuid_out = use_state(|| String::new());
    let uuid_upper = use_state(|| false);
    let uuid_msg = use_state(|| String::new());

    // Hash
    let hash_in = use_state(|| String::new());
    let hash_sha = use_state(|| String::new());
    let hash_md5 = use_state(|| String::new());
    let hash_msg = use_state(|| String::new());

    // Diff
    let diff_left = use_state(|| String::new());
    let diff_right = use_state(|| String::new());
    let diff_is_json = use_state(|| true);
    let diff_out = use_state(|| Vec::<DiffLine>::new());
    let diff_msg = use_state(|| String::new());

    // Regex
    let rx_pat = use_state(|| String::new());
    let rx_text = use_state(|| String::new());
    let rx_out = use_state(|| String::new());
    let rx_msg = use_state(|| String::new());

    let set_tab = {
        let tab = tab.clone();
        Callback::from(move |t: Tab| tab.set(t))
    };

    let msg_view = |s: &str| -> Html {
        if s.trim().is_empty() {
            html! { <div class="smallnote">{ " " }</div> }
        } else if s.to_lowercase().contains("error")
            || s.to_lowercase().contains("failed")
            || s.to_lowercase().contains("doesn't")
            || s.to_lowercase().contains("invalid")
        {
            html! { <div class="alert">{ s }</div> }
        } else {
            html! { <div class="ok">{ s }</div> }
        }
    };

    /* ---------- JSON actions ---------- */

    let on_json_pretty = {
        let json_in = json_in.clone();
        let json_out = json_out.clone();
        let json_msg = json_msg.clone();
        Callback::from(move |_| {
            match pretty_json(&json_in) {
                Ok(s) => { json_out.set(s); json_msg.set("Pretty-printed OK.".to_string()); }
                Err(e) => json_msg.set(e),
            }
        })
    };

    let on_json_minify = {
        let json_in = json_in.clone();
        let json_out = json_out.clone();
        let json_msg = json_msg.clone();
        Callback::from(move |_| {
            match minify_json(&json_in) {
                Ok(s) => { json_out.set(s); json_msg.set("Minified OK.".to_string()); }
                Err(e) => json_msg.set(e),
            }
        })
    };

    let on_json_swap = {
        let json_in = json_in.clone();
        let json_out = json_out.clone();
        let json_msg = json_msg.clone();
        Callback::from(move |_| {
            let a = (*json_in).clone();
            let b = (*json_out).clone();
            json_in.set(b);
            json_out.set(a);
            json_msg.set("Swapped input/output.".to_string());
        })
    };

    let on_json_copy = {
        let json_out = json_out.clone();
        let json_msg = json_msg.clone();
        Callback::from(move |_| {
            let txt = (*json_out).clone();
            let json_msg2 = json_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => json_msg2.set("Copied output to clipboard.".to_string()),
                    Err(e) => json_msg2.set(e),
                }
            });
        })
    };

    /* ---------- JWT actions ---------- */

    let on_jwt_decode = {
        let jwt_in = jwt_in.clone();
        let jwt_header = jwt_header.clone();
        let jwt_payload = jwt_payload.clone();
        let jwt_msg = jwt_msg.clone();
        Callback::from(move |_| {
            let token = (*jwt_in).trim().to_string();
            if token.is_empty() {
                jwt_msg.set("Paste a JWT first.".to_string());
                return;
            }
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() < 2 {
                jwt_msg.set("That doesn't look like a JWT (need header.payload[.sig]).".to_string());
                return;
            }
            match decode_jwt_part(parts[0]) {
                Ok(h) => jwt_header.set(h),
                Err(e) => { jwt_msg.set(format!("Header: {e}")); return; }
            }
            match decode_jwt_part(parts[1]) {
                Ok(p) => jwt_payload.set(p),
                Err(e) => { jwt_msg.set(format!("Payload: {e}")); return; }
            }
            jwt_msg.set("Decoded header + payload (signature not verified).".to_string());
        })
    };

    let on_jwt_copy_header = {
        let jwt_header = jwt_header.clone();
        let jwt_msg = jwt_msg.clone();
        Callback::from(move |_| {
            let txt = (*jwt_header).clone();
            let jwt_msg2 = jwt_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => jwt_msg2.set("Copied header.".to_string()),
                    Err(e) => jwt_msg2.set(e),
                }
            });
        })
    };

    let on_jwt_copy_payload = {
        let jwt_payload = jwt_payload.clone();
        let jwt_msg = jwt_msg.clone();
        Callback::from(move |_| {
            let txt = (*jwt_payload).clone();
            let jwt_msg2 = jwt_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => jwt_msg2.set("Copied payload.".to_string()),
                    Err(e) => jwt_msg2.set(e),
                }
            });
        })
    };

    /* ---------- Base64 actions ---------- */

    let on_b64_encode = {
        let b64_in = b64_in.clone();
        let b64_out = b64_out.clone();
        let b64_msg = b64_msg.clone();
        Callback::from(move |_| {
            let encoded = base64::engine::general_purpose::STANDARD.encode((*b64_in).as_bytes());
            b64_out.set(encoded);
            b64_msg.set("Encoded OK.".to_string());
        })
    };

    let on_b64_decode = {
        let b64_in = b64_in.clone();
        let b64_out = b64_out.clone();
        let b64_msg = b64_msg.clone();
        Callback::from(move |_| {
            let input = (*b64_in).trim().to_string();
            match base64::engine::general_purpose::STANDARD.decode(input.as_bytes()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => { b64_out.set(s); b64_msg.set("Decoded OK.".to_string()); }
                    Err(e) => b64_msg.set(format!("utf8 error: {e} (decoded bytes aren't UTF-8)")),
                },
                Err(e) => b64_msg.set(format!("base64 decode error: {e}")),
            }
        })
    };

    let on_b64_copy = {
        let b64_out = b64_out.clone();
        let b64_msg = b64_msg.clone();
        Callback::from(move |_| {
            let txt = (*b64_out).clone();
            let b64_msg2 = b64_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => b64_msg2.set("Copied output.".to_string()),
                    Err(e) => b64_msg2.set(e),
                }
            });
        })
    };

    /* ---------- URL actions ---------- */

    let on_url_encode = {
        let url_in = url_in.clone();
        let url_out = url_out.clone();
        let url_msg = url_msg.clone();
        Callback::from(move |_| {
            url_out.set(encode(&*url_in).to_string());
            url_msg.set("Encoded OK.".to_string());
        })
    };

    let on_url_decode = {
        let url_in = url_in.clone();
        let url_out = url_out.clone();
        let url_msg = url_msg.clone();
        Callback::from(move |_| {
            match decode(&*url_in) {
                Ok(s) => { url_out.set(s.to_string()); url_msg.set("Decoded OK.".to_string()); }
                Err(e) => url_msg.set(format!("decode error: {e}")),
            }
        })
    };

    let on_url_copy = {
        let url_out = url_out.clone();
        let url_msg = url_msg.clone();
        Callback::from(move |_| {
            let txt = (*url_out).clone();
            let url_msg2 = url_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => url_msg2.set("Copied output.".to_string()),
                    Err(e) => url_msg2.set(e),
                }
            });
        })
    };

    /* ---------- UUID actions ---------- */

    let on_uuid_generate = {
        let uuid_out = uuid_out.clone();
        let uuid_upper = uuid_upper.clone();
        let uuid_msg = uuid_msg.clone();
        Callback::from(move |_| {
            let mut u = Uuid::new_v4().to_string();
            if *uuid_upper { u = u.to_uppercase(); }
            uuid_out.set(u);
            uuid_msg.set("Generated UUID v4.".to_string());
        })
    };

    let on_uuid_toggle_upper = {
        let uuid_upper = uuid_upper.clone();
        Callback::from(move |_| uuid_upper.set(!*uuid_upper))
    };

    let on_uuid_copy = {
        let uuid_out = uuid_out.clone();
        let uuid_msg = uuid_msg.clone();
        Callback::from(move |_| {
            let txt = (*uuid_out).clone();
            let uuid_msg2 = uuid_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => uuid_msg2.set("Copied UUID.".to_string()),
                    Err(e) => uuid_msg2.set(e),
                }
            });
        })
    };

    /* ---------- Hash actions ---------- */

    let on_hash_run = {
        let hash_in = hash_in.clone();
        let hash_sha = hash_sha.clone();
        let hash_md5 = hash_md5.clone();
        let hash_msg = hash_msg.clone();
        Callback::from(move |_| {
            let input = (*hash_in).clone();
            hash_sha.set(sha256_hex(&input));
            hash_md5.set(md5_hex(&input));
            hash_msg.set("Computed SHA-256 and MD5.".to_string());
        })
    };

    let on_hash_copy_sha = {
        let hash_sha = hash_sha.clone();
        let hash_msg = hash_msg.clone();
        Callback::from(move |_| {
            let txt = (*hash_sha).clone();
            let hash_msg2 = hash_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => hash_msg2.set("Copied SHA-256.".to_string()),
                    Err(e) => hash_msg2.set(e),
                }
            });
        })
    };

    let on_hash_copy_md5 = {
        let hash_md5 = hash_md5.clone();
        let hash_msg = hash_msg.clone();
        Callback::from(move |_| {
            let txt = (*hash_md5).clone();
            let hash_msg2 = hash_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => hash_msg2.set("Copied MD5.".to_string()),
                    Err(e) => hash_msg2.set(e),
                }
            });
        })
    };

    /* ---------- Diff actions ---------- */

    let on_diff_toggle_mode = {
        let diff_is_json = diff_is_json.clone();
        Callback::from(move |_| diff_is_json.set(!*diff_is_json))
    };

    let on_diff_run = {
        let diff_left = diff_left.clone();
        let diff_right = diff_right.clone();
        let diff_is_json = diff_is_json.clone();
        let diff_out = diff_out.clone();
        let diff_msg = diff_msg.clone();
        Callback::from(move |_| {
            let left = (*diff_left).clone();
            let right = (*diff_right).clone();

            let (a, b) = if *diff_is_json {
                let na = match normalize_json_for_diff(&left) {
                    Ok(s) => s,
                    Err(e) => { diff_msg.set(format!("Left: {e}")); return; }
                };
                let nb = match normalize_json_for_diff(&right) {
                    Ok(s) => s,
                    Err(e) => { diff_msg.set(format!("Right: {e}")); return; }
                };
                (na, nb)
            } else {
                (left, right)
            };

            let u = unified_diff(&a, &b);
            diff_out.set(classify_unified_diff(&u));
            diff_msg.set(if *diff_is_json {
                "Diff generated (JSON normalized + sorted keys).".to_string()
            } else {
                "Diff generated (raw text).".to_string()
            });
        })
    };

    let on_diff_copy = {
        let diff_out = diff_out.clone();
        let diff_msg = diff_msg.clone();
        Callback::from(move |_| {
            let joined = (*diff_out)
                .iter()
                .map(|l| l.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let diff_msg2 = diff_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(joined).await {
                    Ok(_) => diff_msg2.set("Copied unified diff.".to_string()),
                    Err(e) => diff_msg2.set(e),
                }
            });
        })
    };

    /* ---------- Regex actions ---------- */

    let on_regex_run = {
        let rx_pat = rx_pat.clone();
        let rx_text = rx_text.clone();
        let rx_out = rx_out.clone();
        let rx_msg = rx_msg.clone();
        Callback::from(move |_| {
            let p = (*rx_pat).clone();
            let t = (*rx_text).clone();
            if p.trim().is_empty() {
                rx_msg.set("Enter a regex pattern.".to_string());
                return;
            }
            match run_regex(&p, &t) {
                Ok(lines) => {
                    rx_out.set(lines.join("\n\n"));
                    rx_msg.set("Regex executed.".to_string());
                }
                Err(e) => rx_msg.set(e),
            }
        })
    };

    let on_regex_copy = {
        let rx_out = rx_out.clone();
        let rx_msg = rx_msg.clone();
        Callback::from(move |_| {
            let txt = (*rx_out).clone();
            let rx_msg2 = rx_msg.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => rx_msg2.set("Copied results.".to_string()),
                    Err(e) => rx_msg2.set(e),
                }
            });
        })
    };

    /* ---------- Views ---------- */

    let render_diff = {
        let lines = (*diff_out).clone();
        html! {
          <pre class="diff">
            { for lines.into_iter().map(|l| {
                let cls = match l.kind {
                    DiffKind::Meta => "meta",
                    DiffKind::Add => "add",
                    DiffKind::Del => "del",
                    DiffKind::Ctx => "ctx",
                };
                html!{ <span class={cls}>{ format!("{}\n", l.text) }</span> }
            })}
          </pre>
        }
    };

    let content = match *tab {
        Tab::Json => html! {
            <div class="panel two-col">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Input JSON" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_json_pretty.clone()}>{ "Pretty →" }</button>
                    <button class="btn" onclick={on_json_minify.clone()}>{ "Minify →" }</button>
                    <button class="btn" onclick={on_json_swap.clone()}>{ "Swap" }</button>
                  </div>
                </div>
                <textarea
                  value={(*json_in).clone()}
                  oninput={{
                    let json_in = json_in.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      json_in.set(v);
                    })
                  }}
                  placeholder="{ \"hello\": \"world\" }"
                />
              </div>

              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Output" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_json_copy}>{ "Copy" }</button>
                  </div>
                </div>
                <textarea
                  value={(*json_out).clone()}
                  oninput={{
                    let json_out = json_out.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      json_out.set(v);
                    })
                  }}
                  placeholder="Pretty / minified result shows here"
                />
              </div>

              { msg_view(&json_msg) }
            </div>
        },

        Tab::Jwt => html! {
            <div class="panel">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "JWT (paste token)" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_jwt_decode}>{ "Decode" }</button>
                  </div>
                </div>
                <textarea
                  value={(*jwt_in).clone()}
                  oninput={{
                    let jwt_in = jwt_in.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      jwt_in.set(v);
                    })
                  }}
                  placeholder="header.payload.signature"
                />
              </div>

              <div class="panel two-col">
                <div class="block">
                  <div class="block-head">
                    <div class="block-title">{ "Header" }</div>
                    <div class="btnrow">
                      <button class="btn" onclick={on_jwt_copy_header}>{ "Copy" }</button>
                    </div>
                  </div>
                  <textarea value={(*jwt_header).clone()} placeholder="Decoded header (pretty JSON if possible)" />
                </div>

                <div class="block">
                  <div class="block-head">
                    <div class="block-title">{ "Payload" }</div>
                    <div class="btnrow">
                      <button class="btn" onclick={on_jwt_copy_payload}>{ "Copy" }</button>
                    </div>
                  </div>
                  <textarea value={(*jwt_payload).clone()} placeholder="Decoded payload (pretty JSON if possible)" />
                </div>
              </div>

              { msg_view(&jwt_msg) }
              <div class="smallnote">{ "Note: this decodes base64url; it does not verify signatures." }</div>
            </div>
        },

        Tab::Base64 => html! {
            <div class="panel two-col">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Input" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_b64_encode.clone()}>{ "Encode →" }</button>
                    <button class="btn" onclick={on_b64_decode.clone()}>{ "Decode →" }</button>
                  </div>
                </div>
                <textarea
                  value={(*b64_in).clone()}
                  oninput={{
                    let b64_in = b64_in.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      b64_in.set(v);
                    })
                  }}
                  placeholder="Text or base64 here"
                />
              </div>

              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Output" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_b64_copy}>{ "Copy" }</button>
                  </div>
                </div>
                <textarea
                  value={(*b64_out).clone()}
                  oninput={{
                    let b64_out = b64_out.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      b64_out.set(v);
                    })
                  }}
                  placeholder="Result shows here"
                />
              </div>

              { msg_view(&b64_msg) }
            </div>
        },

        Tab::Url => html! {
            <div class="panel two-col">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Input" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_url_encode.clone()}>{ "Encode →" }</button>
                    <button class="btn" onclick={on_url_decode.clone()}>{ "Decode →" }</button>
                  </div>
                </div>
                <textarea
                  value={(*url_in).clone()}
                  oninput={{
                    let url_in = url_in.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      url_in.set(v);
                    })
                  }}
                  placeholder="https://example.com?q=hello world&x=1"
                />
              </div>

              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Output" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_url_copy}>{ "Copy" }</button>
                  </div>
                </div>
                <textarea
                  value={(*url_out).clone()}
                  oninput={{
                    let url_out = url_out.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      url_out.set(v);
                    })
                  }}
                  placeholder="Result shows here"
                />
              </div>

              { msg_view(&url_msg) }
            </div>
        },

        Tab::Uuid => html! {
            <div class="panel">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "UUID v4 Generator" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_uuid_generate}>{ "Generate" }</button>
                    <button class="btn" onclick={on_uuid_toggle_upper}>{ if *uuid_upper { "Uppercase: ON" } else { "Uppercase: OFF" } }</button>
                    <button class="btn" onclick={on_uuid_copy}>{ "Copy" }</button>
                  </div>
                </div>
                <textarea value={(*uuid_out).clone()} placeholder="Click Generate" />
              </div>
              { msg_view(&uuid_msg) }
            </div>
        },

        Tab::Hash => html! {
            <div class="panel">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Hash Tools" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_hash_run}>{ "Compute" }</button>
                  </div>
                </div>
                <textarea
                  value={(*hash_in).clone()}
                  oninput={{
                    let hash_in = hash_in.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      hash_in.set(v);
                    })
                  }}
                  placeholder="Enter text to hash"
                />
              </div>

              <div class="panel two-col">
                <div class="block">
                  <div class="block-head">
                    <div class="block-title">{ "SHA-256 (hex)" }</div>
                    <div class="btnrow">
                      <button class="btn" onclick={on_hash_copy_sha}>{ "Copy" }</button>
                    </div>
                  </div>
                  <textarea value={(*hash_sha).clone()} placeholder="Compute to populate" />
                </div>

                <div class="block">
                  <div class="block-head">
                    <div class="block-title">{ "MD5 (hex)" }</div>
                    <div class="btnrow">
                      <button class="btn" onclick={on_hash_copy_md5}>{ "Copy" }</button>
                    </div>
                  </div>
                  <textarea value={(*hash_md5).clone()} placeholder="Compute to populate" />
                </div>
              </div>

              { msg_view(&hash_msg) }
              <div class="smallnote">{ "Tip: MD5 is for test parity/legacy checks; SHA-256 is preferred for modern workflows." }</div>
            </div>
        },

        Tab::Diff => html! {
            <div class="panel">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Diff Viewer" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_diff_toggle_mode}>
                      { if *diff_is_json { "Mode: JSON (normalized)" } else { "Mode: Text (raw)" } }
                    </button>
                    <button class="btn" onclick={on_diff_run}>{ "Diff" }</button>
                    <button class="btn" onclick={on_diff_copy}>{ "Copy Diff" }</button>
                  </div>
                </div>
                <div class="panel two-col">
                  <div class="block">
                    <div class="block-head"><div class="block-title">{ "Left" }</div></div>
                    <textarea
                      value={(*diff_left).clone()}
                      oninput={{
                        let diff_left = diff_left.clone();
                        Callback::from(move |e: InputEvent| {
                          let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                          diff_left.set(v);
                        })
                      }}
                      placeholder="{ \"a\": 1, \"b\": 2 }"
                    />
                  </div>

                  <div class="block">
                    <div class="block-head"><div class="block-title">{ "Right" }</div></div>
                    <textarea
                      value={(*diff_right).clone()}
                      oninput={{
                        let diff_right = diff_right.clone();
                        Callback::from(move |e: InputEvent| {
                          let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                          diff_right.set(v);
                        })
                      }}
                      placeholder="{ \"b\": 2, \"a\": 9 }"
                    />
                  </div>
                </div>
              </div>

              { msg_view(&diff_msg) }

              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Unified Diff" }</div>
                </div>
                { render_diff }
              </div>

              <div class="smallnote">
                { "JSON mode parses + sorts keys + pretty-prints before diffing. Text mode diffs raw input." }
              </div>
            </div>
        },

        Tab::Regex => html! {
            <div class="panel">
              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Regex Sandbox" }</div>
                  <div class="btnrow">
                    <button class="btn" onclick={on_regex_run}>{ "Run" }</button>
                    <button class="btn" onclick={on_regex_copy}>{ "Copy Results" }</button>
                  </div>
                </div>

                <div class="textline">
                  <input
                    type="text"
                    value={(*rx_pat).clone()}
                    oninput={{
                      let rx_pat = rx_pat.clone();
                      Callback::from(move |e: InputEvent| {
                        let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                        rx_pat.set(v);
                      })
                    }}
                    placeholder=r#"Pattern (e.g. (\w+)=(\d+))"#
                  />
                </div>

                <textarea
                  value={(*rx_text).clone()}
                  oninput={{
                    let rx_text = rx_text.clone();
                    Callback::from(move |e: InputEvent| {
                      let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                      rx_text.set(v);
                    })
                  }}
                  placeholder="Text to test against..."
                />
              </div>

              { msg_view(&rx_msg) }

              <div class="block">
                <div class="block-head">
                  <div class="block-title">{ "Matches / Captures" }</div>
                </div>
                <pre class="diff">{ (*rx_out).clone() }</pre>
                <div class="kv">
                  <span class="tag">{ "Tip: Use capture groups () to see g1, g2, ..." }</span>
                  <span class="tag">{ "Runs fully client-side (WASM)" }</span>
                </div>
              </div>
            </div>
        },
    };

    html! {
      <div class="app">
        <div class="tabs" role="tablist" aria-label="DevPocket Tabs">
          { for [
              Tab::Json, Tab::Jwt, Tab::Base64, Tab::Url,
              Tab::Uuid, Tab::Hash, Tab::Diff, Tab::Regex
            ].into_iter().map(|t| {
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
          })}
        </div>

        { content }
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