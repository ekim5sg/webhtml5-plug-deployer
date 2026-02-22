use base64::Engine;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

const OWNER: &str = "ekim5sg";
const REPO: &str = "webhtml5-plug-deployer";
const WORKFLOW_FILE: &str = "deploy-hostek-plug.yml"; // .github/workflows/<file>

const LS_PAT: &str = "gh_pat";
const LS_LAST_RUN_ID: &str = "last_run_id";
const LS_LAST_URL: &str = "last_deployed_url";
const LS_LAST_PLUG: &str = "last_plug_name";

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
    status: Option<String>,
    conclusion: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ContentGetResp {
    sha: String,
    content: Option<String>,
}

#[derive(Serialize)]
struct PutContentBody<'a> {
    message: &'a str,
    content: String, // base64
    branch: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct JobsResp {
    jobs: Vec<Job>,
}

#[derive(Deserialize, Debug, Clone)]
struct Job {
    name: String,
    status: Option<String>,
    conclusion: Option<String>,
    steps: Vec<JobStep>,
}

#[derive(Deserialize, Debug, Clone)]
struct JobStep {
    name: String,
    status: Option<String>,
    conclusion: Option<String>,
}

fn b64_encode(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn b64_decode(s: &str) -> Result<String, String> {
    let cleaned = s.replace('\n', "");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cleaned.as_bytes())
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("utf8 decode failed: {e}"))
}

