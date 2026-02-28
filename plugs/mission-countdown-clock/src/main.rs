// src/main.rs
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use gloo_net::http::Request;
use gloo_timers::callback::Interval;

#[wasm_bindgen]
extern "C" {
    // Must match the functions defined in index.html
    #[wasm_bindgen(js_namespace = window, js_name = mccFormatInTz)]
    fn format_in_tz(epoch_ms: f64, tz: &str) -> String;

    #[wasm_bindgen(js_namespace = window, js_name = mccIsoUtc)]
    fn iso_utc(epoch_ms: f64) -> String;
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct LaunchTimeConfig {
    mission_name: Option<String>,
    launch_utc: String, // ISO-8601 UTC: "YYYY-MM-DDTHH:MM:SSZ"
    notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct TzOpt {
    label: &'static str,
    iana: &'static str,
}

const TZ_OPTIONS: &[TzOpt] = &[
    TzOpt { label: "UTC", iana: "UTC" },
    // Use IANA zones so DST is correct automatically.
    TzOpt { label: "PST/PT", iana: "America/Los_Angeles" },
    TzOpt { label: "MST/MT", iana: "America/Denver" },
    TzOpt { label: "CST/CT", iana: "America/Chicago" },
    TzOpt { label: "EST/ET", iana: "America/New_York" },
    TzOpt { label: "Philippines (PHT)", iana: "Asia/Manila" },
];

const LS_TZ_IDX: &str = "mcc_tz_idx";

fn now_ms() -> f64 {
    js_sys::Date::now()
}

fn parse_iso_utc_to_ms(iso: &str) -> Result<f64, String> {
    // JS Date parses ISO-8601 with trailing 'Z' reliably (UTC).
    let d = js_sys::Date::new(&JsValue::from_str(iso));
    let t = d.get_time();
    if t.is_nan() {
        Err("Could not parse launch_utc. Use ISO-8601 UTC like 2026-03-15T13:45:00Z".into())
    } else {
        Ok(t)
    }
}

fn fmt_hhmmss(total_secs: i64) -> String {
    let s = total_secs.abs();
    let hh = s / 3600;
    let mm = (s % 3600) / 60;
    let ss = s % 60;
    format!("{:02}:{:02}:{:02}", hh, mm, ss)
}

fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn clamp_tz_idx(i: usize) -> usize {
    if i >= TZ_OPTIONS.len() { 0 } else { i }
}

fn load_saved_tz_idx() -> usize {
    let Some(ls) = get_local_storage() else { return 3; }; // default CST/Chicago
    let Ok(Some(v)) = ls.get_item(LS_TZ_IDX) else { return 3; };
    clamp_tz_idx(v.parse::<usize>().unwrap_or(3))
}

fn save_tz_idx(i: usize) {
    if let Some(ls) = get_local_storage() {
        let _ = ls.set_item(LS_TZ_IDX, &i.to_string());
    }
}

fn copy_to_clipboard(text: &str) {
    if let Some(w) = web_sys::window() {
        let nav = w.navigator();
        if let Some(clip) = nav.clipboard() {
            let _ = wasm_bindgen_futures::JsFuture::from(clip.write_text(text));
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    // Config + launch time (ms since epoch)
    let cfg = use_state(|| None::<LaunchTimeConfig>);
    let launch_ms = use_state(|| None::<f64>);
    let err = use_state(|| None::<String>);

    // Timer state
    let running = use_state(|| true);
    let tick = use_state(|| 0u64);

    // Display preferences
    let tz_idx = use_state(load_saved_tz_idx);
    let signed_mode = use_state(|| false); // false = auto T-/T+ ; true = explicit sign style (still shows T- or T+)

    // Load launch-time.json once (relative so it works at /mission-countdown-clock/)
    {
        let cfg = cfg.clone();
        let launch_ms = launch_ms.clone();
        let err = err.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                // This resolves to: https://www.webhtml5.info/mission-countdown-clock/launch-time.json
                let resp = Request::get("./launch-time.json").send().await;

                match resp {
                    Ok(r) => match r.json::<LaunchTimeConfig>().await {
                        Ok(c) => match parse_iso_utc_to_ms(&c.launch_utc) {
                            Ok(ms) => {
                                launch_ms.set(Some(ms));
                                cfg.set(Some(c));
                                err.set(None);
                            }
                            Err(e) => err.set(Some(e)),
                        },
                        Err(e) => err.set(Some(format!("Failed parsing launch-time.json: {}", e))),
                    },
                    Err(e) => err.set(Some(format!("Failed fetching ./launch-time.json: {}", e))),
                }
            });
            || ()
        });
    }

    // Tick every second when running
    {
        let tick = tick.clone();
        let running = running.clone();
        use_effect_with(running.clone(), move |r| {
            if **r {
                let handle = Interval::new(1000, move || tick.set(*tick + 1));
                || drop(handle)
            } else {
                || ()
            }
        });
    }

    // Derived
    let mission_name = cfg
        .as_ref()
        .and_then(|c| c.mission_name.clone())
        .unwrap_or_else(|| "Mission Countdown Clock".to_string());

    let tz = TZ_OPTIONS.get(*tz_idx).unwrap_or(&TZ_OPTIONS[0]);

    let computed = if let Some(lm) = *launch_ms {
        let _ = *tick; // keep live updates even if launch_ms constant
        let now = now_ms();
        let delta_s = ((lm - now) / 1000.0).round() as i64; // positive => future

        let prefix = if delta_s >= 0 { "T-" } else { "T+" };
        let t_display = format!("{}{}", prefix, fmt_hhmmss(delta_s));

        let launch_in_sel = format_in_tz(lm, tz.iana);
        let launch_in_utc = format_in_tz(lm, "UTC");
        let now_in_sel = format_in_tz(now, tz.iana);
        let now_in_utc = format_in_tz(now, "UTC");

        Some((t_display, launch_in_sel, launch_in_utc, now_in_sel, now_in_utc, lm))
    } else {
        None
    };

    // Handlers
    let on_toggle_run = {
        let running = running.clone();
        Callback::from(move |_| running.set(!*running))
    };

    let on_reload = {
        let cfg = cfg.clone();
        let launch_ms = launch_ms.clone();
        let err = err.clone();
        Callback::from(move |_| {
            let cfg = cfg.clone();
            let launch_ms = launch_ms.clone();
            let err = err.clone();
            spawn_local(async move {
                let resp = Request::get("./launch-time.json").send().await;
                match resp {
                    Ok(r) => match r.json::<LaunchTimeConfig>().await {
                        Ok(c) => match parse_iso_utc_to_ms(&c.launch_utc) {
                            Ok(ms) => {
                                launch_ms.set(Some(ms));
                                cfg.set(Some(c));
                                err.set(None);
                            }
                            Err(e) => err.set(Some(e)),
                        },
                        Err(e) => err.set(Some(format!("Failed parsing launch-time.json: {}", e))),
                    },
                    Err(e) => err.set(Some(format!("Failed fetching ./launch-time.json: {}", e))),
                }
            });
        })
    };

    let on_tz_change = {
        let tz_idx = tz_idx.clone();
        Callback::from(move |e: Event| {
            let Some(sel) = e.target_dyn_into::<web_sys::HtmlSelectElement>() else { return; };
            if let Ok(v) = sel.value().parse::<usize>() {
                let v = clamp_tz_idx(v);
                save_tz_idx(v);
                tz_idx.set(v);
            }
        })
    };

    let on_toggle_mode = {
        let signed_mode = signed_mode.clone();
        Callback::from(move |_| signed_mode.set(!*signed_mode))
    };

    let on_copy_t = {
        let computed = computed.clone();
        Callback::from(move |_| {
            if let Some((t_display, ..)) = computed.clone() {
                copy_to_clipboard(&t_display);
            }
        })
    };

    let on_copy_launch_iso = {
        let launch_ms = *launch_ms;
        Callback::from(move |_| {
            if let Some(ms) = launch_ms {
                copy_to_clipboard(&iso_utc(ms));
            }
        })
    };

    // Minimal UI (assumes your styles.css provides classes; you can keep your console theme there)
    html! {
        <div class="wrap">
          <div class="header">
            <div class="brand">
              <div class="h1">{ format!("{} — Launch Console", mission_name) }</div>
              <div class="sub">
                { "Source file: " } <span class="code">{ "/mission-countdown-clock/launch-time.json" }</span>
                { " (stored as UTC; display timezone selectable)." }
              </div>
            </div>
          </div>

          <div class="card">
            <div class="cardHead">
              <div class="pills">
                <div class="pill">
                  <span class={classes!("dot", if *running { "good" } else { "warn" })}></span>
                  { if *running { "GO" } else { "HOLD" } }
                </div>
                <div class="pill"><span class="dot"></span>{ "GUIDO" }</div>
                <div class="pill"><span class="dot"></span>{ "FDO" }</div>
                <div class="pill"><span class="dot"></span>{ "EECOM" }</div>
                <div class="pill"><span class="dot"></span>{ "TELMU" }</div>
                <div class="pill"><span class="dot"></span>{ "CAPCOM" }</div>
              </div>

              <div class="row" style="min-width:220px;">
                <select onchange={on_tz_change} value={tz_idx.to_string()}>
                  { for TZ_OPTIONS.iter().enumerate().map(|(i, t)| html!{
                      <option value={i.to_string()}>{ t.label }</option>
                  })}
                </select>
              </div>
            </div>

            <div class="grid">
              <div class="panel">
                <div class="label">
                  <span>{ "Countdown" }</span>
                  <span class="small">{ if *signed_mode { "Mode: signed" } else { "Mode: auto" } }</span>
                </div>

                {
                  if let Some((t_display, launch_in_sel, launch_in_utc, _now_in_sel, _now_in_utc, _lm)) = computed.clone() {
                    html!{
                      <>
                        <div class="big">{ t_display }</div>
                        <div class="bigSmall">
                          <div>{ format!("Launch ({}) — {}", tz.label, launch_in_sel) }</div>
                          <div>{ format!("Launch (UTC) — {}", launch_in_utc) }</div>
                        </div>
                      </>
                    }
                  } else {
                    html!{ <div class="big">{ "—:—:—" }</div> }
                  }
                }

                <div class="btnRow">
                  <button onclick={on_toggle_run}>{ if *running { "Pause (HOLD)" } else { "Resume (GO)" } }</button>
                  <button class="ghost" onclick={on_reload}>{ "Reload JSON" }</button>
                  <button class="ghost" onclick={on_toggle_mode}>{ "Toggle Mode" }</button>
                </div>

                <hr />

                <div class="btnRow">
                  <button class="ghost" onclick={on_copy_t}>{ "Copy T-/T+" }</button>
                  <button class="ghost" onclick={on_copy_launch_iso}>{ "Copy Launch ISO (UTC)" }</button>
                </div>

                if let Some(e) = (*err).clone() {
                  <div class="small" style="margin-top:10px;">
                    <span class="code">{ format!("ERROR: {}", e) }</span>
                  </div>
                }
              </div>

              <div class="panel">
                <div class="label">
                  <span>{ "Clocks" }</span>
                  <span class="small">{ "Live" }</span>
                </div>

                {
                  // live now
                  let _ = *tick;
                  let now = now_ms();
                  let now_in_sel = format_in_tz(now, tz.iana);
                  let now_in_utc = format_in_tz(now, "UTC");
                  html!{
                    <>
                      <div class="bigSmall">{ format!("Now ({}) — {}", tz.label, now_in_sel) }</div>
                      <div class="bigSmall">{ format!("Now (UTC) — {}", now_in_utc) }</div>
                      <hr />
                      <div class="small">
                        { "Tip: using America/* time zones keeps DST correct for PT/MT/CT/ET automatically. Philippines uses Asia/Manila." }
                      </div>
                    </>
                  }
                }
              </div>
            </div>
          </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}