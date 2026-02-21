param(
  [Parameter(Mandatory=$true)]
  [ValidatePattern('^[a-z0-9-]+$')]
  [string]$PlugName,

  [string]$Title = "",
  [string]$Subtitle = "Built in the carpool lane 🚗💨",
  [switch]$CommitAndPush
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$plugDirRel = Join-Path "plugs" $PlugName
$plugDir = Join-Path $repoRoot $plugDirRel

if (-not $Title -or $Title.Trim().Length -eq 0) { $Title = $PlugName }

New-Item -ItemType Directory -Force -Path $plugDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $plugDir "src") | Out-Null

Push-Location $plugDir

if (-not (Test-Path ".\Cargo.toml")) {
  cargo init --bin . | Out-Null
}

@"
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="color-scheme" content="dark" />
    <meta name="theme-color" content="#0b1020" />
    <title>$Title</title>
    <link data-trunk rel="css" href="styles.css" />
  </head>
  <body id="top">
    <div id="app"></div>
    <link data-trunk rel="rust" />
  </body>
</html>
"@ | Set-Content -Encoding UTF8 ".\index.html"

@"
/* MikeGyver Studio • hard-locked dark mode (no light sections) */
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

.hero{
  padding:18px 0 10px;
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

.sub{
  margin:0;
  color:var(--muted);
  font-size:15px;
  line-height:1.5;
  max-width:70ch;
}

.nav{
  display:flex;
  flex-wrap:wrap;
  gap:10px;
  margin-top:14px;
}

.chip{
  padding:10px 12px;
  border-radius:999px;
  border:1px solid var(--line);
  background:rgba(255,255,255,.03);
  color:var(--text);
  font-size:14px;
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

.card-h{
  padding:16px 16px 0;
}

.card-t{
  margin:0 0 6px;
  font-size:18px;
  letter-spacing:-.01em;
}

.card-p{
  margin:0 0 14px;
  color:var(--muted);
  font-size:14px;
  line-height:1.5;
}

.card-b{
  padding:0 16px 16px;
}

.row{
  display:flex;
  gap:10px;
  flex-wrap:wrap;
}

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
  transform: translateZ(0);
}

.btn:active{ transform: scale(.99); }

.btn2{
  background:rgba(255,255,255,.05);
  border:1px solid var(--line);
  box-shadow:none;
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
"@ | Set-Content -Encoding UTF8 ".\styles.css"

@"
[package]
name = "$($PlugName -replace '-', '_')"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
wasm-bindgen = "0.2"
"@ | Set-Content -Encoding UTF8 ".\Cargo.toml"

@"
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct SpecLine {
    k: &'static str,
    v: String,
}

#[function_component(App)]
fn app() -> Html {
    let title = "$Title".to_string();
    let subtitle = "$Subtitle".to_string();

    // "Carpool lane" spec snapshot area — you can paste what you describe here quickly,
    // then later we turn it into real UI + logic.
    let specs = vec![
        SpecLine { k: "Plug name", v: "$PlugName".to_string() },
        SpecLine { k: "Intent", v: "Describe it → ship it (Rust/Yew/WASM)".to_string() },
        SpecLine { k: "Next action", v: "Tell ChatGPT what the app should do; it returns the code delta.".to_string() },
        SpecLine { k: "Deploy URL", v: format!("https://www.webhtml5.info/{}/", "$PlugName") },
    ];

    let on_primary = Callback::from(|_| {
        web_sys::window()
            .and_then(|w| w.alert_with_message("Primary action hooked ✅ (next: replace with real behavior)").ok());
    });

    let on_secondary = Callback::from(|_| {
        web_sys::window()
            .and_then(|w| w.alert_with_message("Secondary action hooked ✅").ok());
    });

    html! {
        <>
          <div class="bg" aria-hidden="true"></div>
          <a id="top"></a>

          <div class="wrap">
            <header class="hero">
              <div class="badge">{ "MikeGyver Studio • webhtml5 plug" }</div>
              <h1 class="h1">{ title }</h1>
              <p class="sub">{ subtitle }</p>

              <nav class="nav" aria-label="Quick navigation">
                <a class="chip" href="#overview">{ "Overview" }</a>
                <a class="chip" href="#actions">{ "Actions" }</a>
                <a class="chip" href="#spec">{ "Spec" }</a>
              </nav>
            </header>

            <section id="overview" class="grid">
              <div class="card">
                <div class="card-h">
                  <h2 class="card-t">{ "Overview" }</h2>
                  <p class="card-p">{ "This starter is hard-locked dark mode, mobile-first, and designed for fast iteration while you describe features on the go." }</p>
                </div>
                <div class="card-b">
                  <div class="kv">
                    { for specs.iter().map(|s| html!{
                      <div class="k">
                        <div class="label">{ s.k }</div>
                        <div class="value">{ s.v.clone() }</div>
                      </div>
                    })}
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-h">
                  <h2 class="card-t" id="actions">{ "Actions" }</h2>
                  <p class="card-p">{ "These are placeholders. Tell me what buttons should do; I’ll wire the logic and UI." }</p>
                </div>
                <div class="card-b">
                  <div class="row">
                    <button class="btn" onclick={on_primary}>{ "Start" }</button>
                    <button class="btn btn2" onclick={on_secondary}>{ "Settings" }</button>
                  </div>
                  <div class="footer">
                    <span>{ "Tip: Keep the spec short; ship often." }</span>
                    <span>{ "Hard-locked dark mode ✅" }</span>
                  </div>
                </div>
              </div>
            </section>

            <section id="spec" class="card" style="margin-top:14px;">
              <div class="card-h">
                <h2 class="card-t">{ "Carpool-lane spec" }</h2>
                <p class="card-p">
                  { "Message format to send me: “Plug: " }{ "$PlugName" }{ " — Users can…, Must…, Buttons…, Data…, Deploy folder…, Done.”" }
                </p>
              </div>
              <div class="card-b">
                <div class="k">
                  <div class="label">{ "Copy/paste this into ChatGPT when you’re in the lane" }</div>
                  <div class="value" style="white-space:pre-wrap;">
{format!(
"Plug: {plug}\nGoal: \nUsers can:\n- \nMust:\n- \nButtons:\n- Primary:\n- Secondary:\nData:\n- \nDeploy:\n- https://www.webhtml5.info/{plug}/\n",
plug = "$PlugName"
)}
                  </div>
                </div>
              </div>
            </section>
          </div>

          <a class="backtop" href="#top" aria-label="Back to top">{ "↑ Top" }</a>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
"@ | Set-Content -Encoding UTF8 ".\src\main.rs"

Pop-Location

Write-Host "✅ Created plug scaffold at: $plugDirRel"
Write-Host "Next: run workflow with plug_name='$PlugName' and app_dir='$plugDirRel'"

if ($CommitAndPush) {
  Push-Location $repoRoot
  git add -A
  git commit -m "Add plug scaffold: $PlugName"
  git push
  Pop-Location
}