fn sanitize_slug_from_app_name(app_name: &str) -> Option<String> {
    let s = app_name.trim();
    if s.is_empty() {
        return None;
    }
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in s.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if c.is_whitespace() || c == '-' || c == '_' {
            if !out.is_empty() && !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        } else {
            // ignore other punctuation
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn is_valid_plug_slug(s: &str) -> bool {
    let p = s.trim();
    !p.is_empty()
        && p.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn deployed_url(plug_slug: &str) -> String {
    format!("https://www.webhtml5.info/{}/", plug_slug.trim())
}

// IMPORTANT: use r## so #0b1020 inside HTML doesn’t terminate.
fn default_index_html(title: &str) -> String {
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
<body>
  <div id="app"></div>
  <link data-trunk rel="rust" />
</body>
</html>
"##,
        title.trim()
    )
}

fn default_styles_css() -> String {
    r#"/* MikeGyver Studio • hard-locked dark mode (no light sections) */
:root{
  --bg0:#070a12;
  --bg1:#0b1020;
  --text:#e8ecff;
  --muted:#aab3d6;
  --line:rgba(255,255,255,.10);
  --shadow:rgba(0,0,0,.55);
  --accent:#7c5cff;
  --accent2:#28d7ff;
  --radius:18px;
}
html,body{height:100%;background:var(--bg0)!important;color:var(--text)!important;margin:0;}
body{font-family:system-ui,-apple-system,Segoe UI,Roboto,Arial,sans-serif;-webkit-font-smoothing:antialiased;-moz-osx-font-smoothing:grayscale;overflow-x:hidden;}
*{box-sizing:border-box;}
a{color:inherit;text-decoration:none;}
button,input,select,textarea{font:inherit;}
.bg{
  position:fixed;inset:-20%;z-index:-1;
  background:
    radial-gradient(900px 600px at 15% 10%, rgba(124,92,255,.28), transparent 55%),
    radial-gradient(900px 600px at 85% 15%, rgba(40,215,255,.20), transparent 55%),
    linear-gradient(180deg, var(--bg0), var(--bg1));
  filter:saturate(115%);
}
.wrap{width:min(1100px,calc(100% - 32px));margin:0 auto;padding:18px 0 90px;}
.badge{display:inline-flex;align-items:center;gap:10px;padding:8px 12px;border:1px solid var(--line);border-radius:999px;background:rgba(255,255,255,.04);box-shadow:0 18px 60px var(--shadow);font-size:13px;color:var(--muted);}
.h1{margin:14px 0 6px;font-size:clamp(28px,4vw,44px);line-height:1.08;letter-spacing:-.02em;}
.h2{margin:0 0 6px;font-size:18px;letter-spacing:-.01em;}
.sub{margin:0;color:var(--muted);font-size:15px;line-height:1.5;max-width:70ch;}
.grid{display:grid;gap:14px;grid-template-columns:1fr;margin-top:16px;}
@media (min-width: 860px){.grid{grid-template-columns:1.2fr .8fr;}}
.card{border:1px solid var(--line);background:linear-gradient(180deg,rgba(255,255,255,.04),rgba(255,255,255,.02));border-radius:var(--radius);box-shadow:0 22px 80px var(--shadow);overflow:hidden;}
.card-h{padding:16px 16px 0;}
.card-b{padding:0 16px 16px;}
.row{display:flex;gap:10px;flex-wrap:wrap;align-items:center;}
.btn{appearance:none;border:none;border-radius:14px;padding:12px 14px;font-weight:700;color:var(--text);
  background:linear-gradient(135deg,rgba(124,92,255,.95),rgba(40,215,255,.70));
  box-shadow:0 14px 30px rgba(124,92,255,.18);cursor:pointer;}
.btn:disabled{opacity:.65;cursor:not-allowed;}
.btn2{background:rgba(255,255,255,.05);border:1px solid var(--line);box-shadow:none;font-weight:650;}
.input,.ta{width:100%;margin-top:6px;padding:12px;border-radius:14px;border:1px solid rgba(255,255,255,.10);background:rgba(0,0,0,.25);color:var(--text);outline:none;}
.ta{min-height:200px;resize:vertical;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;font-size:13px;line-height:1.4;}
.kv{display:grid;grid-template-columns:1fr;gap:10px;margin-top:12px;}
@media (min-width:700px){.kv{grid-template-columns:1fr 1fr;}}
.k{padding:12px;border:1px solid var(--line);border-radius:16px;background:rgba(255,255,255,.03);}
.k .label{color:var(--muted);font-size:12px;}
.k .value{margin-top:4px;font-size:14px;}
.log{white-space:pre-wrap;margin-top:12px;color:var(--muted);font-size:13px;}
.warn{margin-top:10px;padding:10px 12px;border-radius:14px;border:1px solid rgba(255,209,102,.35);background:rgba(255,209,102,.08);color:var(--muted);white-space:pre-wrap;}
.bar{
  height:12px;border-radius:999px;border:1px solid var(--line);background:rgba(255,255,255,.04);
  overflow:hidden;margin-top:10px;
}
.bar > div{height:100%;background:linear-gradient(135deg,rgba(124,92,255,.95),rgba(40,215,255,.70));width:0%;}
.footer{margin-top:18px;color:var(--muted);font-size:13px;display:flex;justify-content:space-between;gap:10px;flex-wrap:wrap;}
.backtop{position:fixed;right:14px;bottom:14px;padding:11px 12px;border-radius:999px;border:1px solid var(--line);background:rgba(10,14,28,.72);color:var(--text);backdrop-filter:blur(10px);box-shadow:0 20px 80px var(--shadow);}
"#.to_string()
}

fn default_cargo_toml(crate_name: &str) -> String {
    // crate name must be underscores, not hyphens
    let pkg = crate_name.replace('-', "_");
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

fn default_main_rs(title: &str, plug_slug: &str) -> String {
    let url = deployed_url(plug_slug);
    // format! needs doubled braces for literal braces inside the template
    format!(
        r#"use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {{
    html! {{
        <main style="font-family:system-ui; padding:24px;">
            <h1>{title}</h1>
            <p>{{"Plug scaffold is live. Replace this content with your real app."}}</p>
            <p>{url}</p>
        </main>
    }}
}}

fn main() {{
    yew::Renderer::<App>::new().render();
}}
"#,
        title = format!("{:?}", title.trim()),
        url = format!("{:?}", url),
    )
}

async fn gh_dispatch_workflow(token: &str, plug_slug: &str) -> Result<(), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
        OWNER, REPO, WORKFLOW_FILE
    );

    let app_dir = format!("plugs/{}", plug_slug);

    let body = DispatchBody {
        git_ref: "main",
        inputs: DispatchInputs {
            plug_name: plug_slug,
            app_dir: &app_dir,
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
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("Dispatch failed: {} {}", st, text))
    }
}

async fn gh_fetch_runs(token: &str, per_page: u32) -> Result<Vec<WorkflowRun>, String> {
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
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch runs failed: {} {}", st, text));
    }

    let json = resp.json::<RunsResp>().await.map_err(|e| e.to_string())?;
    Ok(json.workflow_runs)
}

async fn gh_fetch_jobs(token: &str, run_id: u64) -> Result<JobsResp, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
        OWNER, REPO, run_id
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
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch jobs failed: {} {}", st, text));
    }

    resp.json::<JobsResp>().await.map_err(|e| e.to_string())
}

