use base64::Engine;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
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

    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
}

#[derive(Deserialize)]
struct ContentResp {
    sha: String,
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
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn make_index_html(title: &str) -> String {
    // Use r## to avoid accidental termination when content includes `"#` (e.g., "#0b1020")
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <meta name="color-scheme" content="dark" />
  <meta name="theme-color" content="#0b1020" />
  <title>{}</title>
  <link data-trunk rel="css" href="styles.css" />
</head>
<body id="top">
  <div class="bg" aria-hidden="true"></div>
  <div id="app"></div>
  <link data-trunk rel="rust" />
</body>
</html>
"##,
        title
    )
}

fn make_styles_css() -> String {
    // Starter dark-mode styling for generated plugs (no light sections)
    r#"/* MikeGyver Studio • hard-locked dark mode (no light sections) */
:root{
  --bg0:#070a12;
  --bg1:#0b1020;
  --card:#0f1730;
  --card2:#111c3a;
  --text:#e8ecff;
  --muted:#aab3d6;
  --line:rgba(255,255,255,.10);
  --shadow:rgba(0,0,0,.55);
  --accent:#7c5cff;
  --accent2:#28d7ff;
  --good:#39d98a;
  --warn:#ffd166;
  --danger:#ff5c7a;
  --radius:18px;
}

html,body{
  height:100%;
  background:var(--bg0) !important;
  color:var(--text) !important;
  margin:0;
}

body{
  font-family: system-ui, -apple-system, Segoe UI, Roboto, Arial, sans-serif;
  -webkit-font-smoothing:antialiased;
  -moz-osx-font-smoothing:grayscale;
  overflow-x:hidden;
}

*{ box-sizing:border-box; }
a{ color:inherit; text-decoration:none; }
button, input, select{ font:inherit; }

.bg{
  position:fixed;
  inset:-20%;
  z-index:-1;
  background:
    radial-gradient(900px 600px at 15% 10%, rgba(124,92,255,.28), transparent 55%),
    radial-gradient(900px 600px at 85% 15%, rgba(40,215,255,.20), transparent 55%),
    radial-gradient(900px 700px at 40% 90%, rgba(57,217,138,.12), transparent 60%),
    linear-gradient(180deg, var(--bg0), var(--bg1));
  filter:saturate(115%);
}

.wrap{
  width:min(1100px, calc(100% - 32px));
  margin:0 auto;
  padding:18px 0 64px;
}

.card{
  border:1px solid var(--line);
  background:linear-gradient(180deg, rgba(255,255,255,.04), rgba(255,255,255,.02));
  border-radius:var(--radius);
  box-shadow: 0 22px 80px var(--shadow);
  overflow:hidden;
}

.card-h{ padding:16px 16px 0; }
.card-b{ padding:0 16px 16px; }

.btn{
  appearance:none;
  border:none;
  border-radius:14px;
  padding:12px 14px;
  font-weight:650;
  color:var(--text);
  background:linear-gradient(135deg, rgba(124,92,255,.95), rgba(40,215,255,.70));
  box-shadow: 0 14px 30px rgba(124,92,255,.18);
  cursor:pointer;
}
.btn:active{ transform: scale(.99); }

.btn2{
  background:rgba(255,255,255,.05);
  border:1px solid var(--line);
  box-shadow:none;
}
"#.to_string()
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
        <main class="wrap">
          <section class="card">
            <div class="card-h">
              <h1 style="margin:14px 0 6px; font-size:32px; letter-spacing:-.02em;">{title}</h1>
              <p style="margin:0 0 14px; color:#aab3d6; line-height:1.5;">
                {"Plug scaffold is live. Replace this content with your real app."}
              </p>
            </div>
            <div class="card-b">
              <p style="margin:0; color:#aab3d6;">{url}</p>
            </div>
          </section>
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

async fn github_get_sha(token: &str, path: &str) -> Result<Option<String>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path
    );

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "webhtml5-plug-creator")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 404 {
        return Ok(None);
    }

    if !resp.ok() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("GET {} failed: {} {}", path, status, text));
    }

    let json = resp.json::<ContentResp>().await.map_err(|e| e.to_string())?;
    Ok(Some(json.sha))
}

