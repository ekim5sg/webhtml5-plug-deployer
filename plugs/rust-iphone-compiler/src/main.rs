use base64::Engine;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use gloo_timers::future::TimeoutFuture;
use js_sys::{Function, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

// ====== CONFIG ======
const OWNER: &str = "ekim5sg";
const REPO: &str = "webhtml5-plug-deployer";
const WORKFLOW_FILE: &str = "deploy-hostek-plug.yml"; // .github/workflows/<file>

// ====== GITHUB API TYPES ======
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
    head_branch: Option<String>,
    event: Option<String>,
}

// Minimal response for "contents" API when we only need SHA
#[derive(Deserialize, Debug)]
struct ShaResp {
    sha: String,
}

#[derive(Serialize)]
struct PutContentBody<'a> {
    message: &'a str,
    content: String, // base64
    branch: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
}

// ====== UTIL ======
fn b64_encode(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

// App name -> plug-name (lowercase + hyphen)
fn slugify_app_name(app_name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in app_name.trim().chars() {
        let c = ch.to_ascii_lowercase();
        let is_ok = c.is_ascii_lowercase() || c.is_ascii_digit();

        if is_ok {
            out.push(c);
            prev_dash = false;
        } else if c.is_whitespace() || c == '-' || c == '_' || c == '.' || c == '/' {
            if !out.is_empty() && !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        } else {
            // ignore other characters, but treat as separator
            if !out.is_empty() && !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "my-new-plug".to_string()
    } else {
        out
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

// ====== DEFAULT TEMPLATES (generated into the new plug) ======
fn default_index_html(title: &str) -> String {
    // r## so content="#0b1020" is safe
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
        title.trim()
    )
}

fn default_styles_css() -> String {
    r#"/* MikeGyver Studio • hard-locked dark mode */
:root{
  --bg0:#070a12;
  --bg1:#0b1020;
  --text:#e8ecff;
  --muted:#aab3d6;
  --line:rgba(255,255,255,.10);
  --accent:#7c5cff;
  --accent2:#28d7ff;
  --radius:18px;
}

html,body{ height:100%; background:var(--bg0); color:var(--text); margin:0; }
body{ font-family: system-ui, -apple-system, Segoe UI, Roboto, Arial, sans-serif; }
*{ box-sizing:border-box; }

.bg{
  position:fixed; inset:-20%; z-index:-1;
  background:
    radial-gradient(900px 600px at 15% 10%, rgba(124,92,255,.28), transparent 55%),
    radial-gradient(900px 600px at 85% 15%, rgba(40,215,255,.20), transparent 55%),
    linear-gradient(180deg, var(--bg0), var(--bg1));
}

main{
  width:min(980px, calc(100% - 28px));
  margin:0 auto;
  padding:24px 0 80px;
}

.card{
  border:1px solid var(--line);
  background:rgba(255,255,255,.03);
  border-radius:var(--radius);
  padding:16px;
}

h1{ margin:0 0 8px; letter-spacing:-.02em; }
p{ margin:0; color:var(--muted); line-height:1.45; }
a{ color:var(--text); }
"#
    .to_string()
}

fn default_cargo_toml(plug_name: &str) -> String {
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

fn default_main_rs(title: &str, plug_name: &str) -> String {
    let url = format!("https://www.webhtml5.info/{}/", plug_name);

    format!(
        r#"use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {{
    html! {{
      <>
        <div class="bg" aria-hidden="true"></div>
        <main>
          <div class="card">
            <h1>{}</h1>
            <p>{{"Plug scaffold is live. Replace this content with your real app."}}</p>
            <p style="margin-top:10px;">
              <a href={} target="_blank">{{"Open deployed URL"}}</a>
            </p>
          </div>
        </main>
      </>
    }}
}}

fn main() {{
    yew::Renderer::<App>::new().render();
}}
"#,
        format!("{:?}", title.trim()),
        format!("{:?}", url)
    )
}

// ====== GITHUB API HELPERS ======
fn api_url_contents(path: &str) -> String {
    format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path
    )
}

async fn github_get_file_sha(token: &str, path: &str) -> Result<Option<String>, String> {
    let url = api_url_contents(path);

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

    let json = resp.json::<ShaResp>().await.map_err(|e| e.to_string())?;
    Ok(Some(json.sha))
}

async fn github_put_file(
    token: &str,
    path: &str,
    message: &str,
    content: &str,
    sha: Option<String>,
) -> Result<(), String> {
    let url = api_url_contents(path);

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
        Err(format!("PUT {} failed {}: {}", path, st, text))
    }
}

