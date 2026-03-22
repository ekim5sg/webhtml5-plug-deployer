use gloo_net::http::Request;
use js_sys::{Reflect};
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct MessageData {
    message: String,
}

#[function_component(App)]
fn app() -> Html {
    let loaded_message = use_state(|| None::<String>);
    let current_message = use_state(String::new);
    let is_loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);

    {
        let loaded_message = loaded_message.clone();
        let current_message = current_message.clone();
        let is_loading = is_loading.clone();
        let load_error = load_error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get("message.json").send().await {
                    Ok(response) => match response.json::<MessageData>().await {
                        Ok(data) => {
                            current_message.set(data.message.clone());
                            loaded_message.set(Some(data.message));
                            load_error.set(None);
                        }
                        Err(err) => {
                            load_error.set(Some(format!("Could not parse message.json: {err}")));
                            current_message.set("From Rust and MikeGyver Studio".to_string());
                        }
                    },
                    Err(err) => {
                        load_error.set(Some(format!("Could not load message.json: {err}")));
                        current_message.set("From Rust and MikeGyver Studio".to_string());
                    }
                }

                is_loading.set(false);
            });

            || ()
        });
    }

    let oninput = {
        let current_message = current_message.clone();

        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target() {
                if let Ok(value) = Reflect::get(&target, &JsValue::from_str("value")) {
                    if let Some(text) = value.as_string() {
                        current_message.set(text);
                    }
                }
            }
        })
    };

    let reset_to_original = {
        let loaded_message = loaded_message.clone();
        let current_message = current_message.clone();

        Callback::from(move |_| {
            if let Some(original) = (*loaded_message).clone() {
                current_message.set(original);
            }
        })
    };

    html! {
        <main class="page">
            <section class="card">
                <div class="topline">{ "Rust • Yew • WebAssembly" }</div>

                <h1 class="title">
                    { "Hello World," }
                    <br />
                    <span class="accent">{ (*current_message).clone() }</span>
                </h1>

                <p class="subtitle">
                    { "This demo loads the second part of the greeting from a JSON file. Edit it below to see how Rust + Yew can react instantly in the browser." }
                </p>

                <div class="console">
                    <div class="console-header">
                        <span class="dot red"></span>
                        <span class="dot yellow"></span>
                        <span class="dot green"></span>
                        <span class="console-title">{ "interactive message demo" }</span>
                    </div>

                    <div class="console-body">
                        if *is_loading {
                            <div>
                                <span class="prompt">{ "$ loading message.json..." }</span>
                            </div>
                        } else if let Some(err) = &*load_error {
                            <>
                                <div>
                                    <span class="output">{ "Hello World, From Rust and MikeGyver Studio" }</span>
                                </div>
                                <div style="margin-top: 12px; color: #ffb3b3;">
                                    { err.clone() }
                                </div>
                            </>
                        } else {
                            <>
                                <div>
                                    <span class="prompt">{ "$ message from JSON:" }</span>
                                </div>
                                <div>
                                    <span class="output">
                                        { format!("Hello World, {}", (*current_message).clone()) }
                                    </span>
                                </div>
                            </>
                        }
                    </div>
                </div>

                <div class="editor-panel">
                    <label class="editor-label" for="message-input">
                        { "Change only the second part of the greeting:" }
                    </label>

                    <input
                        id="message-input"
                        class="editor-input"
                        type="text"
                        value={(*current_message).clone()}
                        oninput={oninput}
                        placeholder="From Rust and MikeGyver Studio"
                        disabled={*is_loading}
                    />

                    <div class="signature">
                        <button class="action-button" onclick={reset_to_original} disabled={*is_loading}>
                            { "Reset to JSON value" }
                        </button>
                        <span class="badge">{ "Refresh restores original JSON message" }</span>
                        <span class="badge">{ "Hello World stays fixed" }</span>
                    </div>
                </div>

                <p class="footer-note">
                    { "This gives learners a simple way to experience Rust reactivity without changing compiled code. Edit the text, refresh the page, and the original JSON-driven message returns." }
                </p>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}