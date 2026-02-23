use base64::Engine;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

// ✅ NEW imports (for clipboard + reflection fallback)
use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};

const OWNER: &str = "ekim5sg";
const REPO: &str = "webhtml5-plug-deployer";
const WORKFLOW_FILE: &str = "deploy-hostek-plug.yml"; // .github/workflows/<file>

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

#[derive(Deserialize, Debug, Clone)]
struct RunsResp {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Deserialize, Debug, Clone)]
struct WorkflowRun {
    id: u64,
    html_url: String,
    name: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
    created_at: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ContentGetResp {
    sha: String,
    content: Option<String>,
    encoding: Option<String>,
}

#[derive(Serialize)]
struct PutContentBody<'a> {
    message: &'a str,
    content: String, // base64
    branch: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
}

fn b64_encode(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn b64_decode(s: &str) -> Result<String, String> {
    let cleaned = s.replace('\n', "");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cleaned.as_bytes())
        .map_err(|e| format!("base64 decode failed: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("utf8 decode failed: {}", e))
}

fn iso_short(s: &Option<String>) -> String {
    s.as_deref()
        .unwrap_or("")
        .replace('T', " ")
        .replace('Z', "")
}

// IMPORTANT: r##" .. "## so `content="#0b1020"` does not terminate raw strings.
fn scaffold_index_html(title: &str) -> String {
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
  <div id="app"></div>
  <link data-trunk rel="rust" />
</body>
</html>
"##,
        title
    )
}

fn scaffold_styles_css() -> String {
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
button, input, select, textarea{ font:inherit; }

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
  padding:18px 0 90px;
}

.badge{
  display:inline-flex;
  align-items:center;
  gap:10px;
  padding:8px 12px;
  border:1px solid var(--line);
  border-radius:999px;
  background:rgba(255,255,255,.04);
  box-shadow: 0 18px 60px var(--shadow);
  font-size:13px;
  color:var(--muted);
}

.h1{
  margin:14px 0 6px;
  font-size:clamp(28px, 4vw, 44px);
  line-height:1.08;
  letter-spacing:-.02em;
}

.h2{
  margin:0 0 6px;
  font-size:18px;
  letter-spacing:-.01em;
}

.sub{
  margin:0;
  color:var(--muted);
  font-size:15px;
  line-height:1.5;
  max-width:70ch;
}

.grid{
  display:grid;
  gap:14px;
  grid-template-columns: 1fr;
  margin-top:16px;
}
@media (min-width: 860px){
  .grid{ grid-template-columns: 1.1fr .9fr; }
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

.row{
  display:flex;
  gap:10px;
  flex-wrap:wrap;
  align-items:center;
}

.btn{
  appearance:none;
  border:none;
  border-radius:14px;
  padding:12px 14px;
  font-weight:700;
  color:var(--text);
  background:linear-gradient(135deg, rgba(124,92,255,.95), rgba(40,215,255,.70));
  box-shadow: 0 14px 30px rgba(124,92,255,.18);
  cursor:pointer;
  transform: translateZ(0);
}
.btn:active{ transform: scale(.99); }

.btn2{
  background:rgba(255,255,255,.05);
  border:1px solid var(--line);
  box-shadow:none;
}

.input, .select, .ta{
  width:100%;
  margin-top:6px;
  padding:12px;
  border-radius:14px;
  border:1px solid rgba(255,255,255,.10);
  background:rgba(0,0,0,.25);
  color:var(--text);
  outline:none;
}

.select{
  appearance:none;
}

.ta{
  min-height: 320px;
  resize: vertical;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 13px;
  line-height: 1.4;
}

.kv{
  display:grid;
  grid-template-columns: 1fr;
  gap:10px;
}
@media (min-width: 700px){
  .kv{ grid-template-columns: 1fr 1fr; }
}

.k{
  padding:12px;
  border:1px solid var(--line);
  border-radius:16px;
  background:rgba(255,255,255,.03);
}
.k .label{ color:var(--muted); font-size:12px; }
.k .value{ margin-top:4px; font-size:14px; }

.log{
  white-space:pre-wrap;
  margin-top:12px;
  color:var(--muted);
}

.warn{
  margin-top:10px;
  padding:10px 12px;
  border-radius:14px;
  border:1px solid rgba(255,209,102,.35);
  background:rgba(255,209,102,.08);
  color:var(--muted);
}

.runs{
  display:flex;
  flex-direction:column;
  gap:10px;
}

.run{
  padding:12px;
  border:1px solid var(--line);
  border-radius:16px;
  background:rgba(255,255,255,.03);
}

.run-top{
  display:flex;
  justify-content:space-between;
  gap:10px;
  flex-wrap:wrap;
}

.mono{ font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }

.footer{
  margin-top:18px;
  color:var(--muted);
  font-size:13px;
  display:flex;
  justify-content:space-between;
  gap:10px;
  flex-wrap:wrap;
}

.backtop{
  position:fixed;
  right:14px;
  bottom:14px;
  padding:11px 12px;
  border-radius:999px;
  border:1px solid var(--line);
  background:rgba(10,14,28,.72);
  color:var(--text);
  backdrop-filter: blur(10px);
  box-shadow: 0 20px 80px var(--shadow);
}
"#.to_string()
}

fn scaffold_cargo_toml(plug_name: &str) -> String {
    let pkg = plug_name.replace('-', "_");
    format!(
        r#"[package]
name = "{pkg}"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = {{ version = "0.21", features = ["csr"] }}
"#,
        pkg = pkg
    )
}

fn scaffold_main_rs(title: &str, plug_name: &str) -> String {
    let url = format!("https://www.webhtml5.info/{}/", plug_name);

    // inside format!(): literal braces must be doubled
    format!(
        r#"use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {{
    html! {{
      <>
        <main style="font-family:system-ui; padding:24px;">
          <h1>{title}</h1>
          <p style="color:#667;">{{"Plug scaffold is live. Replace this content with your real app."}}</p>
          <p>{url}</p>
        </main>
      </>
    }}
}}

fn main() {{
    yew::Renderer::<App>::new().render();
}}
"#,
        title = format!("{:?}", title),
        url = format!("{:?}", url),
    )
}

