use serde::Deserialize;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Prompt {
    song_title: String,
    style: String,
    lyrics: String,
}

fn get_db_json_from_dom() -> Result<String, String> {
    let win = window().ok_or("no window")?;
    let doc = win.document().ok_or("no document")?;
    let el = doc
        .get_element_by_id("prompt-db")
        .ok_or("missing <script id=\"prompt-db\" type=\"application/json\">")?;
    Ok(el.text_content().unwrap_or_default())
}

/// Deterministic "daily" index based on YYYY-MM-DD string, stable across reloads.
fn daily_index(date_ymd: &str, len: usize) -> usize {
    // Simple, stable hash (FNV-1a-ish) without extra deps
    let mut hash: u64 = 1469598103934665603;
    for b in date_ymd.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    if len == 0 { 0 } else { (hash as usize) % len }
}

/// Random index using JS Math.random()
fn random_index(len: usize) -> usize {
    if len == 0 { return 0; }
    let r = js_sys::Math::random(); // [0,1)
    let idx = (r * (len as f64)) as usize;
    idx.min(len.saturating_sub(1))
}

fn today_ymd() -> String {
    // Use JS Date in local timezone
    let d = js_sys::Date::new_0();
    let y = d.get_full_year();
    let m = d.get_month() + 1.0; // 0-based
    let day = d.get_date();

    // Format as YYYY-MM-DD
    format!(
        "{:04}-{:02}-{:02}",
        y as i32,
        m as i32,
        day as i32
    )
}

fn ls_get(key: &str) -> Option<String> {
    let win = window()?;
    let storage = win.local_storage().ok()??;
    storage.get_item(key).ok()?
}

fn ls_set(key: &str, val: &str) {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(key, val);
        }
    }
}

async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let win = window().ok_or("no window")?;
    let nav = win.navigator();
    let clipboard = nav.clipboard().ok_or("clipboard not available")?;
    wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text))
        .await
        .map_err(|_| "clipboard write failed".to_string())?;
    Ok(())
}

