use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::Serialize;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

const OWNER: &str = "ekim5sg";
const REPO: &str = "webhtml5-plug-deployer";
const WORKFLOW_FILE: &str = "deploy-hostek-plug.yml"; // file name under .github/workflows

#[derive(Serialize)]
struct PutContentBody<'a> {
    message: &'a str,
    content: String, // base64
    branch: &'a str,
}

#[derive(Serialize)]
struct DispatchBody<'a> {
    #[serde(rename = "ref")]
    git_ref: &'a str,
    inputs: DispatchInputs<'a>,
}

#[derive(Serialize)]
struct DispatchInputs<'a> {
    plug_name: &'a str,
    app_dir: &'a str,
    clean_remote: &'a str,
}

fn b64(s: &str) -> String {
    // minimal base64 (no padding concerns in practice)
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn make_index_html(title: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <meta name="color-scheme" content="dark" />
  <meta name="theme-color" content="#0b1020" />
  <title>{}</title>
</head>
<body>
  <div id="app"></div>
  <link data-trunk rel="rust" />
</body>
</html>
"#,
        title
    )
}

fn make_cargo_toml(plug_name: &str) -> String {
    let pkg = plug_name.replace('-', "_");
    format!(
        r#"[package]
name = "{pkg}"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = {{ version = "0.21", features = ["csr"] }}
wasm-bindgen = "0.2"
"#,
        pkg = pkg
    )
}

fn make_main_rs(title: &str, plug_name: &str) -> String {
    format!(
        r#"use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {{
    html! {{
        <main style="font-family: system-ui; padding: 24px;">
            <h1>{title}</h1>
            <p>{url}</p>
        </main>
    }}
}}

fn main() {{
    yew::Renderer::<App>::new().render();
}}
"#,
        title = format!("{:?}", title),
        url = format!("{:?}", format!("https://www.webhtml5.info/{}/", plug_name))
    )
}

async fn github_put_file(token: &str, path: &str, message: &str, content: &str) -> Result<(), String> {
    let url = format!("https://api.github.com/repos/{}/{}/contents/{}", OWNER, REPO, path);

    let body = PutContentBody {
        message,
        content: b64(content),
        branch: "main",
    };

    let resp = Request::put(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "webhtml5-plug-creator")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("PUT {} failed: {} {}", path, status, text))
    }
}

async fn github_dispatch(token: &str, plug_name: &str) -> Result<(), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
        OWNER, REPO, WORKFLOW_FILE
    );

    let app_dir = format!("plugs/{}", plug_name);

    let body = DispatchBody {
        git_ref: "main",
        inputs: DispatchInputs {
            plug_name,
            app_dir: &app_dir,
            clean_remote: "false",
        },
    };

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "webhtml5-plug-creator")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 204 {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("Dispatch failed: {} {}", status, text))
    }
}

