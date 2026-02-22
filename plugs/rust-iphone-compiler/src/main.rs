use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

const OWNER: &str = "ekim5sg";
const REPO: &str = "webhtml5-plug-deployer";
const WORKFLOW_FILE: &str = "deploy-hostek-plug.yml"; // under .github/workflows

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

fn iso_short(s: &Option<String>) -> String {
    s.as_deref()
        .unwrap_or("")
        .replace('T', " ")
        .replace('Z', "")
}

#[function_component(App)]
fn app() -> Html {
    let token = use_state(|| LocalStorage::get::<String>("gh_pat").ok().unwrap_or_default());
    let plug_name = use_state(|| "rust-iphone-compiler".to_string());
    let status = use_state(|| "".to_string());
    let busy = use_state(|| false);

    let runs = use_state(|| Vec::<WorkflowRun>::new());
    let runs_err = use_state(|| "".to_string());
    let runs_busy = use_state(|| false);

    // ✅ Yew 0.21: use_memo(deps, |deps| ...)
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
            if plug.is_empty()
                || !plug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
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

    html! {
      <>
        <div class="bg" aria-hidden="true"></div>
        <main class="wrap" id="top">
          <section class="card">
            <div class="card-h">
              <div class="badge">{ "Rust iPhone Compiler • Mission Control" }</div>
              <h1 class="h1">{ "Deploy plugs from your phone" }</h1>
              <p class="sub">{ "This triggers GitHub Actions builds and deploys to Hostek (no local Rust compilation on iPhone)." }</p>
            </div>

            <div class="card-b">
              <label class="label">{ "GitHub token (PAT) — stored on this device" }</label>
              <input class="input" value={(*token).clone()} oninput={on_token} placeholder="ghp_..." />
              <div class="row">
                <button class="btn btn2" onclick={on_save_token}>{ "Save token" }</button>
                <button class="btn btn2" onclick={on_refresh} disabled={*runs_busy}>{ if *runs_busy { "Refreshing…" } else { "Refresh runs" } }</button>
              </div>
            </div>
          </section>

          <section class="card" style="margin-top:14px;">
            <div class="card-h">
              <h2 class="h2">{ "Deploy a plug" }</h2>
              <p class="sub">{ "Enter plug_name. app_dir auto-fills as plugs/[plug_name]." }</p>
            </div>
            <div class="card-b">
              <label class="label">{ "plug_name" }</label>
              <input class="input" value={(*plug_name).clone()} oninput={on_plug} />

              <div class="kv">
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