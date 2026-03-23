use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

const API_URL: &str = "https://carpool-content-pack.mikegyver.workers.dev/api/content-pack";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
struct ContentPackRequest {
    prompt: String,
    brand_label: String,
    tone: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
struct ContentPackResponse {
    title: String,
    suno_prompt: String,
    lyrics: String,
    sora_prompts: Vec<String>,
    cover_art_prompt: String,
    linkedin_post: String,
    youtube_description: String,
}

#[function_component(App)]
fn app() -> Html {
    let prompt = use_state(|| String::new());
    let brand_label = use_state(|| "MikeGyver Studio".to_string());
    let tone = use_state(|| "cinematic, practical, uplifting".to_string());

    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let result = use_state(|| None::<ContentPackResponse>);
    let copied_message = use_state(|| None::<String>);

    let on_prompt_input = {
        let prompt = prompt.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            prompt.set(input.value());
        })
    };

    let on_brand_input = {
        let brand_label = brand_label.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            brand_label.set(input.value());
        })
    };

    let on_tone_input = {
        let tone = tone.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            tone.set(input.value());
        })
    };

    let on_generate = {
        let prompt = prompt.clone();
        let brand_label = brand_label.clone();
        let tone = tone.clone();
        let loading = loading.clone();
        let error = error.clone();
        let result = result.clone();
        let copied_message = copied_message.clone();

        Callback::from(move |_| {
            if prompt.trim().is_empty() {
                error.set(Some("Please enter a topic or idea first.".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);
            copied_message.set(None);

            let req = ContentPackRequest {
                prompt: (*prompt).clone(),
                brand_label: (*brand_label).clone(),
                tone: (*tone).clone(),
            };

            let loading = loading.clone();
            let error = error.clone();
            let result = result.clone();

            spawn_local(async move {
                let body = match serde_json::to_string(&req) {
                    Ok(v) => v,
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(format!("Failed to serialize request: {e}")));
                        return;
                    }
                };

                let request_builder = Request::post(API_URL)
                    .header("Content-Type", "application/json")
                    .body(body);

                let response = match request_builder {
                    Ok(req) => req.send().await,
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(format!("Failed to build request: {e}")));
                        return;
                    }
                };

                let response = match response {
                    Ok(resp) => resp,
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(format!("Request failed: {e}")));
                        return;
                    }
                };

                if !response.ok() {
                    let status = response.status();
                    let text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown server error".to_string());
                    loading.set(false);
                    error.set(Some(format!("Server error ({status}): {text}")));
                    return;
                }

                let parsed = response.json::<ContentPackResponse>().await;
                match parsed {
                    Ok(data) => {
                        result.set(Some(data));
                        loading.set(false);
                    }
                    Err(e) => {
                        loading.set(false);
                        error.set(Some(format!("Failed to parse response: {e}")));
                    }
                }
            });
        })
    };

    let on_clear = {
        let prompt = prompt.clone();
        let result = result.clone();
        let error = error.clone();
        let copied_message = copied_message.clone();

        Callback::from(move |_| {
            prompt.set(String::new());
            result.set(None);
            error.set(None);
            copied_message.set(None);
        })
    };

    let make_copy_callback =
        |text: String, label: &'static str, copied_message: UseStateHandle<Option<String>>| {
            Callback::from(move |_| {
                let text = text.clone();
                let copied_message = copied_message.clone();

                spawn_local(async move {
                    let maybe_window = web_sys::window();
                    if let Some(window) = maybe_window {
                        let clipboard = window.navigator().clipboard();
                        let promise = clipboard.write_text(&text);

                        match wasm_bindgen_futures::JsFuture::from(promise).await {
                            Ok(_) => {
                                copied_message.set(Some(format!("{label} copied.")));
                            }
                            Err(_) => {
                                copied_message.set(Some(format!("Could not copy {label}.")));
                            }
                        }
                    } else {
                        copied_message.set(Some("Clipboard unavailable.".to_string()));
                    }
                });
            })
        };

    html! {
        <main class="shell">
            <section class="topbar">
                <div class="brand">{ "MikeGyver Studio" }</div>
                <div class="title">{ "Carpool Content Pack Generator" }</div>
                <div class="subtitle">
                    { "Paste one idea and generate a full content pack for music, story, podcast, or social promotion — built for fast carpool-lane style workflows." }
                </div>
            </section>

            <section class="grid">
                <article class="card">
                    <h2>{ "Input" }</h2>

                    <label class="label" for="prompt">{ "Topic / Idea / Theme" }</label>
                    <textarea
                        id="prompt"
                        class="textarea"
                        placeholder="Example: A cinematic story-song about small acts of courage changing a family legacy..."
                        value={(*prompt).clone()}
                        oninput={on_prompt_input}
                    />

                    <div style="height: 12px;"></div>

                    <label class="label" for="brand">{ "Brand Label" }</label>
                    <input
                        id="brand"
                        class="input"
                        value={(*brand_label).clone()}
                        oninput={on_brand_input}
                    />

                    <div style="height: 12px;"></div>

                    <label class="label" for="tone">{ "Tone / Style Bias" }</label>
                    <input
                        id="tone"
                        class="input"
                        value={(*tone).clone()}
                        oninput={on_tone_input}
                    />

                    <div style="height: 16px;"></div>

                    <div class="row">
                        <button class="btn btn-primary" onclick={on_generate.clone()} disabled={*loading}>
                            { if *loading { "Generating..." } else { "Generate Content Pack" } }
                        </button>
                        <button class="btn btn-secondary" onclick={on_clear}>
                            { "Clear" }
                        </button>
                    </div>

                    <div class="meta">
                        { "Expected output: title, Suno prompt, lyrics, 4 Sora prompts, cover prompt, LinkedIn post, and YouTube description." }
                    </div>

                    {
                        if let Some(msg) = &*error {
                            html! { <div class="status err">{ msg.clone() }</div> }
                        } else {
                            html! {}
                        }
                    }

                    {
                        if let Some(msg) = &*copied_message {
                            html! { <div class="status ok">{ msg.clone() }</div> }
                        } else {
                            html! {}
                        }
                    }
                </article>

                <article class="card">
                    <h2>{ "Generated Pack" }</h2>

                    {
                        if let Some(data) = &*result {
                            let sora_joined = data.sora_prompts.join("\n\n");
                            let title_copy = make_copy_callback(data.title.clone(), "Title", copied_message.clone());
                            let suno_copy = make_copy_callback(data.suno_prompt.clone(), "Suno prompt", copied_message.clone());
                            let lyrics_copy = make_copy_callback(data.lyrics.clone(), "Lyrics", copied_message.clone());
                            let sora_copy = make_copy_callback(sora_joined.clone(), "Sora prompts", copied_message.clone());
                            let cover_copy = make_copy_callback(data.cover_art_prompt.clone(), "Cover art prompt", copied_message.clone());
                            let linkedin_copy = make_copy_callback(data.linkedin_post.clone(), "LinkedIn post", copied_message.clone());
                            let youtube_copy = make_copy_callback(data.youtube_description.clone(), "YouTube description", copied_message.clone());

                            html! {
                                <div class="section-list">
                                    <div class="output-block">
                                        <div class="row" style="justify-content: space-between; align-items: center;">
                                            <div class="output-title">{ "Title" }</div>
                                            <button class="btn btn-ghost" onclick={title_copy}>{ "Copy" }</button>
                                        </div>
                                        <div class="output-content">{ data.title.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="row" style="justify-content: space-between; align-items: center;">
                                            <div class="output-title">{ "Suno Prompt" }</div>
                                            <button class="btn btn-ghost" onclick={suno_copy}>{ "Copy" }</button>
                                        </div>
                                        <div class="output-content">{ data.suno_prompt.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="row" style="justify-content: space-between; align-items: center;">
                                            <div class="output-title">{ "Lyrics" }</div>
                                            <button class="btn btn-ghost" onclick={lyrics_copy}>{ "Copy" }</button>
                                        </div>
                                        <div class="output-content">{ data.lyrics.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="row" style="justify-content: space-between; align-items: center;">
                                            <div class="output-title">{ "Sora Prompts" }</div>
                                            <button class="btn btn-ghost" onclick={sora_copy}>{ "Copy" }</button>
                                        </div>
                                        <div class="output-content">{ sora_joined }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="row" style="justify-content: space-between; align-items: center;">
                                            <div class="output-title">{ "Cover Art Prompt" }</div>
                                            <button class="btn btn-ghost" onclick={cover_copy}>{ "Copy" }</button>
                                        </div>
                                        <div class="output-content">{ data.cover_art_prompt.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="row" style="justify-content: space-between; align-items: center;">
                                            <div class="output-title">{ "LinkedIn Post" }</div>
                                            <button class="btn btn-ghost" onclick={linkedin_copy}>{ "Copy" }</button>
                                        </div>
                                        <div class="output-content">{ data.linkedin_post.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="row" style="justify-content: space-between; align-items: center;">
                                            <div class="output-title">{ "YouTube Description" }</div>
                                            <button class="btn btn-ghost" onclick={youtube_copy}>{ "Copy" }</button>
                                        </div>
                                        <div class="output-content">{ data.youtube_description.clone() }</div>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="meta">
                                    { "Your generated content pack will appear here." }
                                </div>
                            }
                        }
                    }

                    <div class="footer-note">
                        { "Tip: keep prompts specific. Good inputs include a theme, emotional tone, and target use case." }
                    </div>
                </article>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}