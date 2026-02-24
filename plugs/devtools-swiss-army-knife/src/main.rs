// src/main.rs
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use urlencoding::{decode, encode};
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Json,
    Jwt,
    Base64,
    Url,
}

fn tab_label(t: Tab) -> &'static str {
    match t {
        Tab::Json => "JSON",
        Tab::Jwt => "JWT",
        Tab::Base64 => "Base64",
        Tab::Url => "URL",
    }
}

async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let w = window().ok_or("No window".to_string())?;
    let nav = w.navigator();

    // web-sys returns Clipboard (not Option). Browser may still reject if not secure / no gesture.
    let cb = nav.clipboard();
    wasm_bindgen_futures::JsFuture::from(cb.write_text(&text))
        .await
        .map_err(|_| "Clipboard write failed (requires HTTPS + user gesture in many browsers)".to_string())?;

    Ok(())
}

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

#[function_component(App)]
fn app() -> Html {
    let tab = use_state(|| Tab::Json);

    // JSON tab
    let json_in = use_state(|| String::new());
    let json_out = use_state(|| String::new());
    let json_msg = use_state(|| String::new());

    // JWT tab
    let jwt_in = use_state(|| String::new());
    let jwt_header = use_state(|| String::new());
    let jwt_payload = use_state(|| String::new());
    let jwt_msg = use_state(|| String::new());

    // Base64 tab
    let b64_in = use_state(|| String::new());
    let b64_out = use_state(|| String::new());
    let b64_msg = use_state(|| String::new());

    // URL tab
    let url_in = use_state(|| String::new());
    let url_out = use_state(|| String::new());
    let url_msg = use_state(|| String::new());

    let set_tab = {
        let tab = tab.clone();
        Callback::from(move |t: Tab| tab.set(t))
    };

    // JSON actions
    let on_json_pretty = {
        let json_in = json_in.clone();
        let json_out = json_out.clone();
        let json_msg = json_msg.clone();
        Callback::from(move |_| {
            let input = (*json_in).clone();
            match pretty_json(&input) {
                Ok(s) => {
                    json_out.set(s);
                    json_msg.set("Pretty-printed OK.".to_string());
                }
                Err(e) => json_msg.set(e),
            }
        })
    };

    let on_json_minify = {
        let json_in = json_in.clone();
        let json_out = json_out.clone();
        let json_msg = json_msg.clone();
        Callback::from(move |_| {
            let input = (*json_in).clone();
            match minify_json(&input) {
                Ok(s) => {
                    json_out.set(s);
                    json_msg.set("Minified OK.".to_string());
                }
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

    // JWT actions
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
                Err(e) => {
                    jwt_msg.set(format!("Header: {e}"));
                    return;
                }
            }
            match decode_jwt_part(parts[1]) {
                Ok(p) => jwt_payload.set(p),
                Err(e) => {
                    jwt_msg.set(format!("Payload: {e}"));
                    return;
                }
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

    // Base64 actions
    let on_b64_encode = {
        let b64_in = b64_in.clone();
        let b64_out = b64_out.clone();
        let b64_msg = b64_msg.clone();
        Callback::from(move |_| {
            let input = (*b64_in).clone();
            let encoded = base64::engine::general_purpose::STANDARD.encode(input.as_bytes());
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
                    Ok(s) => {
                        b64_out.set(s);
                        b64_msg.set("Decoded OK.".to_string());
                    }
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

    // URL actions
    let on_url_encode = {
        let url_in = url_in.clone();
        let url_out = url_out.clone();
        let url_msg = url_msg.clone();
        Callback::from(move |_| {
            let input = (*url_in).clone();
            url_out.set(encode(&input).to_string());
            url_msg.set("Encoded OK.".to_string());
        })
    };

    let on_url_decode = {
        let url_in = url_in.clone();
        let url_out = url_out.clone();
        let url_msg = url_msg.clone();
        Callback::from(move |_| {
            let input = (*url_in).clone();
            match decode(&input) {
                Ok(s) => {
                    url_out.set(s.to_string());
                    url_msg.set("Decoded OK.".to_string());
                }
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

    let msg_view = |s: &str| -> Html {
        if s.trim().is_empty() {
            html! { <div class="smallnote">{ " " }</div> }
        } else if s.to_lowercase().contains("error")
            || s.to_lowercase().contains("failed")
            || s.to_lowercase().contains("doesn't")
        {
            html! { <div class="alert">{ s }</div> }
        } else {
            html! { <div class="ok">{ s }</div> }
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
                  placeholder="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.something"
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
    };

    html! {
      <div class="app">
        <div class="tabs" role="tablist" aria-label="DevPocket Tabs">
          { for [Tab::Json, Tab::Jwt, Tab::Base64, Tab::Url].into_iter().map(|t| {
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