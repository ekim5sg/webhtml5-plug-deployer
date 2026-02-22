use base64::Engine;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use gloo_timers::future::sleep;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

const OWNER: &str = "ekim5sg";
const REPO: &str = "webhtml5-plug-deployer";
const WORKFLOW_FILE: &str = "deploy-hostek-plug.yml"; // .github/workflows/<file>

const PH_TITLE: &str = "{{TITLE}}";
const PH_PLUG: &str = "{{PLUG_NAME}}";
const PH_PUBLIC_URL: &str = "{{PUBLIC_URL}}";
const PH_HOSTEK_URL: &str = "{{HOSTEK_URL}}";

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

/// Lowercase, split non-alnum to words, join by hyphen, collapse repeats.
fn slugify_app_name(app_name: &str) -> Option<String> {
    let raw = app_name.trim().to_lowercase();
    if raw.is_empty() {
        return None;
    }
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in raw.chars() {
        let is_ok = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        if is_ok {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    // trim hyphens
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn hostek_url(plug_name: &str) -> String {
    format!("https://www.webhtml5.info/{}/", plug_name.trim())
}

fn public_url(plug_name: &str) -> String {
    format!("/{}/", plug_name.trim())
}

fn apply_placeholders(content: &str, title: &str, plug: &str) -> String {
    let h = hostek_url(plug);
    let p = public_url(plug);
    content
        .replace(PH_TITLE, title)
        .replace(PH_PLUG, plug)
        .replace(PH_HOSTEK_URL, &h)
        .replace(PH_PUBLIC_URL, &p)
}

// IMPORTANT: r##" .. "## so `content="#0b1020"` is safe.
fn default_index_html_template() -> String {
    r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <meta name="color-scheme" content="dark" />
  <meta name="theme-color" content="#0b1020" />
  <title>{{TITLE}}</title>
  <link data-trunk rel="css" href="styles.css" />
</head>
<body>
  <div id="app"></div>
  <link data-trunk rel="rust" />
</body>
</html>
"##.to_string()
}

fn default_styles_css_template() -> String {
    r#"/* {{TITLE}} — MikeGyver Studio plug
   Deployed at: {{HOSTEK_URL}}
   public_url: {{PUBLIC_URL}}
*/
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

html,body{ height:100%; background:var(--bg0); color:var(--text); margin:0; }
body{ font-family: system-ui, -apple-system, Segoe UI, Roboto, Arial, sans-serif; overflow-x:hidden; }
*{ box-sizing:border-box; }

.bg{
  position:fixed; inset:-20%; z-index:-1;
  background:
    radial-gradient(900px 600px at 15% 10%, rgba(124,92,255,.28), transparent 55%),
    radial-gradient(900px 600px at 85% 15%, rgba(40,215,255,.20), transparent 55%),
    linear-gradient(180deg, var(--bg0), var(--bg1));
}

.wrap{ width:min(980px, calc(100% - 28px)); margin:0 auto; padding:18px 0 74px; }
.card{
  border:1px solid var(--line);
  background:linear-gradient(180deg, rgba(255,255,255,.04), rgba(255,255,255,.02));
  border-radius:18px;
  box-shadow: 0 22px 80px var(--shadow);
  overflow:hidden;
}
.card-h{ padding:16px 16px 0; }
.card-b{ padding:0 16px 16px; }
.badge{
  display:inline-flex; align-items:center; padding:8px 12px;
  border:1px solid var(--line); border-radius:999px; background:rgba(255,255,255,.04);
  font-size:13px; color:var(--muted);
}
.h1{ margin:14px 0 6px; font-size:clamp(28px, 4vw, 44px); line-height:1.08; letter-spacing:-.02em; }
.sub{ margin:0; color:var(--muted); font-size:15px; line-height:1.5; max-width:70ch; }
.btn{
  appearance:none; border:none; border-radius:14px; padding:12px 14px; font-weight:700;
  color:var(--text);
  background:linear-gradient(135deg, rgba(124,92,255,.95), rgba(40,215,255,.70));
  box-shadow: 0 14px 30px rgba(124,92,255,.18);
  cursor:pointer;
}
"#.to_string()
}

fn default_cargo_toml_template(plug_name: &str) -> String {
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

fn default_main_rs_template() -> String {
    // Keep this dead-simple so new plugs build immediately.
    r#"use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
      <>
        <div class="bg" aria-hidden="true"></div>
        <main class="wrap">
          <section class="card">
            <div class="card-h">
              <div class="badge">{ "{{TITLE}}" }</div>
              <h1 class="h1">{ "{{TITLE}}" }</h1>
              <p class="sub">{ "Plug scaffold is live. Replace this content with your real app." }</p>
              <p class="sub">{ "{{HOSTEK_URL}}" }</p>
            </div>
            <div class="card-b">
              <button class="btn" onclick={Callback::from(|_| web_sys::console::log_1(&"Hello from {{PLUG_NAME}}".into()))}>
                { "Click me" }
              </button>
            </div>
          </section>
        </main>
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
"#.to_string()
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

async fn upsert_file(token: &str, path: &str, msg: &str, content: &str) -> Result<(), String> {
    let sha = match github_get_file(token, path).await {
        Ok((sha, _old)) => Some(sha),
        Err(e) if e.starts_with("Not found:") => None,
        Err(_e) => None, // best-effort create
    };
    github_put_file(token, path, msg, content, sha).await
}

async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("No window".to_string())?;
    let navigator = window.navigator();

    // 1) Try modern async Clipboard API
    // In web_sys this is typically: Result<Clipboard, JsValue>
    if let Ok(clipboard) = navigator.clipboard() {
        let promise = clipboard.write_text(text);
        JsFuture::from(promise)
            .await
            .map_err(|_| "Clipboard write failed (async API)".to_string())?;
        return Ok(());
    }

    // 2) Fallback: execCommand("copy") via a temporary textarea (works in more Safari cases)
    let document = window.document().ok_or("No document".to_string())?;
    let body = document.body().ok_or("No document body".to_string())?;

    let el = document
        .create_element("textarea")
        .map_err(|_| "Failed to create textarea".to_string())?;

    let ta: web_sys::HtmlTextAreaElement = el
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .map_err(|_| "Failed to cast textarea".to_string())?;

    ta.set_value(text);

    // keep it off-screen
    let style = ta.style();
    let _ = style.set_property("position", "fixed");
    let _ = style.set_property("left", "-9999px");
    let _ = style.set_property("top", "0");
    let _ = style.set_property("opacity", "0");

    body.append_child(&ta)
        .map_err(|_| "Failed to append textarea".to_string())?;

    ta.focus().ok();
    ta.select();

    let ok = document
        .exec_command("copy")
        .map_err(|_| "execCommand(copy) failed".to_string())?;

    // cleanup
    let _ = body.remove_child(&ta);

    if ok {
        Ok(())
    } else {
        Err("Copy failed (execCommand returned false)".to_string())
    }
}

#[derive(Clone, PartialEq)]
enum EditTab {
    MainRs,
    IndexHtml,
    StylesCss,
    CargoToml,
}

#[function_component(App)]
fn app() -> Html {
    let token = use_state(|| LocalStorage::get::<String>("gh_pat").ok().unwrap_or_default());

    // App Name -> plug-name
    let app_name = use_state(|| "Rust iPhone Compiler".to_string());
    let plug_name = use_state(|| "rust-iphone-compiler".to_string());

    // Editor
    let tab = use_state(|| EditTab::MainRs);
    let main_rs = use_state(|| default_main_rs_template());
    let index_html = use_state(|| default_index_html_template());
    let styles_css = use_state(|| default_styles_css_template());
    let cargo_toml = use_state(|| default_cargo_toml_template("rust-iphone-compiler"));

    // Status/progress
    let status = use_state(|| "".to_string());
    let progress = use_state(|| 0u8);
    let busy = use_state(|| false);

    // Run link/status
    let run_url = use_state(|| "".to_string());
    let run_state = use_state(|| "".to_string());

    let deployed_url = {
        let plug = (*plug_name).clone();
        use_memo(plug, |p| hostek_url(p.trim()))
    };

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

    let on_app_name = {
        let app_name = app_name.clone();
        let plug_name = plug_name.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            app_name.set(v.clone());
            if let Some(slug) = slugify_app_name(&v) {
                plug_name.set(slug);
            }
        })
    };

    let on_load_defaults = {
        let main_rs = main_rs.clone();
        let index_html = index_html.clone();
        let styles_css = styles_css.clone();
        let cargo_toml = cargo_toml.clone();
        let plug_name = plug_name.clone();
        let status = status.clone();
        Callback::from(move |_| {
            main_rs.set(default_main_rs_template());
            index_html.set(default_index_html_template());
            styles_css.set(default_styles_css_template());
            cargo_toml.set(default_cargo_toml_template(&*plug_name));
            status.set("Loaded defaults (with placeholders).".into());
        })
    };

    let on_edit = {
        let tab = tab.clone();
        let main_rs = main_rs.clone();
        let index_html = index_html.clone();
        let styles_css = styles_css.clone();
        let cargo_toml = cargo_toml.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlTextAreaElement>().value();
            match &*tab {
                EditTab::MainRs => main_rs.set(v),
                EditTab::IndexHtml => index_html.set(v),
                EditTab::StylesCss => styles_css.set(v),
                EditTab::CargoToml => cargo_toml.set(v),
            }
        })
    };

    let on_copy_url = {
        let deployed_url = (*deployed_url).clone();
        let status = status.clone();
        Callback::from(move |_| {
            let status = status.clone();
            let url = deployed_url.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match copy_to_clipboard(url.clone()).await {
                    Ok(_) => status.set(format!("Copied URL ✅ {}", url)),
                    Err(e) => status.set(format!("Copy failed: {}", e)),
                }
            });
        })
    };

    let on_build_deploy = {
        let token = token.clone();
        let app_name = app_name.clone();
        let plug_name = plug_name.clone();
        let main_rs = main_rs.clone();
        let index_html = index_html.clone();
        let styles_css = styles_css.clone();
        let cargo_toml = cargo_toml.clone();
        let status = status.clone();
        let progress = progress.clone();
        let busy = busy.clone();
        let run_url = run_url.clone();
        let run_state = run_state.clone();

        Callback::from(move |_| {
            if *busy {
                return;
            }
            let token_v = (*token).clone();
            if token_v.trim().is_empty() {
                status.set("Missing GitHub token.".into());
                return;
            }
            let title = (*app_name).trim().to_string();
            if title.is_empty() {
                status.set("App Name is required.".into());
                return;
            }
            let Some(plug) = sanitize_plug_name(&(*plug_name)) else {
                status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            };

            let b_main = (*main_rs).clone();
            let b_idx = (*index_html).clone();
            let b_css = (*styles_css).clone();
            let b_toml = (*cargo_toml).clone();

            busy.set(true);
            progress.set(1);
            run_url.set("".into());
            run_state.set("".into());
            status.set("Validating + prepping files…".into());

            wasm_bindgen_futures::spawn_local({
                let status = status.clone();
                let progress = progress.clone();
                let busy = busy.clone();
                let run_url = run_url.clone();
                let run_state = run_state.clone();

                async move {
                    let base = format!("plugs/{}", plug);
                    let msg = format!("Rust iPhone Compiler: update plug {}", plug);

                    progress.set(5);
                    status.set("Applying placeholders…".into());

                    let idx_applied = apply_placeholders(&b_idx, &title, &plug);
                    let css_applied = apply_placeholders(&b_css, &title, &plug);
                    let main_applied = apply_placeholders(&b_main, &title, &plug);

                    progress.set(10);
                    status.set("Normalizing Cargo.toml…".into());
                    let pkg_name = plug.replace('-', "_");
                    let mut toml_applied = apply_placeholders(&b_toml, &title, &plug);

                    let mut lines: Vec<String> = toml_applied.lines().map(|l| l.to_string()).collect();
                    let mut in_package = false;
                    for line in lines.iter_mut() {
                        let t = line.trim();
                        if t.starts_with('[') && t.ends_with(']') {
                            in_package = t == "[package]";
                        } else if in_package && t.starts_with("name") {
                            *line = format!("name = \"{}\"", pkg_name);
                            in_package = false;
                        }
                    }
                    toml_applied = lines.join("\n");
                    if !toml_applied.ends_with('\n') {
                        toml_applied.push('\n');
                    }

                    progress.set(15);
                    status.set("Checking existing workflow runs…".into());

                    let before_ids: HashSet<u64> = match fetch_runs(&token_v, 10).await {
                        Ok(list) => list.into_iter().map(|r| r.id).collect(),
                        Err(_) => HashSet::new(),
                    };

                    progress.set(25);
                    status.set("Uploading files to GitHub…".into());

                    let r1 = upsert_file(&token_v, &format!("{}/index.html", base), &msg, &idx_applied).await;
                    let r2 = upsert_file(&token_v, &format!("{}/styles.css", base), &msg, &css_applied).await;
                    let r3 = upsert_file(&token_v, &format!("{}/Cargo.toml", base), &msg, &toml_applied).await;
                    let r4 = upsert_file(&token_v, &format!("{}/src/main.rs", base), &msg, &main_applied).await;

                    if let Err(e) = &r1 { status.set(format!("Upload error: {}", e)); busy.set(false); return; }
                    if let Err(e) = &r2 { status.set(format!("Upload error: {}", e)); busy.set(false); return; }
                    if let Err(e) = &r3 { status.set(format!("Upload error: {}", e)); busy.set(false); return; }
                    if let Err(e) = &r4 { status.set(format!("Upload error: {}", e)); busy.set(false); return; }

                    progress.set(55);
                    status.set("Files uploaded ✅ Dispatching build…".into());

                    let app_dir = base.clone();
                    if let Err(e) = dispatch_workflow(&token_v, &plug, &app_dir).await {
                        status.set(format!("Dispatch error: {}", e));
                        busy.set(false);
                        return;
                    }

                    progress.set(65);
                    status.set("Dispatched ✅ Waiting for run to appear…".into());

                    let mut picked: Option<WorkflowRun> = None;
                    for _ in 0..30 {
                        sleep(Duration::from_secs(2)).await;
                        if let Ok(list) = fetch_runs(&token_v, 10).await {
                            if let Some(r) = list.into_iter().find(|r| !before_ids.contains(&r.id)) {
                                picked = Some(r);
                                break;
                            }
                        }
                    }

                    let Some(run) = picked else {
                        status.set("Dispatched ✅ (Could not detect new run yet). Open Actions to confirm.".into());
                        progress.set(70);
                        busy.set(false);
                        return;
                    };

                    run_url.set(run.html_url.clone());
                    run_state.set(format!(
                        "status: {} • conclusion: {}",
                        run.status.clone().unwrap_or_else(|| "unknown".into()),
                        run.conclusion.clone().unwrap_or_else(|| "—".into())
                    ));

                    progress.set(75);
                    status.set("Run found ✅ Monitoring…".into());

                    let run_id = run.id;
                    for i in 0..60 {
                        sleep(Duration::from_secs(3)).await;

                        if let Ok(list) = fetch_runs(&token_v, 10).await {
                            if let Some(r) = list.into_iter().find(|x| x.id == run_id) {
                                run_url.set(r.html_url.clone());
                                let st = r.status.clone().unwrap_or_else(|| "unknown".into());
                                let conc = r.conclusion.clone().unwrap_or_else(|| "—".into());
                                run_state.set(format!("status: {} • conclusion: {}", st, conc));

                                let p = 75u8.saturating_add((i as u8).min(20));
                                progress.set(p.min(95));

                                if st == "completed" {
                                    if conc == "success" {
                                        progress.set(100);
                                        status.set(format!("SUCCESS ✅ Deployed: {}", hostek_url(&plug)));
                                    } else {
                                        status.set(format!(
                                            "Build finished with conclusion: {}. Open run logs: {}",
                                            conc, r.html_url
                                        ));
                                    }
                                    busy.set(false);
                                    return;
                                }
                            }
                        }
                    }

                    status.set(format!("Still running… Open run logs: {}", *run_url));
                    progress.set(95);
                    busy.set(false);
                }
            });
        })
    };

    let editor_value = match &*tab {
        EditTab::MainRs => (*main_rs).clone(),
        EditTab::IndexHtml => (*index_html).clone(),
        EditTab::StylesCss => (*styles_css).clone(),
        EditTab::CargoToml => (*cargo_toml).clone(),
    };

    html! {
      <>
        <div class="bg" aria-hidden="true"></div>

        <main class="wrap" id="top">
          <section class="card">
            <div class="card-h">
              <div class="badge">{ "Rust iPhone Compiler • Build Rust/Yew/WASM from your phone" }</div>
              <h1 class="h1">{ "Rust iPhone Compiler" }</h1>
              <p class="sub">{ "Fill in your app files → Build + Deploy → Hostek. No laptop required in the carpool lane." }</p>
            </div>
            <div class="card-b">
              <label class="label">{ "GitHub token (PAT) — stored on this device" }</label>
              <input class="input" value={(*token).clone()} oninput={on_token} placeholder="ghp_..." />
              <div class="row">
                <button class="btn btn2" onclick={on_save_token}>{ "Save token" }</button>
                <button class="btn btn2" onclick={on_load_defaults}>{ "Load defaults" }</button>
              </div>

              <div class="hr"></div>

              <label class="label">{ "App Name (used for title + slugifies to plug-name automatically)" }</label>
              <input class="input" value={(*app_name).clone()} oninput={on_app_name} placeholder="Rust iPhone Compiler" />

              <div class="kv">
                <div class="k">
                  <div class="label">{ "plug-name (Hostek directory)" }</div>
                  <div class="value mono">{ (*plug_name).clone() }</div>
                </div>
                <div class="k">
                  <div class="label">{ "Hostek URL" }</div>
                  <div class="value mono">{ (*deployed_url).clone() }</div>
                </div>
              </div>

              <div class="row" style="margin-top:12px;">
                <button class="btn" onclick={on_build_deploy} disabled={*busy}>{ if *busy { "Building…" } else { "Build + Deploy" } }</button>
                <button class="btn btn2" onclick={on_copy_url}>{ "Copy URL" }</button>
                <a class="btn btn2" href={(*deployed_url).clone()} target="_blank">{ "Open App" }</a>
              </div>

              <div class="progress">
                <div style={format!("width:{}%", *progress)}></div>
              </div>

              if !run_url.is_empty() {
                <div class="ok" style="margin-top:10px;">
                  <div>{ "Workflow Run:" }</div>
                  <a class="mono" href={(*run_url).clone()} target="_blank">{ (*run_url).clone() }</a>
                  <div class="mono" style="margin-top:6px;">{ (*run_state).clone() }</div>
                </div>
              }

              <pre class="log">{ (*status).clone() }</pre>
            </div>
          </section>

          <section class="card" style="margin-top:14px;">
            <div class="card-h">
              <h2 class="h2">{ "App files (editable)" }</h2>
              <p class="sub">{ "You can use placeholders: {{TITLE}}, {{PLUG_NAME}}, {{PUBLIC_URL}}, {{HOSTEK_URL}}. They are auto-applied right before Build + Deploy." }</p>
              <div class="tabs">
                <button class={classes!("tab", (*tab == EditTab::MainRs).then_some("active"))}
                  onclick={{
                    let tab = tab.clone();
                    Callback::from(move |_| tab.set(EditTab::MainRs))
                  }}>{ "main.rs" }</button>

                <button class={classes!("tab", (*tab == EditTab::IndexHtml).then_some("active"))}
                  onclick={{
                    let tab = tab.clone();
                    Callback::from(move |_| tab.set(EditTab::IndexHtml))
                  }}>{ "index.html" }</button>

                <button class={classes!("tab", (*tab == EditTab::StylesCss).then_some("active"))}
                  onclick={{
                    let tab = tab.clone();
                    Callback::from(move |_| tab.set(EditTab::StylesCss))
                  }}>{ "styles.css" }</button>

                <button class={classes!("tab", (*tab == EditTab::CargoToml).then_some("active"))}
                  onclick={{
                    let tab = tab.clone();
                    Callback::from(move |_| tab.set(EditTab::CargoToml))
                  }}>{ "Cargo.toml" }</button>
              </div>
            </div>
            <div class="card-b">
              <textarea class="ta" value={editor_value} oninput={on_edit} placeholder="// edit here…"></textarea>
            </div>
          </section>

          <div class="footer">
            <span>{ "webhtml5.info • Hostek plug deployer" }</span>
            <a class="backtop" href="#top">{ "↑" }</a>
          </div>
        </main>
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}