async fn gh_get_file_sha(token: &str, path: &str) -> Result<Option<String>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path
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
        return Ok(None);
    }
    if !resp.ok() {
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("GET sha failed {}: {}", st, text));
    }

    let json = resp.json::<ContentGetResp>().await.map_err(|e| e.to_string())?;
    Ok(Some(json.sha))
}

async fn gh_put_file(
    token: &str,
    path: &str,
    message: &str,
    content: &str,
    sha: Option<String>,
) -> Result<(), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path
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
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("PUT {} failed: {} {}", path, st, text))
    }
}

async fn gh_upsert_file(
    token: &str,
    path: &str,
    message: &str,
    content: &str,
) -> Result<(), String> {
    let sha = match gh_get_file_sha(token, path).await {
        Ok(s) => s,
        Err(_) => None, // best-effort; still try create
    };
    gh_put_file(token, path, message, content, sha).await
}

fn job_progress(jobs: &JobsResp) -> (u32, u32, String) {
    let mut total: u32 = 0;
    let mut done: u32 = 0;
    let mut current = String::new();

    for j in &jobs.jobs {
        for s in &j.steps {
            total += 1;
            if s.status.as_deref() == Some("completed") {
                done += 1;
            } else if current.is_empty() {
                current = format!("{} → {}", j.name, s.name);
            }
        }
    }

    if total == 0 {
        (0, 0, "Waiting for job steps…".into())
    } else if current.is_empty() && done == total {
        (done, total, "Finalizing…".into())
    } else {
        (done, total, current)
    }
}

/// Copy helper:
/// 1) try Clipboard API writeText (promise)
/// 2) fallback to textarea selection + execCommand("copy") if available
async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window".to_string())?;
    let navigator = window.navigator();

    // Clipboard API path
    // NOTE: in web-sys, navigator.clipboard() returns Clipboard (not Result/Option)
    // but it can still fail when calling write_text.
    let clipboard = navigator.clipboard();
    let p = clipboard.write_text(text);
    match JsFuture::from(p).await {
        Ok(_) => return Ok(()),
        Err(_) => {
            // fallback below
        }
    }

    // Fallback path (older browsers)
    let document = window.document().ok_or("No document".to_string())?;
    let ta = document
        .create_element("textarea")
        .map_err(|_| "create_element failed".to_string())?
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .map_err(|_| "dyn_into textarea failed".to_string())?;

    ta.set_value(text);

    // keep offscreen
    let style = ta.style();
    style
        .set_property("position", "fixed")
        .map_err(|_| "style failed".to_string())?;
    style
        .set_property("left", "-10000px")
        .map_err(|_| "style failed".to_string())?;
    style
        .set_property("top", "0")
        .map_err(|_| "style failed".to_string())?;

    let body = document.body().ok_or("No body".to_string())?;
    body.append_child(&ta)
        .map_err(|_| "append failed".to_string())?;

    ta.focus().map_err(|_| "focus failed".to_string())?;
    ta.select();

    // execCommand is deprecated but still widely supported
    let ok = document.exec_command("copy").unwrap_or(false);

    body.remove_child(&ta)
        .map_err(|_| "remove failed".to_string())?;

    if ok {
        Ok(())
    } else {
        Err("Copy failed. Try long-press select and copy.".into())
    }
}

/// Find the run that was created by our dispatch.
/// Strategy:
/// - baseline = highest run id we can see now
/// - after dispatch, poll runs list until we find run id > baseline
async fn wait_for_new_run_id(
    token: &str,
    baseline_run_id: u64,
    timeout_ms: u32,
) -> Result<WorkflowRun, String> {
    let start = js_sys::Date::now();
    let mut backoff_ms: u32 = 1500;

    loop {
        let now = js_sys::Date::now();
        if (now - start) as u32 > timeout_ms {
            return Err("Stopped polling (timeout). Tap Resume Polling.".into());
        }

        let runs = gh_fetch_runs(token, 8).await?;
        if let Some(found) = runs.into_iter().find(|r| r.id > baseline_run_id) {
            return Ok(found);
        }

        TimeoutFuture::new(backoff_ms).await;
        backoff_ms = (backoff_ms as f32 * 1.35) as u32;
        if backoff_ms > 12000 {
            backoff_ms = 12000;
        }
    }
}