async fn github_upsert_file(token: &str, path: &str, msg: &str, content: &str) -> Result<(), String> {
    let sha = github_get_file_sha(token, path).await?;
    github_put_file(token, path, msg, content, sha).await
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
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("Dispatch failed {}: {}", st, text))
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
        let st = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Fetch runs failed {}: {}", st, text));
    }

    let json = resp.json::<RunsResp>().await.map_err(|e| e.to_string())?;
    Ok(json.workflow_runs)
}

// ====== CLIPBOARD ======
async fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window".to_string())?;
    let navigator = window.navigator();

    // In your build, clipboard() returns Clipboard directly (not Option/Result)
    let clipboard = navigator.clipboard();
    let promise = clipboard.write_text(text);

    if JsFuture::from(promise).await.is_ok() {
        return Ok(());
    }

    // Fallback: execCommand("copy") via JS Reflect
    let document = window.document().ok_or("No document".to_string())?;
    let body = document.body().ok_or("No document body".to_string())?;

    let el = document
        .create_element("textarea")
        .map_err(|_| "Failed to create textarea".to_string())?;

    let ta: web_sys::HtmlTextAreaElement = el
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .map_err(|_| "Failed to cast textarea".to_string())?;

    ta.set_value(text);
    ta.set_attribute(
        "style",
        "position:fixed;left:-9999px;top:0;opacity:0;pointer-events:none;",
    )
    .map_err(|_| "Failed to set textarea style".to_string())?;

    body.append_child(&ta)
        .map_err(|_| "Failed to append textarea".to_string())?;

    ta.focus().ok();
    ta.select();

    let doc_js: &JsValue = document.as_ref();
    let exec = Reflect::get(doc_js, &JsValue::from_str("execCommand"))
        .map_err(|_| "execCommand not available".to_string())?;

    let ok = if exec.is_function() {
        let f: Function = exec.dyn_into().map_err(|_| "execCommand not a function".to_string())?;
        let result = f
            .call1(doc_js, &JsValue::from_str("copy"))
            .map_err(|_| "execCommand call failed".to_string())?;
        result.as_bool().unwrap_or(false)
    } else {
        false
    };

    let _ = body.remove_child(&ta);

    if ok {
        Ok(())
    } else {
        Err("Copy failed (clipboard + fallback)".to_string())
    }
}

