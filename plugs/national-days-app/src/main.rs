use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct DayItem {
    title: String,
    #[serde(default)]
    url: Option<String>,
    summary: String,
    fun_fact: String,
    try_this: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct ApiResp {
    date: String,         // YYYY-MM-DD
    timezone: String,     // America/Chicago
    source: String,       // upstream page
    items: Vec<DayItem>,  // max 5
    generatedAt: String,  // ISO
    #[serde(default)]
    note: Option<String>,
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no window")
}

// web_sys::Navigator::clipboard() returns Clipboard (not Option), so no ok_or needed.
async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let nav = window().navigator();
    let clip = nav.clipboard();
    wasm_bindgen_futures::JsFuture::from(clip.write_text(&text))
        .await
        .map_err(|_| "Clipboard write failed".to_string())?;
    Ok(())
}

#[function_component(App)]
fn app() -> Html {
    let worker_base = use_state(|| {
        // Put your worker URL here after deploy, e.g.:
        // https://national-days-worker.mikegyver.workers.dev
        // For local dev with wrangler: http://127.0.0.1:8787
        String::from("https://national-days-worker.mikegyver.workers.dev")
    });

    let data = use_state(|| None::<ApiResp>);
    let err = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let selected_date = use_state(|| None::<String>);

    let fetch_today = {
        let worker_base = worker_base.clone();
        let data = data.clone();
        let err = err.clone();
        let loading = loading.clone();

        Callback::from(move |_e: MouseEvent| {
            let worker_base = (*worker_base).clone();
            let data = data.clone();
            let err = err.clone();
            let loading = loading.clone();

            loading.set(true);
            err.set(None);

            spawn_local(async move {
                let url = format!("{}/api/today", worker_base.trim_end_matches('/'));
                match gloo_net::http::Request::get(&url).send().await {
                    Ok(resp) => {
                        if !resp.ok() {
                            loading.set(false);
                            err.set(Some(format!("Worker error: HTTP {}", resp.status())));
                            return;
                        }
                        match resp.json::<ApiResp>().await {
                            Ok(j) => {
                                data.set(Some(j));
                                loading.set(false);
                            }
                            Err(_) => {
                                loading.set(false);
                                err.set(Some("Failed to parse JSON from worker.".into()));
                            }
                        }
                    }
                    Err(_) => {
                        loading.set(false);
                        err.set(Some("Network error calling worker.".into()));
                    }
                }
            });
        })
    };

    let fetch_date = {
        let worker_base = worker_base.clone();
        let data = data.clone();
        let err = err.clone();
        let loading = loading.clone();
        let selected_date = selected_date.clone();

        Callback::from(move |_e: MouseEvent| {
            let worker_base = (*worker_base).clone();
            let data = data.clone();
            let err = err.clone();
            let loading = loading.clone();
            let selected = (*selected_date).clone();

            let Some(ymd) = selected else {
                err.set(Some("Pick a date first.".into()));
                return;
            };

            loading.set(true);
            err.set(None);

            spawn_local(async move {
                let url = format!("{}/api/date?ymd={}", worker_base.trim_end_matches('/'), ymd);
                match gloo_net::http::Request::get(&url).send().await {
                    Ok(resp) => {
                        if !resp.ok() {
                            loading.set(false);
                            err.set(Some(format!("Worker error: HTTP {}", resp.status())));
                            return;
                        }
                        match resp.json::<ApiResp>().await {
                            Ok(j) => {
                                data.set(Some(j));
                                loading.set(false);
                            }
                            Err(_) => {
                                loading.set(false);
                                err.set(Some("Failed to parse JSON from worker.".into()));
                            }
                        }
                    }
                    Err(_) => {
                        loading.set(false);
                        err.set(Some("Network error calling worker.".into()));
                    }
                }
            });
        })
    };

    // Auto-load today on first render
    {
        let fetch_today = fetch_today.clone();
        use_effect_with((), move |_| {
            fetch_today.emit(MouseEvent::new("click").unwrap());
            || ()
        });
    }

    let on_date_change = {
        let selected_date = selected_date.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let v = input.value();
            if v.trim().is_empty() {
                selected_date.set(None);
            } else {
                selected_date.set(Some(v));
            }
        })
    };

    let copy_list = {
        let data = data.clone();
        let err = err.clone();
        Callback::from(move |_e: MouseEvent| {
            let err = err.clone();
            if let Some(d) = (*data).clone() {
                let mut lines: Vec<String> = Vec::new();
                for (i, it) in d.items.iter().enumerate() {
                    let mut block = String::new();
                    block.push_str(&format!("{}. {}\n", i + 1, it.title));
                    if let Some(u) = &it.url {
                        block.push_str(&format!("   Link: {}\n", u));
                    }
                    block.push_str(&format!("   Summary: {}\n", it.summary));
                    block.push_str(&format!("   Fun fact: {}\n", it.fun_fact));
                    block.push_str(&format!("   Try this: {}\n", it.try_this));
                    lines.push(block);
                }

                let text = format!(
                    "Top National Days for {} ({}):\n\n{}\nSource: {}\nGenerated: {}",
                    d.date,
                    d.timezone,
                    lines.join("\n"),
                    d.source,
                    d.generatedAt
                );

                spawn_local(async move {
                    if let Err(e) = copy_to_clipboard(text).await {
                        err.set(Some(e));
                    }
                });
            } else {
                err.set(Some("Nothing to copy yet.".into()));
            }
        })
    };

    let content = if *loading {
        html! { <div class="small">{ "Loading today’s list from the worker…" }</div> }
    } else if let Some(d) = (*data).clone() {
        let note = d.note.unwrap_or_default();
        html! {
          <>
            <div class="badges">
              <span class="badge"><strong>{"Date:"}</strong>{format!(" {}", d.date)}</span>
              <span class="badge"><strong>{"TZ:"}</strong>{format!(" {}", d.timezone)}</span>
              <span class="badge"><strong>{"Items:"}</strong>{format!(" {}", d.items.len())}</span>
            </div>

            if !note.is_empty() {
              <div class="err">{ note }</div>
            }

            <div class="grid">
              { for d.items.iter().enumerate().map(|(i, item)| {
                  let title_node = if let Some(u) = &item.url {
                    html!{
                      <a class="titlelink" href={u.clone()} target="_blank" rel="noopener noreferrer">
                        { item.title.clone() }
                      </a>
                    }
                  } else {
                    html!{ <div class="title">{ item.title.clone() }</div> }
                  };

                  html!{
                    <div class="card">
                      <div class="kicker">{ format!("#{}", i+1) }</div>

                      { title_node }

                      <div class="meta">
                        <div class="label">{ "Summary" }</div>
                        <div class="text">{ item.summary.clone() }</div>

                        <div class="label">{ "Fun fact" }</div>
                        <div class="text">{ item.fun_fact.clone() }</div>

                        <div class="label">{ "Try this" }</div>
                        <div class="text">{ item.try_this.clone() }</div>
                      </div>
                    </div>
                  }
              }) }
            </div>

            <div class="footer">
              {"Source: "}
              <a href={d.source.clone()} target="_blank" rel="noopener noreferrer">{ d.source.clone() }</a>
              {" · Generated: "}
              { d.generatedAt }
            </div>
          </>
        }
    } else {
        html! { <div class="small">{ "No data yet." }</div> }
    };

    let err_block = if let Some(e) = (*err).clone() {
        html! { <div class="err">{ e }</div> }
    } else {
        html! {}
    };

    html! {
      <div class="wrap">
        <div class="hero">
          <div class="toprow">
            <div class="hgroup">
              <h1>{ "Top National Days (Max 5)" }</h1>
              <p>
                { "Powered by a Cloudflare Worker + Workers AI. Front-end is Rust + Yew WASM — perfect for a Carpool Lane iPhone Compiler build." }
              </p>
            </div>
          </div>

          <div class="controls">
            <button onclick={fetch_today.clone()}>{ "Refresh Today" }</button>
            <input type="date" onchange={on_date_change} />
            <button class="secondary" onclick={fetch_date}>{ "Load Date" }</button>
            <button class="secondary" onclick={copy_list}>{ "Copy List" }</button>
          </div>

          { err_block }
          <div style="margin-top:12px;">
            { content }
          </div>
        </div>
      </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}