/// Poll the run’s jobs/steps for progress until completed.
/// Soft timeout: returns Err(timeout) but preserves run id for Resume.
async fn poll_run_progress(
    token: &str,
    run_id: u64,
    timeout_ms: u32,
    on_update: impl Fn(u8, String, Option<String>, Option<String>) + 'static,
) -> Result<(Option<String>, Option<String>), String> {
    let start = js_sys::Date::now();
    let mut backoff_ms: u32 = 1600;

    loop {
        let now = js_sys::Date::now();
        if (now - start) as u32 > timeout_ms {
            return Err("Stopped polling (timeout). Tap Resume Polling.".into());
        }

        let jobs = gh_fetch_jobs(token, run_id).await?;
        let (done, total, current) = job_progress(&jobs);
        let pct = if total == 0 {
            0
        } else {
            ((done as f32 / total as f32) * 100.0).round() as u8
        };

        // Determine overall state:
        // If all jobs concluded, pick the "worst" conclusion. Otherwise in_progress.
        let mut any_incomplete = false;
        let mut any_failure = false;
        let mut any_cancel = false;

        for j in &jobs.jobs {
            if j.status.as_deref() != Some("completed") {
                any_incomplete = true;
            }
            if j.conclusion.as_deref() == Some("failure") {
                any_failure = true;
            }
            if j.conclusion.as_deref() == Some("cancelled") {
                any_cancel = true;
            }
        }

        let status = if any_incomplete {
            Some("in_progress".to_string())
        } else {
            Some("completed".to_string())
        };

        let conclusion = if any_incomplete {
            None
        } else if any_failure {
            Some("failure".to_string())
        } else if any_cancel {
            Some("cancelled".to_string())
        } else {
            Some("success".to_string())
        };

        on_update(pct.min(100), current, status.clone(), conclusion.clone());

        if status.as_deref() == Some("completed") {
            return Ok((status, conclusion));
        }

        TimeoutFuture::new(backoff_ms).await;
        backoff_ms = (backoff_ms as f32 * 1.25) as u32;
        if backoff_ms > 14000 {
            backoff_ms = 14000;
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    // auth
    let token = use_state(|| LocalStorage::get::<String>(LS_PAT).ok().unwrap_or_default());
    let auth_status = use_state(|| "".to_string());

    // app name -> slug
    let app_name = use_state(|| "Rust iPhone Compiler Demo".to_string());
    let plug_slug = use_state(|| "rust-iphone-compiler-demo".to_string());

    // file editors
    let code_main = use_state(|| default_main_rs("Rust iPhone Compiler Demo", "rust-iphone-compiler-demo"));
    let code_index = use_state(|| default_index_html("Rust iPhone Compiler Demo"));
    let code_css = use_state(|| default_styles_css());
    let code_toml = use_state(|| default_cargo_toml("rust-iphone-compiler-demo"));

    // run tracking/progress
    let busy = use_state(|| false);
    let progress_pct = use_state(|| 0u8);
    let progress_line = use_state(|| "".to_string());
    let run_status = use_state(|| "".to_string());
    let run_conclusion = use_state(|| "".to_string());
    let run_id = use_state(|| LocalStorage::get::<String>(LS_LAST_RUN_ID).ok().and_then(|s| s.parse::<u64>().ok()));
    let run_url = use_state(|| LocalStorage::get::<String>(LS_LAST_URL).ok().unwrap_or_default());
    let log = use_state(|| "".to_string());

    // handlers: token
    let on_token = {
        let token = token.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            token.set(v);
        })
    };
    let on_save_token = {
        let token = token.clone();
        let auth_status = auth_status.clone();
        Callback::from(move |_| {
            let t = (*token).clone();
            if t.trim().is_empty() {
                auth_status.set("Token is empty.".into());
                return;
            }
            let _ = LocalStorage::set(LS_PAT, t);
            auth_status.set("Saved token to this device (localStorage).".into());
        })
    };

    // app name -> slug auto
    let on_app_name = {
        let app_name = app_name.clone();
        let plug_slug = plug_slug.clone();
        let code_main = code_main.clone();
        let code_index = code_index.clone();
        let code_toml = code_toml.clone();

        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            app_name.set(v.clone());

            if let Some(slug) = sanitize_slug_from_app_name(&v) {
                plug_slug.set(slug.clone());

                // helpful: also refresh defaults for new plugs (user can edit after)
                code_index.set(default_index_html(&v));
                code_toml.set(default_cargo_toml(&slug));
                code_main.set(default_main_rs(&v, &slug));
            }
        })
    };

    // editor handlers
    let on_main = {
        let code_main = code_main.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlTextAreaElement>().value();
            code_main.set(v);
        })
    };
    let on_index = {
        let code_index = code_index.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlTextAreaElement>().value();
            code_index.set(v);
        })
    };
    let on_css = {
        let code_css = code_css.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlTextAreaElement>().value();
            code_css.set(v);
        })
    };
    let on_toml = {
        let code_toml = code_toml.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlTextAreaElement>().value();
            code_toml.set(v);
        })
    };

    // Copy URL button
    let on_copy_url = {
        let run_url = run_url.clone();
        let log = log.clone();
        Callback::from(move |_| {
            let url = (*run_url).clone();
            if url.trim().is_empty() {
                log.set("No URL to copy yet.".into());
                return;
            }
            wasm_bindgen_futures::spawn_local({
                let log = log.clone();
                async move {
                    match copy_to_clipboard(&url).await {
                        Ok(_) => log.set("Copied URL ✅".into()),
                        Err(e) => log.set(format!("Copy failed: {}", e)),
                    }
                }
            });
        })
    };

    // Core: Build + Deploy
    let on_build_deploy = {
        let token = token.clone();
        let app_name = app_name.clone();
        let plug_slug = plug_slug.clone();

        let code_main = code_main.clone();
        let code_index = code_index.clone();
        let code_css = code_css.clone();
        let code_toml = code_toml.clone();

        let busy = busy.clone();
        let log = log.clone();
        let progress_pct = progress_pct.clone();
        let progress_line = progress_line.clone();
        let run_status = run_status.clone();
        let run_conclusion = run_conclusion.clone();
        let run_id_state = run_id.clone();
        let run_url = run_url.clone();

        Callback::from(move |_| {
            if *busy {
                return;
            }

            let token = (*token).clone();
            if token.trim().is_empty() {
                log.set("Missing GitHub token.".into());
                return;
            }

            let title = (*app_name).clone();
            let slug = (*plug_slug).clone();
            if !is_valid_plug_slug(&slug) {
                log.set("Invalid plug-name slug. Use App Name field to auto-generate, or ensure lowercase letters/numbers/hyphens.".into());
                return;
            }

            let base = format!("plugs/{}", slug);
            let msg = format!("Rust iPhone Compiler: build {}", slug);

            // Capture content
            let mainrs = (*code_main).clone();
            let idx = (*code_index).clone();
            let css = (*code_css).clone();
            let toml = (*code_toml).clone();

            busy.set(true);
            progress_pct.set(0);
            progress_line.set("Starting…".into());
            run_status.set("".into());
            run_conclusion.set("".into());
            log.set(format!("Preparing repo files for: {}\nplug: {}", title, slug));

            wasm_bindgen_futures::spawn_local({
                let busy = busy.clone();
                let log = log.clone();
                let progress_pct = progress_pct.clone();
                let progress_line = progress_line.clone();
                let run_status = run_status.clone();
                let run_conclusion = run_conclusion.clone();
                let run_id_state = run_id_state.clone();
                let run_url = run_url.clone();

                async move {
                    // 1) baseline run id
                    let baseline = match gh_fetch_runs(&token, 1).await {
                        Ok(list) => list.first().map(|r| r.id).unwrap_or(0),
                        Err(_) => 0,
                    };

                    // 2) upsert files (overwrite-safe via sha)
                    progress_line.set("Uploading files…".into());
                    let r1 = gh_upsert_file(&token, &format!("{}/index.html", base), &msg, &idx).await;
                    let r2 = gh_upsert_file(&token, &format!("{}/styles.css", base), &msg, &css).await;
                    let r3 = gh_upsert_file(&token, &format!("{}/Cargo.toml", base), &msg, &toml).await;
                    let r4 = gh_upsert_file(&token, &format!("{}/src/main.rs", base), &msg, &mainrs).await;

                    let mut errs = vec![];
                    for r in [r1, r2, r3, r4] {
                        if let Err(e) = r {
                            errs.push(e);
                        }
                    }
                    if !errs.is_empty() {
                        log.set(format!("Create/update file error:\n{}", errs.join("\n")));
                        busy.set(false);
                        return;
                    }

                    // 3) dispatch
                    progress_line.set("Dispatching workflow…".into());
                    if let Err(e) = gh_dispatch_workflow(&token, &slug).await {
                        log.set(format!("Dispatch error: {}", e));
                        busy.set(false);
                        return;
                    }

                    // 4) find new run
                    progress_line.set("Finding the run that was created…".into());
                    let run = match wait_for_new_run_id(&token, baseline, 120_000).await {
                        Ok(r) => r,
                        Err(e) => {
                            log.set(format!("{e}\nTip: Refresh runs or resume polling."));
                            busy.set(false);
                            return;
                        }
                    };

                    let rid = run.id;
                    let url = deployed_url(&slug);
                    run_id_state.set(Some(rid));
                    run_url.set(url.clone());

                    let _ = LocalStorage::set(LS_LAST_RUN_ID, rid.to_string());
                    let _ = LocalStorage::set(LS_LAST_URL, url.clone());
                    let _ = LocalStorage::set(LS_LAST_PLUG, slug.clone());

                    log.set(format!(
                        "Run attached ✅\nRun ID: {}\nGitHub run: {}\nDeployed URL: {}",
                        rid, run.html_url, url
                    ));

                    // 5) poll jobs/steps for progress
                    let updater = {
                        let progress_pct = progress_pct.clone();
                        let progress_line = progress_line.clone();
                        let run_status = run_status.clone();
                        let run_conclusion = run_conclusion.clone();
                        move |pct: u8, line: String, st: Option<String>, conc: Option<String>| {
                            progress_pct.set(pct);
                            progress_line.set(line);
                            if let Some(s) = st { run_status.set(s); }
                            if let Some(c) = conc { run_conclusion.set(c); }
                        }
                    };

                    progress_line.set("Polling progress…".into());
                    match poll_run_progress(&token, rid, 1_200_000, updater).await {
                        Ok((_st, conc)) => {
                            let conc = conc.unwrap_or_else(|| "unknown".into());
                            if conc == "success" {
                                log.set(format!("✅ Success!\nDeployed: {}", url));
                            } else {
                                log.set(format!(
                                    "Run completed with conclusion: {}\nOpen GitHub run for full logs if needed.\nDeployed URL: {}",
                                    conc, url
                                ));
                            }
                        }
                        Err(e) => {
                            // soft timeout – allow resume
                            log.set(format!(
                                "{}\nRun ID: {}\nYou can tap Resume Polling, or open the GitHub run link shown above.",
                                e, rid
                            ));
                        }
                    }

                    busy.set(false);
                }
            });
        })
    };

    // Resume polling button (uses saved run id)
    let on_resume = {
        let token = token.clone();
        let run_id_state = run_id.clone();
        let busy = busy.clone();
        let progress_pct = progress_pct.clone();
        let progress_line = progress_line.clone();
        let run_status = run_status.clone();
        let run_conclusion = run_conclusion.clone();
        let log = log.clone();

        Callback::from(move |_| {
            if *busy {
                return;
            }
            let token = (*token).clone();
            if token.trim().is_empty() {
                log.set("Missing GitHub token.".into());
                return;
            }
            let Some(rid) = *run_id_state else {
                log.set("No saved run id. Build + Deploy first.".into());
                return;
            };

            busy.set(true);
            progress_line.set("Resuming polling…".into());
            log.set(format!("Resuming run {}…", rid));

            wasm_bindgen_futures::spawn_local({
                let busy = busy.clone();
                let progress_pct = progress_pct.clone();
                let progress_line = progress_line.clone();
                let run_status = run_status.clone();
                let run_conclusion = run_conclusion.clone();
                let log = log.clone();
                async move {
                    let updater = {
                        let progress_pct = progress_pct.clone();
                        let progress_line = progress_line.clone();
                        let run_status = run_status.clone();
                        let run_conclusion = run_conclusion.clone();
                        move |pct: u8, line: String, st: Option<String>, conc: Option<String>| {
                            progress_pct.set(pct);
                            progress_line.set(line);
                            if let Some(s) = st { run_status.set(s); }
                            if let Some(c) = conc { run_conclusion.set(c); }
                        }
                    };

                    match poll_run_progress(&token, rid, 1_200_000, updater).await {
                        Ok((_st, conc)) => {
                            let conc = conc.unwrap_or_else(|| "unknown".into());
                            log.set(format!("Run complete: {}", conc));
                        }
                        Err(e) => log.set(e),
                    }

                    busy.set(false);
                }
            });
        })
    };

    // UI derived
    let slug_preview = (*plug_slug).clone();
    let url_preview = if is_valid_plug_slug(&slug_preview) {
        deployed_url(&slug_preview)
    } else {
        "".into()
    };
    let pct = *progress_pct;
    let pct_style = format!("width:{}%;", pct.min(100));
    let can_go = !(*run_url).trim().is_empty();

    html! {
      <>
        <div class="bg" aria-hidden="true"></div>

        <main class="wrap" id="top">
          <section class="card">
            <div class="card-h">
              <div class="badge">{ "Rust iPhone Compiler • Build + Deploy from iPhone" }</div>
              <h1 class="h1">{ "Compile Rust Yew WASM on GitHub, deploy to Hostek" }</h1>
              <p class="sub">{ "Enter App Name, edit files, tap Build + Deploy. Progress is tracked by run id + job steps (no GitHub required)." }</p>
            </div>
            <div class="card-b">
              <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "GitHub token (PAT) — stored on this device" }</label>
              <input class="input" value={(*token).clone()} oninput={on_token} placeholder="ghp_..." />
              <div class="row" style="margin-top:10px;">
                <button class="btn btn2" onclick={on_save_token}>{ "Save token" }</button>
                <button class="btn btn2" onclick={on_resume} disabled={*busy}>{ "Resume Polling" }</button>
                <button class="btn btn2" onclick={on_copy_url} disabled={!can_go}>{ "Copy URL" }</button>
                if can_go {
                  <a class="btn btn2" href={(*run_url).clone()} target="_blank">{ "Go to deployed app" }</a>
                }
              </div>
              if !(*auth_status).is_empty() {
                <pre class="log">{ (*auth_status).clone() }</pre>
              }
            </div>
          </section>

          <div class="grid">
            <section class="card">
              <div class="card-h">
                <h2 class="h2">{ "1) App Name → plug-name slug" }</h2>
                <p class="sub">{ "Slug is the Hostek top-level directory. It auto-generates from the App Name." }</p>
              </div>
              <div class="card-b">
                <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "App Name" }</label>
                <input class="input" value={(*app_name).clone()} oninput={on_app_name} placeholder="Rust iPhone Compiler Demo" />

                <div class="kv">
                  <div class="k">
                    <div class="label">{ "plug-name (auto)" }</div>
                    <div class="value">{ slug_preview }</div>
                  </div>
                  <div class="k">
                    <div class="label">{ "Hostek URL" }</div>
                    <div class="value">{ url_preview }</div>
                  </div>
                </div>

                <div class="row" style="margin-top:12px;">
                  <button class="btn" onclick={on_build_deploy} disabled={*busy}>{ if *busy { "Working…" } else { "Build + Deploy" } }</button>
                </div>

                <div class="bar"><div style={pct_style}></div></div>
                <pre class="log">
{ format!(
"Progress: {}%\nCurrent: {}\nRun status: {}\nConclusion: {}\nSaved run id: {}\nSaved URL: {}",
pct,
(*progress_line).clone(),
(*run_status).clone(),
(*run_conclusion).clone(),
match *run_id { Some(x) => x.to_string(), None => "—".into() },
(*run_url).clone()
) }
                </pre>

                <pre class="log">{ (*log).clone() }</pre>
              </div>
            </section>

            <section class="card">
              <div class="card-h">
                <h2 class="h2">{ "2) Edit files" }</h2>
                <p class="sub">{ "These are written into plugs/[plug-name]/ and compiled by your workflow." }</p>
              </div>
              <div class="card-b">
                <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "Cargo.toml" }</label>
                <textarea class="ta" value={(*code_toml).clone()} oninput={on_toml}></textarea>

                <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "index.html" }</label>
                <textarea class="ta" value={(*code_index).clone()} oninput={on_index}></textarea>

                <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "styles.css" }</label>
                <textarea class="ta" value={(*code_css).clone()} oninput={on_css}></textarea>

                <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "src/main.rs" }</label>
                <textarea class="ta" value={(*code_main).clone()} oninput={on_main}></textarea>

                <div class="warn">
{ "Tip: If a build fails, the Jobs/Steps view will usually show the failing step name here — without opening GitHub." }
                </div>
              </div>
            </section>
          </div>

          <div class="footer">
            <span>{ "webhtml5.info • Rust iPhone Compiler" }</span>
            <a class="backtop" href="#top">{ "↑" }</a>
          </div>
        </main>
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}