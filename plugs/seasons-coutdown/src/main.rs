use std::collections::HashMap;

use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use js_sys::Date;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Storage};
use yew::prelude::*;

const LS_FACTS_KEY: &str = "seasonFactsJsonOverride";
const DEFAULT_SCRIPT_ID: &str = "season-facts";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall => "Fall",
            Season::Winter => "Winter",
        }
    }

    // Meteorological season starts (month/day)
    fn start_md(self) -> (u32, u32) {
        match self {
            Season::Spring => (3, 1),
            Season::Summer => (6, 1),
            Season::Fall => (9, 1),
            Season::Winter => (12, 1),
        }
    }

    fn all() -> [Season; 4] {
        [Season::Spring, Season::Summer, Season::Fall, Season::Winter]
    }
}

#[derive(Debug, Deserialize, Clone)]
struct FactsFile {
    #[serde(flatten)]
    map: HashMap<String, Vec<String>>,
}

fn local_storage() -> Option<Storage> {
    window()?.local_storage().ok().flatten()
}

fn read_embedded_json(script_id: &str) -> Option<String> {
    let w = window()?;
    let doc = w.document()?;
    let el = doc.get_element_by_id(script_id)?;
    el.text_content()
}

fn today_local_ymd() -> (i32, u32, u32) {
    let now = Date::new_0();
    let y = now.get_full_year() as i32;
    let m = (now.get_month() + 1.0) as u32; // JS months are 0-11
    let d = now.get_date() as u32;
    (y, m, d)
}

// Days until a target local date at 00:00, rolling to next year if already passed/started.
// Returns (target_year, days_until)
fn days_until_month_day(target_month: u32, target_day: u32) -> (i32, i32) {
    let now = Date::new_0();
    let year = now.get_full_year() as i32;

    let t0 = Date::new_with_year_month_day(year as f64, (target_month - 1) as i32, target_day as i32);
    let ms_per_day = 86_400_000.0;

    let diff = t0.get_time() - now.get_time();
    if diff > 0.0 {
        let days = (diff / ms_per_day).ceil() as i32;
        (year, days)
    } else {
        let next_year = year + 1;
        let t1 = Date::new_with_year_month_day(next_year as f64, (target_month - 1) as i32, target_day as i32);
        let diff2 = t1.get_time() - now.get_time();
        let days = (diff2 / ms_per_day).ceil() as i32;
        (next_year, days)
    }
}

// Determine "current season" based on meteorological ranges:
// Spring: Mar-May, Summer: Jun-Aug, Fall: Sep-Nov, Winter: Dec-Feb
fn current_season() -> Season {
    let (_y, m, _d) = today_local_ymd();
    match m {
        3 | 4 | 5 => Season::Spring,
        6 | 7 | 8 => Season::Summer,
        9 | 10 | 11 => Season::Fall,
        _ => Season::Winter,
    }
}

fn format_start_date(season: Season, year: i32) -> String {
    let (m, d) = season.start_md();
    // Keep it simple and unambiguous
    format!("{:04}-{:02}-{:02}", year, m, d)
}

fn pick_random_fact(facts: &FactsFile, season: Season) -> Option<String> {
    let key = season.name().to_string();
    let list = facts.map.get(&key)?;
    if list.is_empty() {
        return None;
    }
    let r = js_sys::Math::random();
    let idx = (r * (list.len() as f64)).floor() as usize;
    list.get(idx).cloned()
}

#[derive(Clone, PartialEq)]
struct SeasonCardData {
    season: Season,
    target_year: i32,
    days: i32,
    is_current: bool,
}