async fn dispatch_workflow(token: &str, plug_name: &str, app_dir: &str) -> Result<(), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
        OWNER, REPO, WORKFLOW_FILE
    );

    let body = DispatchBody {
        git_ref: "main",
        inputs: DispatchInputs {
            plug_name,
            app_dir,
            clean_remote: "false",
        },
    };

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "webhtml5-rust-iphone-compiler")
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

async fn fetch_runs(token: &str, per_page: u32) -> Result<Vec<WorkflowRun>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs?per_page={}",
        OWNER, REPO, WORKFLOW_FILE, per_page
    );

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "webhtml5-rust-iphone-compiler")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch runs failed: {} {}", status, text));
    }

    let json = resp.json::<RunsResp>().await.map_err(|e| e.to_string())?;
    Ok(json.workflow_runs)
}

async fn github_get_file(token: &str, path_str: &str) -> Result<(String, String), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path_str
    );

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "webhtml5-rust-iphone-compiler")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 404 {
        return Err(format!("Not found: {}", path_str));
    }
    if !resp.ok() {
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("GET failed {}: {}", st, text));
    }

    let json = resp.json::<ContentGetResp>().await.map_err(|e| e.to_string())?;
    let sha = json.sha;
    let content = json.content.unwrap_or_default();
    let decoded = b64_decode(&content)?;
    Ok((sha, decoded))
}

async fn github_put_file(
    token: &str,
    path_str: &str,
    message: &str,
    content: &str,
    sha: Option<String>,
) -> Result<(), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path_str
    );

    let body = PutContentBody {
        message,
        content: b64_encode(content),
        branch: "main",
        sha,
    };

    let resp = Request::put(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "webhtml5-rust-iphone-compiler")
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
        Err(format!("PUT {} failed: {} {}", path_str, status, text))
    }
}

fn sanitize_plug_name(s: &str) -> Option<String> {
    let p = s.trim();
    if p.is_empty() {
        return None;
    }
    if p.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        Some(p.to_string())
    } else {
        None
    }
}