#[function_component(App)]
fn app() -> Html {
    let token = use_state(|| LocalStorage::get::<String>("gh_pat").ok().unwrap_or_default());
    let plug_name = use_state(|| "my-new-plug".to_string());
    let title = use_state(|| "My New Plug".to_string());
    let status = use_state(|| "".to_string());
    let busy = use_state(|| false);

    let on_token = {
        let token = token.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            token.set(v);
        })
    };

    let on_plug = {
        let plug_name = plug_name.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            plug_name.set(v);
        })
    };

    let on_title = {
        let title = title.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            title.set(v);
        })
    };

    let on_save_token = {
        let token = token.clone();
        let status = status.clone();
        Callback::from(move |_| {
            let t = (*token).clone();
            if t.trim().is_empty() {
                status.set("Token is empty.".into());
                return;
            }
            let _ = LocalStorage::set("gh_pat", t);
            status.set("Saved token to this device (localStorage).".into());
        })
    };

    let on_create = {
        let token = token.clone();
        let plug_name = plug_name.clone();
        let title = title.clone();
        let status = status.clone();
        let busy = busy.clone();

        Callback::from(move |_| {
            let token = (*token).clone();
            let plug = (*plug_name).trim().to_string();
            let title = (*title).trim().to_string();

            if token.trim().is_empty() {
                status.set("Missing GitHub token.".into());
                return;
            }
            if plug.is_empty() || !plug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            }

            busy.set(true);
            status.set("Creating files in GitHub…".into());

            wasm_bindgen_futures::spawn_local({
                let status = status.clone();
                let busy = busy.clone();
                async move {
                    let base = format!("plugs/{}", plug);

                    let idx = make_index_html(&title);
                    let toml = make_cargo_toml(&plug);
                    let mainrs = make_main_rs(&title, &plug);

                    let msg = format!("Add plug scaffold: {}", plug);

                    let r1 = github_put_file(&token, &format!("{}/index.html", base), &msg, &idx).await;
                    let r2 = github_put_file(&token, &format!("{}/Cargo.toml", base), &msg, &toml).await;
                    let r3 = github_put_file(&token, &format!("{}/src/main.rs", base), &msg, &mainrs).await;

                    match (r1, r2, r3) {
                        (Ok(_), Ok(_), Ok(_)) => {
                            status.set("Files created ✅ Dispatching workflow…".into());
                            match github_dispatch(&token, &plug).await {
                                Ok(_) => status.set(format!("Workflow dispatched ✅ URL: https://www.webhtml5.info/{}/", plug)),
                                Err(e) => status.set(format!("Dispatch error: {}", e)),
                            }
                        }
                        (a, b, c) => {
                            let mut errs = vec![];
                            if let Err(e) = a { errs.push(e); }
                            if let Err(e) = b { errs.push(e); }
                            if let Err(e) = c { errs.push(e); }
                            status.set(format!("Create file error:\n{}", errs.join("\n")));
                        }
                    }

                    busy.set(false);
                }
            });
        })
    };

    html! {
        <>
          <div style="position:fixed; inset:0; background: radial-gradient(900px 600px at 20% 10%, rgba(124,92,255,.28), transparent 55%), linear-gradient(180deg, #070a12, #0b1020); z-index:-1;"></div>
          <div style="max-width:860px; margin:0 auto; padding:18px 16px 70px; color:#e8ecff; font-family:system-ui;">
            <div style="display:inline-block; padding:8px 12px; border:1px solid rgba(255,255,255,.10); border-radius:999px; background:rgba(255,255,255,.04); color:#aab3d6; font-size:13px;">
              { "webhtml5 Plug Creator (iPhone mode)" }
            </div>

            <h1 style="margin:14px 0 6px; font-size:32px; letter-spacing:-.02em;">{ "Create + Deploy a new plug" }</h1>
            <p style="margin:0 0 14px; color:#aab3d6; line-height:1.5;">
              { "Paste a GitHub token once, then type plug name + title. It creates files in the repo and triggers the deploy workflow." }
            </p>

            <div style="border:1px solid rgba(255,255,255,.10); border-radius:18px; padding:14px; background:rgba(255,255,255,.03); margin-bottom:12px;">
              <label style="display:block; font-size:12px; color:#aab3d6;">{ "GitHub token (PAT) — stored on this device" }</label>
              <input value={(*token).clone()} oninput={on_token}
                placeholder="ghp_..."
                style="width:100%; margin-top:6px; padding:12px; border-radius:14px; border:1px solid rgba(255,255,255,.10); background:rgba(0,0,0,.25); color:#e8ecff;" />
              <button onclick={on_save_token}
                style="margin-top:10px; padding:12px 14px; border-radius:14px; border:1px solid rgba(255,255,255,.10); background:rgba(255,255,255,.05); color:#e8ecff;">
                { "Save token" }
              </button>
            </div>

            <div style="border:1px solid rgba(255,255,255,.10); border-radius:18px; padding:14px; background:rgba(255,255,255,.03);">
              <label style="display:block; font-size:12px; color:#aab3d6;">{ "plug_name (lowercase + hyphens)" }</label>
              <input value={(*plug_name).clone()} oninput={on_plug}
                style="width:100%; margin-top:6px; padding:12px; border-radius:14px; border:1px solid rgba(255,255,255,.10); background:rgba(0,0,0,.25); color:#e8ecff;" />

              <label style="display:block; margin-top:12px; font-size:12px; color:#aab3d6;">{ "Title" }</label>
              <input value={(*title).clone()} oninput={on_title}
                style="width:100%; margin-top:6px; padding:12px; border-radius:14px; border:1px solid rgba(255,255,255,.10); background:rgba(0,0,0,.25); color:#e8ecff;" />

              <button onclick={on_create} disabled={*busy}
                style="margin-top:12px; padding:12px 14px; border-radius:14px; border:none; color:#e8ecff; font-weight:700;
                       background:linear-gradient(135deg, rgba(124,92,255,.95), rgba(40,215,255,.70));">
                { if *busy { "Working…" } else { "Create + Deploy" } }
              </button>

              <pre style="white-space:pre-wrap; margin-top:12px; color:#aab3d6;">{ (*status).clone() }</pre>
            </div>
          </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
