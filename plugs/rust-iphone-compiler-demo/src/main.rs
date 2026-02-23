use serde::Deserialize;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::window;
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

    // More reliable than text_content() for script tags in some environments.
    let s = el.inner_html();
    if s.trim().is_empty() {
        Err("prompt-db element found but empty (inner_html was empty)".to_string())
    } else {
        Ok(s)
    }
}

/// Deterministic "daily" index based on YYYY-MM-DD string.
fn daily_index(date_ymd: &str, len: usize) -> usize {
    let mut hash: u64 = 1469598103934665603;
    for b in date_ymd.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    if len == 0 { 0 } else { (hash as usize) % len }
}

fn random_index(len: usize) -> usize {
    if len == 0 { return 0; }
    let r = js_sys::Math::random();
    let idx = (r * (len as f64)) as usize;
    idx.min(len.saturating_sub(1))
}

fn today_ymd() -> String {
    let d = js_sys::Date::new_0();
    let y = d.get_full_year() as i32;
    let m = (d.get_month() + 1) as i32;
    let day = d.get_date() as i32;
    format!("{:04}-{:02}-{:02}", y, m, day)
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
    let clipboard = nav.clipboard();

    JsFuture::from(clipboard.write_text(&text))
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

    // Load prompts once from embedded JSON.
    {
        let prompts = prompts.clone();
        let toast = toast.clone();
        use_effect_with((), move |_| {
            match get_db_json_from_dom() {
                Ok(json) => match serde_json::from_str::<Vec<Prompt>>(&json) {
                    Ok(v) => prompts.set(v),
                    Err(e) => toast.set(Some(format!("JSON parse error: {e}"))),
                },
                Err(e) => toast.set(Some(format!("JSON load error: {e}"))),
            }

            || ()
        });
    }

    // Choose initial prompt after load.
    {
        let idx = idx.clone();
        let today = (*today).clone();
        use_effect_with(prompts.clone(), move |p| {
            if !p.is_empty() {
                let mut chosen: Option<usize> = None;

                if let Some(saved) = ls_get("daily_suno_prompt:last_index") {
                    if let Ok(n) = saved.parse::<usize>() {
                        if n < p.len() {
                            chosen = Some(n);
                        }
                    }
                }

                let di = chosen.unwrap_or_else(|| daily_index(&today, p.len()));
                idx.set(di);
            }
            || ()
        });
    }

    let current = (*prompts).get(*idx).cloned();

    let set_toast = {
        let toast = toast.clone();
        Callback::from(move |msg: String| {
            toast.set(Some(msg));
            let toast2 = toast.clone();
            gloo::timers::callback::Timeout::new(1800, move || {
                toast2.set(None);
            }).forget();
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
            let set_toast2 = set_toast.clone();
            spawn_local(async move {
                match copy_to_clipboard(v).await {
                    Ok(_) => set_toast2.emit(format!("Copied {label}.")),
                    Err(_) => set_toast2.emit("Copy failed (clipboard permission?).".to_string()),
                }
            });
        })
    };

    let copy_all = {
        let set_toast = set_toast.clone();
        let current = current.clone();
        Callback::from(move |_| {
            let set_toast2 = set_toast.clone();
            if let Some(p) = current.clone() {
                let blob = format!(
                    "TITLE:\n{}\n\nSTYLE:\n{}\n\nLYRICS:\n{}",
                    p.song_title, p.style, p.lyrics
                );
                spawn_local(async move {
                    match copy_to_clipboard(blob).await {
                        Ok(_) => set_toast2.emit("Copied ALL (title + style + lyrics).".to_string()),
                        Err(_) => set_toast2.emit("Copy failed (clipboard permission?).".to_string()),
                    }
                });
            } else {
                set_toast2.emit("Nothing to copy yet.".to_string());
            }
        })
    };

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
                                  <textarea value={p.song_title} />
                                </div>

                                <div class="field">
                                  <div class="field-head">
                                    <div class="label">{"Style"}</div>
                                    <button onclick={c_style}>{"Copy"}</button>
                                  </div>
                                  <textarea value={p.style} />
                                </div>

                                <div class="field">
                                  <div class="field-head">
                                    <div class="label">{"Lyrics"}</div>
                                    <button onclick={c_lyrics}>{"Copy"}</button>
                                  </div>
                                  <textarea value={p.lyrics} />
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
                      {"Tip: If you ever move to an external prompts.json, Trunk can copy it too via data-trunk rel=\"copy-file\"."}
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