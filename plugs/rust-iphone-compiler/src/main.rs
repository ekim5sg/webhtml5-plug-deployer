use base64::Engine;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

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

// GitHub "contents" API response (subset)
#[derive(Deserialize, Debug)]
struct ContentResp {
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

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn iso_short(s: &Option<String>) -> String {
    s.as_deref()
        .unwrap_or("")
        .replace('T', " ")
        .replace('Z', "")
}

fn make_scaffold_index_html(title: &str) -> String {
    // IMPORTANT: use r##" .. "## because the HTML contains `"#` sequences like content="#0b1020"
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
        title
    )
}

fn make_scaffold_styles_css() -> String {
    r#"/* webhtml5 plug scaffold — MikeGyver Studio dark-mode baseline */
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

html,body{ height:100%; margin:0; background:var(--bg0); color:var(--text); }
body{ font-family: system-ui,-apple-system,Segoe UI,Roboto,Arial,sans-serif; overflow-x:hidden; }

.bg{
  position:fixed; inset:-20%; z-index:-1;
  background:
    radial-gradient(900px 600px at 15% 10%, rgba(124,92,255,.28), transparent 55%),
    radial-gradient(900px 600px at 85% 15%, rgba(40,215,255,.20), transparent 55%),
    linear-gradient(180deg, var(--bg0), var(--bg1));
}

main{
  width:min(980px, calc(100% - 32px));
  margin:0 auto;
  padding:22px 0 64px;
}

.card{
  border:1px solid var(--line);
  background:linear-gradient(180deg, rgba(255,255,255,.04), rgba(255,255,255,.02));
  border-radius:var(--radius);
  box-shadow: 0 22px 80px var(--shadow);
  overflow:hidden;
}

.h{ padding:16px 16px 0; }
.b{ padding:0 16px 16px; }

.btn{
  appearance:none; border:none; cursor:pointer;
  border-radius:14px; padding:12px 14px;
  color:var(--text); font-weight:700;
  background:linear-gradient(135deg, rgba(124,92,255,.95), rgba(40,215,255,.70));
}

.mono{ font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
"#.to_string()
}

fn make_scaffold_cargo_toml(plug_name: &str) -> String {
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

fn make_scaffold_main_rs(title: &str, plug_name: &str) -> String {
    let url = format!("https://www.webhtml5.info/{}/", plug_name);

    // IMPORTANT: this is inside format!(), so any literal { } must be doubled: {{ and }}
    format!(
        r#"use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {{
    html! {{
      <>
        <div class="bg" aria-hidden="true"></div>
        <main>
          <section class="card">
            <div class="h">
              <h1 style="margin:10px 0 6px; font-size:32px; letter-spacing:-.02em;">{title}</h1>
              <p style="margin:0 0 12px; color:var(--muted); line-height:1.5;">
                {{"Plug scaffold is live. Replace this content with your real app."}}
              </p>
            </div>
            <div class="b">
              <div class="mono" style="color:var(--muted);">{url}</div>
            </div>
          </section>
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

async fn github_get_sha(token: &str, path_str: &str) -> Result<Option<String>, String> {
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

    match resp.status() {
        200 => {
            let json = resp.json::<ContentResp>().await.map_err(|e| e.to_string())?;
            Ok(Some(json.sha))
        }
        404 => Ok(None),
        _ => {
            let st = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(format!("GET sha failed: {} {}", st, text))
        }
    }
}

async fn github_put_file(
    token: &str,
    path_str: &str,
    message: &str,
    content: &str,
    overwrite: bool,
) -> Result<(), String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        OWNER, REPO, path_str
    );

    let sha = match github_get_sha(token, path_str).await? {
        Some(existing_sha) => {
            if overwrite {
                Some(existing_sha)
            } else {
                return Err(format!("File exists (overwrite disabled): {}", path_str));
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

#[function_component(App)]
fn app() -> Html {
    let token = use_state(|| LocalStorage::get::<String>("gh_pat").ok().unwrap_or_default());

    // Deploy panel
    let plug_name = use_state(|| "rust-iphone-compiler".to_string());
    let status = use_state(|| "".to_string());
    let busy = use_state(|| false);

    // Create panel
    let new_plug = use_state(|| "my-new-plug".to_string());
    let new_title = use_state(|| "My New Plug".to_string());
    let create_status = use_state(|| "".to_string());
    let create_busy = use_state(|| false);

    // Runs panel
    let runs = use_state(|| Vec::<WorkflowRun>::new());
    let runs_err = use_state(|| "".to_string());
    let runs_busy = use_state(|| false);

    let app_dir = {
        let plug = (*plug_name).clone();
        use_memo(plug, |p| format!("plugs/{}", p.trim()))
    };

    let deployed_url = {
        let plug = (*plug_name).clone();
        use_memo(plug, |p| format!("https://www.webhtml5.info/{}/", p.trim()))
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

    let on_plug = {
        let plug_name = plug_name.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            plug_name.set(v);
        })
    };

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

            wasm_bindgen_futures::spawn_local({
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
            let plug = (*plug_name).trim().to_string();

            if token.trim().is_empty() {
                status.set("Missing GitHub token.".into());
                return;
            }
            if plug.is_empty() || !plug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            }

            let app_dir = format!("plugs/{}", plug);

            busy.set(true);
            status.set("Dispatching deploy workflow…".into());

            wasm_bindgen_futures::spawn_local({
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
            let plug = (*new_plug).trim().to_string();
            let title = (*new_title).trim().to_string();

            if token.trim().is_empty() {
                create_status.set("Missing GitHub token.".into());
                return;
            }
            if plug.is_empty() || !plug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                create_status.set("plug_name must be lowercase letters, numbers, hyphens.".into());
                return;
            }
            if title.is_empty() {
                create_status.set("Title is required.".into());
                return;
            }

            create_busy.set(true);
            create_status.set("Creating/overwriting plug files in GitHub…".into());

            wasm_bindgen_futures::spawn_local({
                let create_status = create_status.clone();
                let create_busy = create_busy.clone();
                async move {
                    let base = format!("plugs/{}", plug);
                    let msg = format!("Add plug scaffold: {}", plug);
                    let overwrite = true; // carpool lane default

                    let idx = make_scaffold_index_html(&title);
                    let css = make_scaffold_styles_css();
                    let toml = make_scaffold_cargo_toml(&plug);
                    let mainrs = make_scaffold_main_rs(&title, &plug);

                    let r1 = github_put_file(&token, &format!("{}/index.html", base), &msg, &idx, overwrite).await;
                    let r2 = github_put_file(&token, &format!("{}/styles.css", base), &msg, &css, overwrite).await;
                    let r3 = github_put_file(&token, &format!("{}/Cargo.toml", base), &msg, &toml, overwrite).await;
                    let r4 = github_put_file(&token, &format!("{}/src/main.rs", base), &msg, &mainrs, overwrite).await;

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
                            create_status.set(format!("Create file error:\n{}", errs.join("\n")));
                        }
                    }

                    create_busy.set(false);
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
              <h1 class="h1">{ "Deploy plugs from your phone" }</h1>
              <p class="sub">{ "This triggers GitHub Actions builds and deploys to Hostek (no local Rust compilation on iPhone)." }</p>
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

          <section class="card" style="margin-top:14px;">
            <div class="card-h">
              <h2 class="h2">{ "Create a new plug + deploy" }</h2>
              <p class="sub">{ "Carpool-lane mode: creates or overwrites scaffold files, then deploys." }</p>
            </div>
            <div class="card-b">
              <p class="sub" style="margin:0 0 6px; max-width:none;">{ "new plug_name" }</p>
              <input class="input" value={(*new_plug).clone()} oninput={on_new_plug} />

              <p class="sub" style="margin:10px 0 6px; max-width:none;">{ "title" }</p>
              <input class="input" value={(*new_title).clone()} oninput={on_new_title} />

              <button class="btn" onclick={on_create_and_deploy} disabled={*create_busy} style="margin-top:12px;">
                { if *create_busy { "Working…" } else { "Create + Deploy" } }
              </button>

              <pre class="log">{ (*create_status).clone() }</pre>
            </div>
          </section>

          <section class="card" style="margin-top:14px;">
            <div class="card-h">
              <h2 class="h2">{ "Deploy an existing plug" }</h2>
              <p class="sub">{ "Enter plug_name. app_dir auto-fills as plugs/[plug_name]." }</p>
            </div>
            <div class="card-b">
              <p class="sub" style="margin:0 0 6px; max-width:none;">{ "plug_name" }</p>
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