// ====== APP ======
#[function_component(App)]
fn app() -> Html {
    // token
    let token = use_state(|| LocalStorage::get::<String>("gh_pat").ok().unwrap_or_default());
    let token_status = use_state(|| "".to_string());

    // main inputs
    let app_name = use_state(|| "Rust iPhone Compiler Demo".to_string());
    let plug_name = use_state(|| slugify_app_name("Rust iPhone Compiler Demo"));

    // file editors
    let idx = use_state(|| default_index_html("Rust iPhone Compiler Demo"));
    let css = use_state(|| default_styles_css());
    let toml = use_state(|| default_cargo_toml(&slugify_app_name("Rust iPhone Compiler Demo")));
    let mainrs = use_state(|| {
        default_main_rs(
            "Rust iPhone Compiler Demo",
            &slugify_app_name("Rust iPhone Compiler Demo"),
        )
    });

    // deploy status
    let busy = use_state(|| false);
    let progress = use_state(|| 0u8);
    let log = use_state(|| "".to_string());
    let deployed_url = use_state(|| "".to_string());
    let run_url = use_state(|| "".to_string());
    let last_conclusion = use_state(|| "".to_string());

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
        let token_status = token_status.clone();
        Callback::from(move |_| {
            let t = (*token).clone();
            if t.trim().is_empty() {
                token_status.set("Token is empty.".into());
                return;
            }
            let _ = LocalStorage::set("gh_pat", t);
            token_status.set("Saved token to this device (localStorage).".into());
        })
    };

    let on_app_name = {
        let app_name = app_name.clone();
        let plug_name = plug_name.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            app_name.set(v.clone());
            plug_name.set(slugify_app_name(&v));
        })
    };

    let on_regen_templates = {
        let app_name = app_name.clone();
        let plug_name = plug_name.clone();
        let idx = idx.clone();
        let css = css.clone();
        let toml = toml.clone();
        let mainrs = mainrs.clone();

        Callback::from(move |_| {
            let title = (*app_name).clone();
            let plug = (*plug_name).clone();

            idx.set(default_index_html(&title));
            css.set(default_styles_css());
            toml.set(default_cargo_toml(&plug));
            mainrs.set(default_main_rs(&title, &plug));
        })
    };

    // editors
    let on_idx = {
        let idx = idx.clone();
        Callback::from(move |e: InputEvent| {
            idx.set(e.target_unchecked_into::<HtmlTextAreaElement>().value())
        })
    };
    let on_css = {
        let css = css.clone();
        Callback::from(move |e: InputEvent| {
            css.set(e.target_unchecked_into::<HtmlTextAreaElement>().value())
        })
    };
    let on_toml = {
        let toml = toml.clone();
        Callback::from(move |e: InputEvent| {
            toml.set(e.target_unchecked_into::<HtmlTextAreaElement>().value())
        })
    };
    let on_mainrs = {
        let mainrs = mainrs.clone();
        Callback::from(move |e: InputEvent| {
            mainrs.set(e.target_unchecked_into::<HtmlTextAreaElement>().value())
        })
    };

    // Build + Deploy
    let on_build_deploy = {
        let token = token.clone();
        let app_name = app_name.clone();
        let plug_name = plug_name.clone();
        let idx = idx.clone();
        let css = css.clone();
        let toml = toml.clone();
        let mainrs = mainrs.clone();

        let busy = busy.clone();
        let progress = progress.clone();
        let log = log.clone();
        let deployed_url = deployed_url.clone();
        let run_url = run_url.clone();
        let last_conclusion = last_conclusion.clone();

        Callback::from(move |_| {
            if *busy {
                return;
            }

            let token_v = (*token).clone();
            if token_v.trim().is_empty() {
                log.set("Missing GitHub token. Paste it and tap Save token.".into());
                return;
            }

            let plug_v = (*plug_name).clone();
            let Some(plug_ok) = sanitize_plug_name(&plug_v) else {
                log.set("plug-name invalid. Use letters/numbers/hyphens only.".into());
                return;
            };

            let title = (*app_name).clone();
            let idx_v = (*idx).clone();
            let css_v = (*css).clone();
            let toml_v = (*toml).clone();
            let mainrs_v = (*mainrs).clone();

            busy.set(true);
            progress.set(5);
            deployed_url.set(format!("https://www.webhtml5.info/{}/", plug_ok));
            run_url.set("".into());
            last_conclusion.set("".into());
            log.set(format!(
                "Starting build for: {}\nplug-name: {}\n",
                title.trim(),
                plug_ok
            ));

            wasm_bindgen_futures::spawn_local({
                let busy = busy.clone();
                let progress = progress.clone();
                let log = log.clone();
                let deployed_url = deployed_url.clone();
                let run_url = run_url.clone();
                let last_conclusion = last_conclusion.clone();

                async move {
                    let base = format!("plugs/{}", plug_ok);
                    let msg = format!("Rust iPhone Compiler: update {}", plug_ok);

                    progress.set(15);
                    log.set(format!("{}\nUploading files to GitHub…", (*log)));

                    let r1 = github_upsert_file(
                        &token_v,
                        &format!("{}/index.html", base),
                        &msg,
                        &idx_v,
                    )
                    .await;
                    let r2 = github_upsert_file(
                        &token_v,
                        &format!("{}/styles.css", base),
                        &msg,
                        &css_v,
                    )
                    .await;
                    let r3 = github_upsert_file(
                        &token_v,
                        &format!("{}/Cargo.toml", base),
                        &msg,
                        &toml_v,
                    )
                    .await;
                    let r4 = github_upsert_file(
                        &token_v,
                        &format!("{}/src/main.rs", base),
                        &msg,
                        &mainrs_v,
                    )
                    .await;

                    if let Err(e) = r1 {
                        log.set(format!("Upload error:\n{}", e));
                        busy.set(false);
                        return;
                    }
                    if let Err(e) = r2 {
                        log.set(format!("Upload error:\n{}", e));
                        busy.set(false);
                        return;
                    }
                    if let Err(e) = r3 {
                        log.set(format!("Upload error:\n{}", e));
                        busy.set(false);
                        return;
                    }
                    if let Err(e) = r4 {
                        log.set(format!("Upload error:\n{}", e));
                        busy.set(false);
                        return;
                    }

                    progress.set(55);
                    log.set(format!(
                        "{}\nFiles uploaded ✅\nDispatching workflow…",
                        (*log)
                    ));

                    let app_dir = format!("plugs/{}", plug_ok);
                    if let Err(e) = dispatch_workflow(&token_v, &plug_ok, &app_dir).await {
                        log.set(format!("Dispatch error:\n{}", e));
                        busy.set(false);
                        return;
                    }

                    progress.set(70);
                    log.set(format!(
                        "{}\nWorkflow dispatched ✅\nPolling workflow runs…",
                        (*log)
                    ));

                    let mut attempts = 0u32;
                    loop {
                        attempts += 1;

                        match fetch_runs(&token_v, 12).await {
                            Ok(runs) => {
                                let mut picked: Option<WorkflowRun> = None;
                                for r in runs {
                                    let is_main = r.head_branch.as_deref().unwrap_or("") == "main";
                                    let is_dispatch =
                                        r.event.as_deref().unwrap_or("") == "workflow_dispatch";
                                    if is_main && is_dispatch {
                                        picked = Some(r);
                                        break;
                                    }
                                }

                                if let Some(r) = picked {
                                    run_url.set(r.html_url.clone());
                                    let st = r.status.clone().unwrap_or_else(|| "unknown".into());
                                    let conc = r.conclusion.clone().unwrap_or_else(|| "—".into());
                                    last_conclusion.set(conc.clone());

                                    let p = if conc != "—" && !conc.is_empty() {
                                        100
                                    } else if st == "completed" {
                                        100
                                    } else {
                                        let cur = *progress;
                                        cur.saturating_add(5).min(95)
                                    };
                                    progress.set(p);

                                    log.set(format!(
                                        "{}\nRun: {}\nstatus: {} • conclusion: {}",
                                        (*log),
                                        r.id,
                                        st,
                                        conc
                                    ));

                                    if st == "completed" {
                                        progress.set(100);
                                        log.set(format!(
                                            "{}\n\nDone ✅\nOpen: {}",
                                            (*log),
                                            (*deployed_url)
                                        ));
                                        break;
                                    }
                                } else {
                                    log.set(format!("{}\nNo dispatch run found yet…", (*log)));
                                }
                            }
                            Err(e) => {
                                log.set(format!("{}\nPoll error: {}", (*log), e));
                            }
                        }

                        if attempts >= 30 {
                            log.set(format!(
                                "{}\n\nStopped polling (timeout). You can open the run in GitHub and refresh the site.",
                                (*log)
                            ));
                            break;
                        }

                        TimeoutFuture::new(3000).await;
                    }

                    busy.set(false);
                }
            });
        })
    };

    let on_copy_url = {
        let deployed_url = deployed_url.clone();
        let log = log.clone();
        Callback::from(move |_| {
            let url = (*deployed_url).clone();
            if url.trim().is_empty() {
                log.set(format!("{}\nNothing to copy yet.", (*log)));
                return;
            }
            wasm_bindgen_futures::spawn_local({
                let log = log.clone();
                async move {
                    match copy_to_clipboard(&url).await {
                        Ok(_) => log.set(format!("{}\nCopied URL ✅", (*log))),
                        Err(e) => log.set(format!("{}\nCopy failed: {}", (*log), e)),
                    }
                }
            });
        })
    };

    let progress_width = format!("width:{}%;", *progress);

    html! {
      <>
        <div class="bg" aria-hidden="true"></div>

        <main class="wrap" id="top">
          <section class="card">
            <div class="card-h">
              <div class="badge">{ "Rust iPhone Compiler • Build & deploy Yew apps from your phone" }</div>
              <h1 class="h1">{ "Rust iPhone Compiler" }</h1>
              <p class="sub">{ "You paste code on iPhone, tap Build + Deploy, and GitHub Actions compiles + uploads to Hostek." }</p>
            </div>
            <div class="card-b">
              <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "GitHub token (PAT) — stored on this device" }</label>
              <input class="input" value={(*token).clone()} oninput={on_token} placeholder="ghp_..." />
              <div class="row" style="margin-top:10px;">
                <button class="btn btn2" onclick={on_save_token}>{ "Save token" }</button>
              </div>
              if !token_status.is_empty() {
                <pre class="log">{ (*token_status).clone() }</pre>
              }
            </div>
          </section>

          <div class="grid">
            <section class="card">
              <div class="card-h">
                <h2 class="h2">{ "1) App Name → plug-name" }</h2>
                <p class="sub">{ "Enter the app name. plug-name auto-generates (lowercase + hyphens). This is the Hostek folder." }</p>
              </div>
              <div class="card-b">
                <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "App name" }</label>
                <input class="input" value={(*app_name).clone()} oninput={on_app_name} placeholder="My Cool App" />

                <div class="kv" style="margin-top:10px;">
                  <div class="k">
                    <div class="label">{ "plug-name (Hostek folder)" }</div>
                    <div class="value mono">{ (*plug_name).clone() }</div>
                  </div>
                  <div class="k">
                    <div class="label">{ "Hostek URL" }</div>
                    <div class="value mono">{ format!("https://www.webhtml5.info/{}/", (*plug_name).clone()) }</div>
                  </div>
                </div>

                <div class="row" style="margin-top:12px;">
                  <button class="btn btn2" onclick={on_regen_templates}>{ "Generate templates" }</button>
                </div>
              </div>
            </section>

            <section class="card">
              <div class="card-h">
                <h2 class="h2">{ "2) Build + Deploy" }</h2>
                <p class="sub">{ "Uploads files into plugs/[plug-name]/..., triggers the workflow, then polls status." }</p>
              </div>
              <div class="card-b">
                <div class="progress" style="margin-top:6px;">
                  <div style={progress_width}></div>
                </div>

                <div class="row" style="margin-top:12px;">
                  <button class="btn" onclick={on_build_deploy} disabled={*busy}>
                    { if *busy { "Working…" } else { "Build + Deploy" } }
                  </button>
                  <button class="btn btn2" onclick={on_copy_url} disabled={(*deployed_url).is_empty()}>
                    { "Copy URL" }
                  </button>
                  <a class="btn btn2" href={(*deployed_url).clone()} target="_blank" style={ if (*deployed_url).is_empty() { "pointer-events:none;opacity:.55" } else { "" } }>
                    { "Open URL" }
                  </a>
                </div>

                if !run_url.is_empty() {
                  <div class="ok">
                    <div>{ "Workflow run:" }{" "}<a href={(*run_url).clone()} target="_blank">{ (*run_url).clone() }</a></div>
                  </div>
                }

                <pre class="log">{ (*log).clone() }</pre>
              </div>
            </section>
          </div>

          <section class="card" style="margin-top:14px;">
            <div class="card-h">
              <h2 class="h2">{ "3) Paste your files" }</h2>
              <p class="sub">{ "These four textareas are exactly what gets pushed into GitHub under plugs/[plug-name]/" }</p>
            </div>

            <div class="card-b">
              <label class="sub" style="display:block; margin:0 0 6px; max-width:none;">{ "index.html" }</label>
              <textarea class="ta" value={(*idx).clone()} oninput={on_idx}></textarea>

              <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "styles.css" }</label>
              <textarea class="ta" value={(*css).clone()} oninput={on_css}></textarea>

              <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "Cargo.toml" }</label>
              <textarea class="ta" value={(*toml).clone()} oninput={on_toml}></textarea>

              <label class="sub" style="display:block; margin:12px 0 6px; max-width:none;">{ "src/main.rs" }</label>
              <textarea class="ta" value={(*mainrs).clone()} oninput={on_mainrs}></textarea>
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