async fn github_put_file(
    token: &str,
    path: &str,
    message: &str,
    content: &str,
    overwrite: bool,
) -> Result<(), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path
    );

    let sha = match github_get_sha(token, path).await? {
        Some(existing_sha) => {
            if overwrite {
                Some(existing_sha)
            } else {
                return Err(format!("File already exists (overwrite disabled): {}", path));
            }
        }
        None => None,
    };

    let body = PutContentBody {
        message,
        content: b64(content),
        branch: "main",
        sha,
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
    let plug_name = use_state(|| "rust-iphone-compiler".to_string());
    let title = use_state(|| "Rust iPhone Compiler".to_string());
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
            if plug.is_empty()
                || !plug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            }

            busy.set(true);
            status.set("Creating/updating files in GitHub…".into());

            wasm_bindgen_futures::spawn_local({
                let status = status.clone();
                let busy = busy.clone();
                async move {
                    let base = format!("plugs/{}", plug);
                    let msg = format!("Add plug scaffold: {}", plug);

                    // Carpool lane default: overwrite existing files if present
                    let overwrite = true;

                    let idx = make_index_html(&title);
                    let toml = make_cargo_toml(&plug);
                    let mainrs = make_main_rs(&title, &plug);
                    let css = make_styles_css();

                    let r1 = github_put_file(&token, &format!("{}/index.html", base), &msg, &idx, overwrite).await;
                    let r2 = github_put_file(&token, &format!("{}/Cargo.toml", base), &msg, &toml, overwrite).await;
                    let r3 = github_put_file(&token, &format!("{}/src/main.rs", base), &msg, &mainrs, overwrite).await;
                    let r4 = github_put_file(&token, &format!("{}/styles.css", base), &msg, &css, overwrite).await;

                    match (r1, r2, r3, r4) {
                        (Ok(_), Ok(_), Ok(_), Ok(_)) => {
                            status.set("Files created/updated ✅ Dispatching workflow…".into());
                            match github_dispatch(&token, &plug).await {
                                Ok(_) => status.set(format!(
                                    "Workflow dispatched ✅ URL: https://www.webhtml5.info/{}/",
                                    plug
                                )),
                                Err(e) => status.set(format!("Dispatch error: {}", e)),
                            }
                        }
                        (a, b, c, d) => {
                            let mut errs = vec![];
                            if let Err(e) = a { errs.push(e); }
                            if let Err(e) = b { errs.push(e); }
                            if let Err(e) = c { errs.push(e); }
                            if let Err(e) = d { errs.push(e); }
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
          <div class="bg" aria-hidden="true"></div>
          <main class="wrap">
            <section class="card" style="margin-bottom:14px;">
              <div class="card-h">
                <div class="badge">{ "webhtml5 Plug Creator (iPhone mode)" }</div>
                <h1 class="h1">{ "Create + Deploy a new plug" }</h1>
                <p class="sub">{ "Paste token once. Create or overwrite a plug. Dispatches deploy workflow." }</p>
              </div>
              <div class="card-b">
                <label style="display:block; font-size:12px; color:#aab3d6;">{ "GitHub token (PAT) — stored on this device" }</label>
                <input value={(*token).clone()} oninput={on_token}
                  placeholder="ghp_..."
                  style="width:100%; margin-top:6px; padding:12px; border-radius:14px; border:1px solid rgba(255,255,255,.10); background:rgba(0,0,0,.25); color:#e8ecff;" />
                <div class="row" style="margin-top:10px;">
                  <button class="btn btn2" onclick={on_save_token}>{ "Save token" }</button>
                </div>
              </div>
            </section>

            <section class="card">
              <div class="card-h">
                <h2 style="margin:0 0 6px; font-size:18px;">{ "Plug details" }</h2>
                <p class="sub" style="max-width:none;">{ "Default behavior overwrites existing plug files if they already exist." }</p>
              </div>
              <div class="card-b">
                <label style="display:block; font-size:12px; color:#aab3d6;">{ "plug_name (lowercase + hyphens)" }</label>
                <input value={(*plug_name).clone()} oninput={on_plug}
                  style="width:100%; margin-top:6px; padding:12px; border-radius:14px; border:1px solid rgba(255,255,255,.10); background:rgba(0,0,0,.25); color:#e8ecff;" />

                <label style="display:block; margin-top:12px; font-size:12px; color:#aab3d6;">{ "Title" }</label>
                <input value={(*title).clone()} oninput={on_title}
                  style="width:100%; margin-top:6px; padding:12px; border-radius:14px; border:1px solid rgba(255,255,255,.10); background:rgba(0,0,0,.25); color:#e8ecff;" />

                <button class="btn" onclick={on_create} disabled={*busy} style="margin-top:12px;">
                  { if *busy { "Working…" } else { "Create + Deploy" } }
                </button>

                <pre style="white-space:pre-wrap; margin-top:12px; color:#aab3d6;">{ (*status).clone() }</pre>
              </div>
            </section>
          </main>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