//
// ✅ FIXED copy_to_clipboard: Clipboard API returns Promise (not Result) + reflection fallback
//
async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "window not available".to_string())?;

    // 1) Modern Clipboard API (preferred) — write_text returns Promise
    {
        let clipboard = window.navigator().clipboard();
        let promise = clipboard.write_text(text);
        if JsFuture::from(promise).await.is_ok() {
            return Ok(());
        }
        // if it fails, fall back
    }

    // 2) Fallback: hidden textarea + document["execCommand"]("copy") via reflection
    let document = window
        .document()
        .ok_or_else(|| "document not available".to_string())?;

    let body = document
        .body()
        .ok_or_else(|| "document.body not available".to_string())?;

    let ta_el = document
        .create_element("textarea")
        .map_err(|e| format!("create_element failed: {:?}", e))?
        .dyn_into::<HtmlTextAreaElement>()
        .map_err(|_| "failed to cast textarea element".to_string())?;

    ta_el.set_value(text);

    ta_el
        .set_attribute("readonly", "")
        .map_err(|e| format!("set_attribute readonly failed: {:?}", e))?;
    ta_el
        .set_attribute(
            "style",
            "position:fixed;left:-9999px;top:0;opacity:0;pointer-events:none;",
        )
        .map_err(|e| format!("set_attribute style failed: {:?}", e))?;

    body.append_child(&ta_el)
        .map_err(|e| format!("append_child failed: {:?}", e))?;

    ta_el.focus().ok();
    ta_el.select();

    let ok = Reflect::get(&document, &JsValue::from_str("execCommand"))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
        .and_then(|f| f.call1(&document, &JsValue::from_str("copy")).ok())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let _ = body.remove_child(&ta_el);

    if ok {
        Ok(())
    } else {
        Err("Copy failed (clipboard unavailable + execCommand failed)".to_string())
    }
}