#[function_component(App)]
fn app() -> Html {
    let prompts = use_state(|| Vec::<Prompt>::new());
    let idx = use_state(|| 0usize);
    let toast = use_state(|| Option::<String>::None);
    let today = use_state(today_ymd);

    // Load prompts from embedded JSON once.
    {
        let prompts = prompts.clone();
        use_effect_with((), move |_| {
            let json = match get_db_json_from_dom() {
                Ok(s) => s,
                Err(e) => {
                    web_sys::console::error_1(&e.into());
                    "[]".to_string()
                }
            };
            match serde_json::from_str::<Vec<Prompt>>(&json) {
                Ok(v) => prompts.set(v),
                Err(e) => web_sys::console::error_1(&format!("JSON parse error: {e}").into()),
            };
            || ()
        });
    }

    // After prompts load, choose index: localStorage override else daily.
    {
        let prompts = prompts.clone();
        let idx = idx.clone();
        let today = (*today).clone();
        use_effect_with(prompts, move |p| {
            if p.is_empty() {
                return || ();
            }

            // If user previously shuffled and we saved it, restore that index
            if let Some(saved) = ls_get("daily_suno_prompt:last_index") {
                if let Ok(n) = saved.parse::<usize>() {
                    if n < p.len() {
                        idx.set(n);
                        return || ();
                    }
                }
            }

            let di = daily_index(&today, p.len());
            idx.set(di);
            || ()
        });
    }

    let current = (*prompts).get(*idx).cloned();

    let set_toast = {
        let toast = toast.clone();
        Callback::from(move |msg: String| {
            toast.set(Some(msg));
            // Auto-clear toast after ~1.8s using a JS timeout
            let toast2 = toast.clone();
            let _ = gloo::timers::callback::Timeout::new(1800, move || {
                toast2.set(None);
            })
            .forget();
        })
    };

    let on_shuffle = {
        let prompts = prompts.clone();
        let idx = idx.clone();
        let set_toast = set_toast.clone();
        Callback::from(move |_| {
            if prompts.is_empty() {
                set_toast.emit("No prompts loaded.".to_string());
                return;
            }
            let n = random_index(prompts.len());
            idx.set(n);
            ls_set("daily_suno_prompt:last_index", &n.to_string());
            set_toast.emit("Shuffled a new prompt.".to_string());
        })
    };

    let on_prev = {
        let prompts = prompts.clone();
        let idx = idx.clone();
        Callback::from(move |_| {
            if prompts.is_empty() { return; }
            let len = prompts.len();
            let cur = *idx;
            let n = if cur == 0 { len - 1 } else { cur - 1 };
            idx.set(n);
            ls_set("daily_suno_prompt:last_index", &n.to_string());
        })
    };

    let on_next = {
        let prompts = prompts.clone();
        let idx = idx.clone();
        Callback::from(move |_| {
            if prompts.is_empty() { return; }
            let len = prompts.len();
            let cur = *idx;
            let n = (cur + 1) % len;
            idx.set(n);
            ls_set("daily_suno_prompt:last_index", &n.to_string());
        })
    };

    let on_daily_reset = {
        let prompts = prompts.clone();
        let idx = idx.clone();
        let today = (*today).clone();
        let set_toast = set_toast.clone();
        Callback::from(move |_| {
            if prompts.is_empty() {
                set_toast.emit("No prompts loaded.".to_string());
                return;
            }
            let di = daily_index(&today, prompts.len());
            idx.set(di);
            ls_set("daily_suno_prompt:last_index", &di.to_string());
            set_toast.emit("Reset to today's Daily Pick.".to_string());
        })
    };

    let copy_field = |label: &'static str, value: String| {
        let set_toast = set_toast.clone();
        Callback::from(move |_| {
            let v = value.clone();
            let set_toast = set_toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(v).await {
                    Ok(_) => set_toast.emit(format!("Copied {label}.")),
                    Err(_) => set_toast.emit("Copy failed (clipboard permission?).".to_string()),
                }
            });
        })
    };

    let copy_all = {
        let set_toast = set_toast.clone();
        let current = current.clone();
        Callback::from(move |_| {
            let set_toast = set_toast.clone();
            if let Some(p) = current.clone() {
                let blob = format!(
                    "TITLE:\n{}\n\nSTYLE:\n{}\n\nLYRICS:\n{}",
                    p.song_title, p.style, p.lyrics
                );
                wasm_bindgen_futures::spawn_local(async move {
                    match copy_to_clipboard(blob).await {
                        Ok(_) => set_toast.emit("Copied ALL (title + style + lyrics).".to_string()),
                        Err(_) => set_toast.emit("Copy failed (clipboard permission?).".to_string()),
                    }
                });
            } else {
                set_toast.emit("Nothing to copy yet.".to_string());
            }
        })
    };

    // Optional: autoresize textareas on input (purely cosmetic). Keep simple.
    let on_textarea_input = Callback::from(|e: InputEvent| {
        if let Some(target) = e.target() {
            if let Ok(ta) = target.dyn_into::<HtmlTextAreaElement>() {
                let _ = ta.style().set_property("height", "auto");
                let h = ta.scroll_height();
                let _ = ta
                    .style()
                    .set_property("height", &format!("{}px", h.max(72)));
            }
        }
    });

    html! {
        <div class="wrap">
            <div class="header">
                <div class="kicker">
                    <div class="brand">
                        <span>{"🎵 Daily Suno Prompt"}</span>
                        <span class="badge">{"Rust/WASM"}</span>
                        <span class="badge">{"Local JSON DB"}</span>
                        <span class="badge">{"Copy-first UI"}</span>
                    </div>
                    <div class="badges">
                        <span class="badge">{format!("Today: {}", (*today))}</span>
                        <span class="badge">{format!("Loaded: {}", prompts.len())}</span>
                    </div>
                </div>
                <h1 class="h1">{"One seriously awesome Suno song prompt per day."}</h1>
                <p class="sub">
                    {"Daily Pick is deterministic (changes each day). Use Shuffle to explore, then copy Title/Style/Lyrics into Suno in seconds."}
                </p>
            </div>

            <div class="card">
                <div class="card-inner">
                    <div class="controls">
                        <div class="btnrow">
                            <button class="primary" onclick={on_daily_reset}>{"Daily Pick"}</button>
                            <button onclick={on_shuffle}>{"Shuffle"}</button>
                            <button onclick={on_prev}>{"Prev"}</button>
                            <button onclick={on_next}>{"Next"}</button>
                        </div>
                        <div class="btnrow">
                            <button onclick={copy_all}>{"Copy All"}</button>
                        </div>
                    </div>

                    <div class="meta">
                        {
                            if let Some(p) = current.clone() {
                                html!{
                                    <>
                                      <span>{format!("Prompt {}/{}", (*idx + 1), prompts.len().max(1))}</span>
                                      <span>{"•"}</span>
                                      <span>{format!("“{}”", p.song_title)}</span>
                                    </>
                                }
                            } else {
                                html!{ <span>{"Loading prompts…"}</span> }
                            }
                        }
                    </div>

                    <hr class="sep" />

                    {
                        if let Some(p) = current {
                            let c_title = copy_field("Title", p.song_title.clone());
                            let c_style = copy_field("Style", p.style.clone());
                            let c_lyrics = copy_field("Lyrics", p.lyrics.clone());

                            html!{
                              <div class="grid">
                                <div class="field">
                                  <div class="field-head">
                                    <div class="label">{"Song Title"}</div>
                                    <button onclick={c_title}>{"Copy"}</button>
                                  </div>
                                  <textarea value={p.song_title} oninput={on_textarea_input.clone()} />
                                </div>

                                <div class="field">
                                  <div class="field-head">
                                    <div class="label">{"Style"}</div>
                                    <button onclick={c_style}>{"Copy"}</button>
                                  </div>
                                  <textarea value={p.style} oninput={on_textarea_input.clone()} />
                                </div>

                                <div class="field">
                                  <div class="field-head">
                                    <div class="label">{"Lyrics"}</div>
                                    <button onclick={c_lyrics}>{"Copy"}</button>
                                  </div>
                                  <textarea value={p.lyrics} oninput={on_textarea_input} />
                                </div>
                              </div>
                            }
                        } else {
                            html! {
                              <div class="field">
                                <div class="field-head">
                                  <div class="label">{"Status"}</div>
                                  <button disabled=true>{"Copy"}</button>
                                </div>
                                <textarea value={"Loading database…"} />
                              </div>
                            }
                        }
                    }

                    <div class="footer">
                      {"Tip: if you want the JSON as a standalone file later, move the <script id=\"prompt-db\"> content into prompts.json and fetch it. This build keeps everything in the 4-file constraint."}
                    </div>
                </div>
            </div>

            {
                if let Some(msg) = (*toast).clone() {
                    html!{ <div class="toast">{msg}</div> }
                } else {
                    html!{}
                }
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}