#[function_component(App)]
fn app() -> Html {
    let cards = use_state(|| Vec::<SeasonCardData>::new());
    let now_pill = use_state(|| String::new());

    let facts = use_state(|| None::<FactsFile>);
    let fact_text = use_state(|| String::new());
    let fact_err = use_state(|| String::new());

    let facts_json_editor = use_state(|| String::new());
    let editor_status = use_state(|| String::new());

    let facts_url = use_state(|| String::new());
    let url_status = use_state(|| String::new());

    // Helper: recompute countdown cards + pill
    let recompute = {
        let cards = cards.clone();
        let now_pill = now_pill.clone();
        Callback::from(move |_| {
            let now = Date::new_0();
            let (y, m, d) = today_local_ymd();
            let h = now.get_hours() as i32;
            let min = now.get_minutes() as i32;
            now_pill.set(format!("Local time: {:04}-{:02}-{:02} {:02}:{:02}", y, m, d, h, min));

            let cur = current_season();
            let mut out = Vec::new();
            for s in Season::all() {
                let (sm, sd) = s.start_md();
                let (ty, days) = days_until_month_day(sm, sd);
                out.push(SeasonCardData {
                    season: s,
                    target_year: ty,
                    days,
                    is_current: s == cur,
                });
            }
            cards.set(out);
        })
    };

    // On mount: recompute immediately, then refresh every 60 seconds
    {
        let recompute = recompute.clone();
        use_effect_with((), move |_| {
            recompute.emit(());
            let handle = Interval::new(60_000, move || {
                recompute.emit(());
            });
            || drop(handle)
        });
    }

    // On mount: load facts (localStorage override > embedded JSON)
    {
        let facts = facts.clone();
        let facts_json_editor = facts_json_editor.clone();
        let fact_text = fact_text.clone();
        let fact_err = fact_err.clone();

        use_effect_with((), move |_| {
            let mut chosen_json: Option<String> = None;

            if let Some(ls) = local_storage() {
                if let Ok(Some(v)) = ls.get_item(LS_FACTS_KEY) {
                    if !v.trim().is_empty() {
                        chosen_json = Some(v);
                    }
                }
            }

            if chosen_json.is_none() {
                chosen_json = read_embedded_json(DEFAULT_SCRIPT_ID);
            }

            let Some(json) = chosen_json else {
                fact_err.set("Could not find any facts JSON (embedded or localStorage override).".to_string());
                return || {};
            };

            facts_json_editor.set(json.clone());

            match serde_json::from_str::<FactsFile>(&json) {
                Ok(parsed) => {
                    let season = current_season();
                    let picked = pick_random_fact(&parsed, season)
                        .unwrap_or_else(|| "No facts found for this season in the JSON.".to_string());
                    facts.set(Some(parsed));
                    fact_text.set(picked);
                    fact_err.set(String::new());
                }
                Err(e) => {
                    facts.set(None);
                    fact_text.set(String::new());
                    fact_err.set(format!("Facts JSON parse error: {e}"));
                }
            }

            || {}
        });
    }

    let on_new_fact = {
        let facts = facts.clone();
        let fact_text = fact_text.clone();
        let fact_err = fact_err.clone();
        Callback::from(move |_| {
            if let Some(f) = (*facts).clone() {
                let season = current_season();
                let picked = pick_random_fact(&f, season)
                    .unwrap_or_else(|| "No facts found for this season in the JSON.".to_string());
                fact_text.set(picked);
                fact_err.set(String::new());
            } else {
                fact_err.set("Facts are not loaded (JSON parse error or missing).".to_string());
            }
        })
    };

    let on_apply_editor_json = {
        let facts_json_editor = facts_json_editor.clone();
        let facts = facts.clone();
        let fact_text = fact_text.clone();
        let fact_err = fact_err.clone();
        let editor_status = editor_status.clone();

        Callback::from(move |_| {
            let json = (*facts_json_editor).clone();
            match serde_json::from_str::<FactsFile>(&json) {
                Ok(parsed) => {
                    if let Some(ls) = local_storage() {
                        let _ = ls.set_item(LS_FACTS_KEY, &json);
                    }
                    let season = current_season();
                    let picked = pick_random_fact(&parsed, season)
                        .unwrap_or_else(|| "No facts found for this season in the JSON.".to_string());
                    facts.set(Some(parsed));
                    fact_text.set(picked);
                    fact_err.set(String::new());
                    editor_status.set("Saved override to localStorage and reloaded facts ✅".to_string());
                }
                Err(e) => {
                    editor_status.set(String::new());
                    fact_err.set(format!("Facts JSON parse error: {e}"));
                }
            }
        })
    };

    let on_clear_override = {
        let editor_status = editor_status.clone();
        Callback::from(move |_| {
            if let Some(ls) = local_storage() {
                let _ = ls.remove_item(LS_FACTS_KEY);
            }
            editor_status.set("localStorage override cleared. Refresh to use embedded JSON ✅".to_string());
        })
    };

    let on_fetch_from_url = {
        let facts_url = facts_url.clone();
        let facts = facts.clone();
        let facts_json_editor = facts_json_editor.clone();
        let fact_text = fact_text.clone();
        let fact_err = fact_err.clone();
        let url_status = url_status.clone();

        Callback::from(move |_| {
            let url = (*facts_url).trim().to_string();
            if url.is_empty() {
                url_status.set("Enter a URL first.".to_string());
                return;
            }

            url_status.set("Fetching…".to_string());

            spawn_local(async move {
                let resp = Request::get(&url).send().await;
                match resp {
                    Ok(r) => {
                        let text = r.text().await.unwrap_or_default();
                        match serde_json::from_str::<FactsFile>(&text) {
                            Ok(parsed) => {
                                // Also place into editor + localStorage override for persistence
                                facts_json_editor.set(text.clone());
                                if let Some(ls) = local_storage() {
                                    let _ = ls.set_item(LS_FACTS_KEY, &text);
                                }
                                let season = current_season();
                                let picked = pick_random_fact(&parsed, season)
                                    .unwrap_or_else(|| "No facts found for this season in the JSON.".to_string());
                                facts.set(Some(parsed));
                                fact_text.set(picked);
                                fact_err.set(String::new());
                                url_status.set("Fetched + saved to localStorage override ✅".to_string());
                            }
                            Err(e) => {
                                url_status.set(String::new());
                                fact_err.set(format!("URL JSON parse error: {e}"));
                            }
                        }
                    }
                    Err(e) => {
                        url_status.set(String::new());
                        fact_err.set(format!("Fetch error: {e:?}"));
                    }
                }
            });
        })
    };

    let next_season_label = {
        // find smallest positive days among the season starts
        let mut best: Option<(Season, i32, i32)> = None; // (season, year, days)
        for c in (*cards).iter() {
            if best.is_none() || c.days < best.unwrap().2 {
                best = Some((c.season, c.target_year, c.days));
            }
        }
        best.map(|(s, y, d)| format!("Next up: {} ({}) in {} day{}", s.name(), format_start_date(s, y), d, if d == 1 { "" } else { "s" }))
            .unwrap_or_else(|| "Next up: —".to_string())
    };

    let cur = current_season();

    html! {
      <div class="wrap">
        <div class="top">
          <div class="brand">
            <h1 class="h1">{ "Countdown to Seasons (Carpool Lane)" }</h1>
            <p class="sub">
              { "Meteorological seasons (fixed dates): Spring Mar 1 • Summer Jun 1 • Fall Sep 1 • Winter Dec 1. " }
              { "If a season has already started this year, the countdown automatically rolls to next year." }
            </p>
          </div>
          <div class="pills">
            <div class="pill">{ (*now_pill).clone() }</div>
            <div class="pill">{ next_season_label }</div>
          </div>
        </div>

        <div class="grid">
          { for (*cards).iter().map(|c| {
              let cls = if c.is_current { "card current" } else { "card" };
              let badge_cls = if c.is_current { "badge current" } else { "badge" };
              html!{
                <div class={cls}>
                  <div class="label">
                    <div class="season">{ c.season.name() }</div>
                    <div class={badge_cls}>{ if c.is_current { "Current season" } else { "Countdown" } }</div>
                  </div>

                  <div class="big">
                    { c.days }
                    <small>{ "days" }</small>
                  </div>

                  <div class="meta">
                    { "Starts: " }{ format_start_date(c.season, c.target_year) }
                  </div>
                </div>
              }
          }) }
        </div>

        <div class="row">
          <div class="panel">
            <h2 class="h2">{ "Random Season Fact" }</h2>
            <p class="fact">
              <span class="seasonname">{ cur.name() }</span>
              { " — " }
              { (*fact_text).clone() }
            </p>

            <div class="btns">
              <button onclick={on_new_fact}>{ "New fact" }</button>
              <button onclick={recompute.clone()}>{ "Refresh countdown" }</button>
            </div>

            {
              if !(*fact_err).is_empty() {
                html!{ <div class="err">{ (*fact_err).clone() }</div> }
              } else {
                html!{}
              }
            }
          </div>

          <div class="panel">
            <h2 class="h2">{ "Facts JSON (runtime editable — no recompile)" }</h2>
            <p class="small">
              { "Option A: edit the embedded JSON in index.html (script#season-facts) after deploy. " }
              { "Option B: paste JSON here and Apply — it saves to localStorage as an override." }
            </p>

            <textarea
              value={(*facts_json_editor).clone()}
              oninput={{
                let facts_json_editor = facts_json_editor.clone();
                Callback::from(move |e: InputEvent| {
                  let t = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>();
                  facts_json_editor.set(t.value());
                })
              }}
            />

            <div class="btns">
              <button onclick={on_apply_editor_json}>{ "Apply + Save Override" }</button>
              <button onclick={on_clear_override}>{ "Clear Override" }</button>
            </div>

            {
              if !(*editor_status).is_empty() {
                html!{ <div class="ok">{ (*editor_status).clone() }</div> }
              } else {
                html!{}
              }
            }

            <hr style="border:none;border-top:1px solid rgba(255,255,255,.10); margin:14px 0;" />

            <h2 class="h2">{ "Optional: Load Facts from a URL" }</h2>
            <p class="small">{ "Provide a URL that returns the same JSON shape. Fetching also saves to localStorage override." }</p>

            <input
              placeholder="https://example.com/season-facts.json"
              value={(*facts_url).clone()}
              oninput={{
                let facts_url = facts_url.clone();
                Callback::from(move |e: InputEvent| {
                  let t = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                  facts_url.set(t.value());
                })
              }}
            />

            <div class="btns">
              <button onclick={on_fetch_from_url}>{ "Fetch from URL" }</button>
            </div>

            {
              if !(*url_status).is_empty() {
                html!{ <div class="ok">{ (*url_status).clone() }</div> }
              } else {
                html!{}
              }
            }
          </div>
        </div>
      </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}