#[function_component(App)]
fn app() -> Html {
    let token = use_state(|| LocalStorage::get::<String>("gh_pat").ok().unwrap_or_default());

    // deploy existing plug
    let plug_name = use_state(|| "rust-iphone-compiler".to_string());
    let status = use_state(|| "".to_string());
    let busy = use_state(|| false);

    // create + deploy
    let new_plug = use_state(|| "my-new-plug".to_string());
    let new_title = use_state(|| "My New Plug".to_string());
    let create_status = use_state(|| "".to_string());
    let create_busy = use_state(|| false);

    // runs
    let runs = use_state(|| Vec::<WorkflowRun>::new());
    let runs_err = use_state(|| "".to_string());
    let runs_busy = use_state(|| false);

    // quick editor
    let edit_plug = use_state(|| "rust-iphone-compiler".to_string());
    let edit_file = use_state(|| "src/main.rs".to_string());
    let edit_text = use_state(|| "".to_string());
    let edit_status = use_state(|| "".to_string());
    let edit_busy = use_state(|| false);
    let sha_map = use_state(|| HashMap::<String, String>::new());

    let app_dir = {
        let plug = (*plug_name).clone();
        use_memo(plug, |p| format!("plugs/{}", p.trim()))
    };
    let deployed_url = {
        let plug = (*plug_name).clone();
        use_memo(plug, |p| format!("https://www.webhtml5.info/{}/", p.trim()))
    };

    // token handlers
    let on_token = {
        let token = token.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            token.set(v);
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

    // deploy existing
    let on_plug = {
        let plug_name = plug_name.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            plug_name.set(v);
        })
    };
    let on_deploy = {
        let token = token.clone();
        let plug_name = plug_name.clone();
        let status = status.clone();
        let busy = busy.clone();

        Callback::from(move |_| {
            if *busy {
                return;
            }
            let token = (*token).clone();
            let plug = (*plug_name).clone();

            if token.trim().is_empty() {
                status.set("Missing GitHub token.".into());
                return;
            }
            let Some(plug) = sanitize_plug_name(&plug) else {
                status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            };

            let app_dir = format!("plugs/{}", plug);

            busy.set(true);
            status.set("Dispatching deploy workflow…".into());

            spawn_local({
                let status = status.clone();
                let busy = busy.clone();
                async move {
                    match dispatch_workflow(&token, &plug, &app_dir).await {
                        Ok(_) => status.set(format!(
                            "Workflow dispatched ✅ Deployed URL: https://www.webhtml5.info/{}/",
                            plug
                        )),
                        Err(e) => status.set(format!("Dispatch error: {}", e)),
                    }
                    busy.set(false);
                }
            });
        })
    };

    // refresh runs
    let on_refresh = {
        let token = token.clone();
        let runs = runs.clone();
        let runs_err = runs_err.clone();
        let runs_busy = runs_busy.clone();

        Callback::from(move |_| {
            let token = (*token).clone();
            if token.trim().is_empty() {
                runs_err.set("Enter token first to view workflow runs.".into());
                return;
            }

            runs_busy.set(true);
            runs_err.set("".into());

            spawn_local({
                let runs = runs.clone();
                let runs_err = runs_err.clone();
                let runs_busy = runs_busy.clone();
                async move {
                    match fetch_runs(&token, 10).await {
                        Ok(list) => runs.set(list),
                        Err(e) => runs_err.set(e),
                    }
                    runs_busy.set(false);
                }
            });
        })
    };

    // create + deploy scaffold
    let on_new_plug = {
        let new_plug = new_plug.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            new_plug.set(v);
        })
    };
    let on_new_title = {
        let new_title = new_title.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            new_title.set(v);
        })
    };
    let on_create_and_deploy = {
        let token = token.clone();
        let new_plug = new_plug.clone();
        let new_title = new_title.clone();
        let create_status = create_status.clone();
        let create_busy = create_busy.clone();

        Callback::from(move |_| {
            if *create_busy {
                return;
            }

            let token = (*token).clone();
            let plug = (*new_plug).clone();
            let title = (*new_title).clone();

            if token.trim().is_empty() {
                create_status.set("Missing GitHub token.".into());
                return;
            }
            let Some(plug) = sanitize_plug_name(&plug) else {
                create_status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            };
            if title.trim().is_empty() {
                create_status.set("Title is required.".into());
                return;
            }

            create_busy.set(true);
            create_status.set("Creating/overwriting scaffold files…".into());

            spawn_local({
                let create_status = create_status.clone();
                let create_busy = create_busy.clone();
                async move {
                    let base = format!("plugs/{}", plug);
                    let msg = format!("Add plug scaffold: {}", plug);

                    async fn upsert(
                        token: &str,
                        path: &str,
                        msg: &str,
                        content: &str,
                    ) -> Result<(), String> {
                        let sha = match github_get_file(token, path).await {
                            Ok((sha, _old)) => Some(sha),
                            Err(e) if e.starts_with("Not found:") => None,
                            Err(_) => None,
                        };
                        github_put_file(token, path, msg, content, sha).await
                    }

                    let idx = scaffold_index_html(title.trim());
                    let css = scaffold_styles_css();
                    let toml = scaffold_cargo_toml(&plug);
                    let mainrs = scaffold_main_rs(title.trim(), &plug);

                    let r1 = upsert(&token, &format!("{}/index.html", base), &msg, &idx).await;
                    let r2 = upsert(&token, &format!("{}/styles.css", base), &msg, &css).await;
                    let r3 = upsert(&token, &format!("{}/Cargo.toml", base), &msg, &toml).await;
                    let r4 = upsert(&token, &format!("{}/src/main.rs", base), &msg, &mainrs).await;

                    match (r1, r2, r3, r4) {
                        (Ok(_), Ok(_), Ok(_), Ok(_)) => {
                            create_status.set("Files created ✅ Dispatching deploy workflow…".into());
                            let app_dir = format!("plugs/{}", plug);
                            match dispatch_workflow(&token, &plug, &app_dir).await {
                                Ok(_) => create_status.set(format!(
                                    "Workflow dispatched ✅ URL: https://www.webhtml5.info/{}/",
                                    plug
                                )),
                                Err(e) => create_status.set(format!("Dispatch error: {}", e)),
                            }
                        }
                        (a, b, c, d) => {
                            let mut errs = vec![];
                            if let Err(e) = a { errs.push(e); }
                            if let Err(e) = b { errs.push(e); }
                            if let Err(e) = c { errs.push(e); }
                            if let Err(e) = d { errs.push(e); }
                            create_status.set(format!("Create error:\n{}", errs.join("\n")));
                        }
                    }

                    create_busy.set(false);
                }
            });
        })
    };

    // editor controls
    let on_edit_plug = {
        let edit_plug = edit_plug.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            edit_plug.set(v);
        })
    };

    let on_edit_file = {
        let edit_file = edit_file.clone();
        Callback::from(move |e: Event| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            edit_file.set(v);
        })
    };

    let on_edit_text = {
        let edit_text = edit_text.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlTextAreaElement>().value();
            edit_text.set(v);
        })
    };

    let editor_path = {
        let p = (*edit_plug).clone();
        let f = (*edit_file).clone();
        use_memo((p, f), |(plug, file)| format!("plugs/{}/{}", plug.trim(), file))
    };

    let on_load_file = {
        let token = token.clone();
        let edit_plug = edit_plug.clone();
        let edit_text = edit_text.clone();
        let edit_status = edit_status.clone();
        let edit_busy = edit_busy.clone();
        let sha_map = sha_map.clone();
        let edit_file = edit_file.clone();

        Callback::from(move |_| {
            if *edit_busy {
                return;
            }
            let token = (*token).clone();
            if token.trim().is_empty() {
                edit_status.set("Missing GitHub token.".into());
                return;
            }

            let Some(plug) = sanitize_plug_name(&(*edit_plug)) else {
                edit_status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            };

            let file = (*edit_file).clone();
            let path = format!("plugs/{}/{}", plug, file);

            edit_busy.set(true);
            edit_status.set(format!("Loading {}…", path));

            spawn_local({
                let edit_text = edit_text.clone();
                let edit_status = edit_status.clone();
                let edit_busy = edit_busy.clone();
                let sha_map = sha_map.clone();
                async move {
                    match github_get_file(&token, &path).await {
                        Ok((sha, txt)) => {
                            edit_text.set(txt);
                            let mut m = (*sha_map).clone();
                            m.insert(path.clone(), sha);
                            sha_map.set(m);
                            edit_status.set(format!("Loaded ✅ {}", path));
                        }
                        Err(e) => edit_status.set(format!("Load error: {}", e)),
                    }
                    edit_busy.set(false);
                }
            });
        })
    };

    let on_save_file = {
        let token = token.clone();
        let edit_plug = edit_plug.clone();
        let edit_file = edit_file.clone();
        let edit_text = edit_text.clone();
        let edit_status = edit_status.clone();
        let edit_busy = edit_busy.clone();
        let sha_map = sha_map.clone();

        Callback::from(move |_| {
            if *edit_busy {
                return;
            }
            let token = (*token).clone();
            if token.trim().is_empty() {
                edit_status.set("Missing GitHub token.".into());
                return;
            }

            let Some(plug) = sanitize_plug_name(&(*edit_plug)) else {
                edit_status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            };

            let file = (*edit_file).clone();
            let path = format!("plugs/{}/{}", plug, file);

            let text = (*edit_text).clone();
            let msg = format!("Edit {} via rust-iphone-compiler", path);

            edit_busy.set(true);
            edit_status.set(format!("Saving {}…", path));

            spawn_local({
                let edit_status = edit_status.clone();
                let edit_busy = edit_busy.clone();
                let sha_map = sha_map.clone();
                async move {
                    let sha = (*sha_map).get(&path).cloned();

                    let sha = match sha {
                        Some(s) => Some(s),
                        None => match github_get_file(&token, &path).await {
                            Ok((sha, _)) => Some(sha),
                            Err(e) if e.starts_with("Not found:") => None,
                            Err(_) => None,
                        },
                    };

                    match github_put_file(&token, &path, &msg, &text, sha).await {
                        Ok(_) => {
                            match github_get_file(&token, &path).await {
                                Ok((new_sha, _)) => {
                                    let mut m = (*sha_map).clone();
                                    m.insert(path.clone(), new_sha);
                                    sha_map.set(m);
                                }
                                Err(_) => {}
                            }
                            edit_status.set(format!("Saved ✅ {}", path));
                        }
                        Err(e) => edit_status.set(format!("Save error: {}", e)),
                    }

                    edit_busy.set(false);
                }
            });
        })
    };

    // ✅ FIX: onclick needs MouseEvent callback, but on_save_file is Callback<()>
    let on_save_file_click = {
        let on_save_file = on_save_file.clone();
        Callback::from(move |_| {
            on_save_file.emit(());
        })
    };

    let on_save_and_deploy = {
        let on_save_file = on_save_file.clone();
        let token = token.clone();
        let edit_plug = edit_plug.clone();
        let status = status.clone();
        let busy = busy.clone();

        Callback::from(move |_| {
            on_save_file.emit(());

            let token = (*token).clone();
            let plug = (*edit_plug).clone();
            if token.trim().is_empty() {
                status.set("Missing GitHub token.".into());
                return;
            }
            let Some(plug) = sanitize_plug_name(&plug) else {
                status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            };

            if *busy {
                return;
            }

            let app_dir = format!("plugs/{}", plug);
            busy.set(true);
            status.set("Dispatching deploy after save…".into());

            spawn_local({
                let status = status.clone();
                let busy = busy.clone();
                async move {
                    match dispatch_workflow(&token, &plug, &app_dir).await {
                        Ok(_) => status.set(format!(
                            "Workflow dispatched ✅ Deployed URL: https://www.webhtml5.info/{}/",
                            plug
                        )),
                        Err(e) => status.set(format!("Dispatch error: {}", e)),
                    }
                    busy.set(false);
                }
            });
        })
    };

    html! {
      <>
        <div class="bg" aria-hidden="true"></div>

        <main class="wrap" id="top">
          <section class="card">
            <div class="card-h">
              <div class="badge">{ "Rust iPhone Compiler • Mission Control" }</div>
              <h1 class="h1">{ "Deploy & edit plugs from your phone" }</h1>
              <p class="sub">{ "This triggers GitHub Actions builds and deploys to Hostek — no local Rust compilation on iPhone." }</p>
              <p class="sub" style="margin-top:10px; max-width:none;">{ "GitHub token (PAT) — stored on this device" }</p>
            </div>
            <div class="card-b">
              <input class="input" value={(*token).clone()} oninput={on_token} placeholder="ghp_..." />
              <div class="row" style="margin-top:10px;">
                <button class="btn btn2" onclick={on_save_token}>{ "Save token" }</button>
                <button class="btn btn2" onclick={on_refresh} disabled={*runs_busy}>{ if *runs_busy { "Refreshing…" } else { "Refresh runs" } }</button>
              </div>
            </div>
          </section>

          <div class="grid">
            <section class="card">
              <div class="card-h">
                <h2 class="h2">{ "Quick Edit (iPhone) + Deploy" }</h2>
                <p class="sub">{ "Load a plug file from GitHub, edit, save, and deploy." }</p>
              </div>
              <div class="card-b">
                <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "plug_name" }</label>
                <input class="input" value={(*edit_plug).clone()} oninput={on_edit_plug} placeholder="my-new-plug" />

                <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "file" }</label>
                <select class="select input" onchange={on_edit_file} value={(*edit_file).clone()}>
                  <option value="src/main.rs">{ "src/main.rs" }</option>
                  <option value="styles.css">{ "styles.css" }</option>
                  <option value="index.html">{ "index.html" }</option>
                  <option value="Cargo.toml">{ "Cargo.toml" }</option>
                </select>

                <div class="kv" style="margin-top:10px;">
                  <div class="k">
                    <div class="label">{ "editor path" }</div>
                    <div class="value mono">{ (*editor_path).clone() }</div>
                  </div>
                  <div class="k">
                    <div class="label">{ "tip" }</div>
                    <div class="value">{ "Tap Load before editing so SHA is tracked." }</div>
                  </div>
                </div>

                <div class="row" style="margin-top:12px;">
                  <button class="btn btn2" onclick={on_load_file} disabled={*edit_busy}>{ if *edit_busy { "Loading…" } else { "Load" } }</button>

                  // ✅ FIXED: Save button uses MouseEvent wrapper
                  <button class="btn btn2" onclick={on_save_file_click} disabled={*edit_busy}>
                    { if *edit_busy { "Saving…" } else { "Save" } }
                  </button>

                  <button class="btn" onclick={on_save_and_deploy} disabled={*edit_busy || *busy}>{ "Save + Deploy" }</button>
                </div>

                <textarea class="ta" value={(*edit_text).clone()} oninput={on_edit_text} placeholder="// edit here…"></textarea>
                <pre class="log">{ (*edit_status).clone() }</pre>
              </div>
            </section>

            <section class="card">
              <div class="card-h">
                <h2 class="h2">{ "Create a new plug + deploy" }</h2>
                <p class="sub">{ "Creates/overwrites scaffold files, then deploys." }</p>
              </div>
              <div class="card-b">
                <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "new plug_name" }</label>
                <input class="input" value={(*new_plug).clone()} oninput={on_new_plug} />

                <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "title" }</label>
                <input class="input" value={(*new_title).clone()} oninput={on_new_title} />

                <button class="btn" onclick={on_create_and_deploy} disabled={*create_busy} style="margin-top:12px;">
                  { if *create_busy { "Working…" } else { "Create + Deploy" } }
                </button>

                <pre class="log">{ (*create_status).clone() }</pre>
              </div>
            </section>
          </div>

          <section class="card" style="margin-top:14px;">
            <div class="card-h">
              <h2 class="h2">{ "Deploy an existing plug" }</h2>
              <p class="sub">{ "Enter plug_name. app_dir auto-fills as plugs/[plug_name]." }</p>
            </div>
            <div class="card-b">
              <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "plug_name" }</label>
              <input class="input" value={(*plug_name).clone()} oninput={on_plug} />

              <div class="kv" style="margin-top:10px;">
                <div class="k">
                  <div class="label">{ "app_dir" }</div>
                  <div class="value mono">{ (*app_dir).clone() }</div>
                </div>
                <div class="k">
                  <div class="label">{ "deployed URL" }</div>
                  <div class="value mono">{ (*deployed_url).clone() }</div>
                </div>
              </div>

              <button class="btn" onclick={on_deploy} disabled={*busy} style="margin-top:12px;">
                { if *busy { "Working…" } else { "Deploy plug" } }
              </button>

              <pre class="log">{ (*status).clone() }</pre>
            </div>
          </section>

          <section class="card" style="margin-top:14px;">
            <div class="card-h">
              <h2 class="h2">{ "Recent workflow runs" }</h2>
              <p class="sub">{ "Latest runs for deploy-hostek-plug.yml" }</p>
            </div>
            <div class="card-b">
              if !runs_err.is_empty() {
                <div class="warn">{ (*runs_err).clone() }</div>
              }
              <div class="runs">
                { for (*runs).iter().map(|r| {
                    let name = r.name.clone().unwrap_or_else(|| "Run".into());
                    let when = iso_short(&r.created_at);
                    let st = r.status.clone().unwrap_or_default();
                    let conc = r.conclusion.clone().unwrap_or_else(|| "—".into());
                    html! {
                      <a class="run" href={r.html_url.clone()} target="_blank">
                        <div class="run-top">
                          <div class="run-name">{ format!("{} #{}", name, r.id) }</div>
                          <div class="run-meta mono">{ when }</div>
                        </div>
                        <div class="run-meta">{ format!("status: {} • conclusion: {}", st, conc) }</div>
                      </a>
                    }
                }) }
              </div>
            </div>
          </section>

          <div class="footer">
            <span>{ "webhtml5.info • Hostek deployer" }</span>
            <a class="backtop" href="#top">{ "↑" }</a>
          </div>
        </main>
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}