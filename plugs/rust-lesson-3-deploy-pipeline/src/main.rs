use yew::prelude::*;

struct Chapter {
    id: &'static str,
    title: &'static str,
    audio: &'static str,
    lines: &'static [&'static str],
}

struct SourceFile {
    title: &'static str,
    language: &'static str,
    code: &'static str,
}

const CHAPTERS: [Chapter; 5] = [
    Chapter {
        id: "intro",
        title: "Intro - From Rust Code to Live Website",
        audio: "./assets/audio/lesson3-intro.mp3",
        lines: &[
            "From the MikeGyver Studio...",
            "where imagination becomes software...",
            "and software becomes something the world can actually use...",
            "Welcome back to the Rust iPhone Compiler tutorial series.",
            "In Lesson One...",
            "we discovered that Rust could run directly inside the browser through WebAssembly.",
            "In Lesson Two...",
            "we built interactive applications with Yew...",
            "adding state...",
            "buttons...",
            "and dynamic interfaces.",
            "But now...",
            "we move beyond simply building apps.",
            "Because real software is not finished when it compiles.",
            "Real software ships.",
            "In this lesson...",
            "we will follow the complete journey from source code...",
            "to a live public website.",
            "We will learn how Trunk transforms four simple files into a deployable WebAssembly application.",
            "We will explore GitHub Actions...",
            "automated deployment pipelines...",
            "and why hosting environments like IIS and FTP servers sometimes need different strategies than large modern CDNs.",
            "And perhaps most importantly...",
            "we will learn that deployment engineering is part of software engineering.",
            "Because writing code is only the beginning.",
            "Shipping code...",
            "is where software becomes real.",
            "Cue...",
            "The...",
            "Music...",
        ],
    },
    Chapter {
        id: "part1",
        title: "Part 1 - The Four File Pattern",
        audio: "./assets/audio/lesson3-part1.mp3",
        lines: &[
            "One of the biggest challenges for beginners learning Rust WebAssembly development...",
            "is complexity.",
            "A new developer opens a Rust project...",
            "and suddenly sees dozens of folders...",
            "configuration files...",
            "toolchains...",
            "targets...",
            "and build systems.",
            "And for many people...",
            "that complexity becomes overwhelming before the learning even begins.",
            "So the Rust iPhone Compiler takes a different approach.",
            "A strategic approach.",
            "Instead of starting with an entire ecosystem...",
            "we begin with only four files.",
            "An index.html file...",
            "which defines the web page itself.",
            "A styles.css file...",
            "which controls the visual appearance.",
            "A Cargo.toml file...",
            "which defines the Rust package and dependencies.",
            "And finally...",
            "src/main.rs...",
            "the heart of the Rust application.",
            "Four files.",
            "That is enough to create a real WebAssembly application.",
            "That simplicity matters.",
            "Because beginners do not need to understand every advanced Rust concept on day one.",
            "They need momentum.",
            "They need visible progress.",
            "And they need a mental model they can actually remember.",
            "The four file pattern creates exactly that.",
            "It reduces intimidation...",
            "while still teaching real engineering concepts.",
            "And as the projects grow...",
            "those same four files remain the foundation.",
            "Simple enough for learning.",
            "Powerful enough for production.",
            "That is strategic software education.",
        ],
    },
    Chapter {
        id: "part2",
        title: "Part 2 - What Trunk Actually Does",
        audio: "./assets/audio/lesson3-part2.mp3",
        lines: &[
            "Now that we understand the four file pattern...",
            "the next question becomes...",
            "how does all of this actually turn into a website?",
            "That is where Trunk enters the picture.",
            "Trunk is the build system that connects everything together.",
            "It reads the HTML.",
            "It finds the CSS.",
            "It compiles the Rust source code into WebAssembly.",
            "It generates the JavaScript bridge code required for browsers to communicate with the WASM module.",
            "And finally...",
            "it packages everything into a deployable dist folder.",
            "In other words...",
            "Trunk acts like an automated assembly line.",
            "Without it...",
            "students would need to manually manage WebAssembly compilation...",
            "JavaScript loading...",
            "asset linking...",
            "and browser integration.",
            "That would be extremely difficult for beginners.",
            "But Trunk abstracts that complexity away.",
            "The developer writes Rust.",
            "Trunk handles the plumbing.",
            "And this is one of the most important lessons in modern software engineering...",
            "good tooling changes what becomes possible.",
            "Because powerful tools reduce friction.",
            "And reduced friction allows developers to focus on creativity...",
            "problem solving...",
            "and architecture...",
            "instead of repetitive setup tasks.",
            "This is why build systems matter.",
            "Not because they are glamorous...",
            "but because they remove barriers between ideas...",
            "and execution.",
        ],
    },
    Chapter {
        id: "part3",
        title: "Part 3 - The filehash Equals False Strategy",
        audio: "./assets/audio/lesson3-part3.mp3",
        lines: &[
            "When Trunk builds a WebAssembly application...",
            "it normally generates hashed filenames.",
            "Names like...",
            "app dash A one B two C three dot JS...",
            "or styles dash X Y Z dot CSS.",
            "And in many environments...",
            "that is exactly the correct strategy.",
            "Hashed filenames help browsers know when files change.",
            "They improve cache management.",
            "And they are extremely common in modern cloud deployments.",
            "But software engineering is about context.",
            "Different deployment environments...",
            "have different operational needs.",
            "In this tutorial series...",
            "we deploy through GitHub Actions...",
            "FTP...",
            "and IIS hosting on Hostek.",
            "And in that environment...",
            "predictable filenames provide advantages.",
            "Stable names simplify uploads.",
            "They reduce leftover deployment artifacts.",
            "And they make the build output easier for students to understand.",
            "So we introduced a small...",
            "but powerful workflow improvement.",
            "Before the Trunk build begins...",
            "the GitHub Actions workflow dynamically creates a temporary Trunk dot toml file...",
            "containing only this:",
            "open bracket build close bracket...",
            "filehash equals false.",
            "That single configuration changes the output from unpredictable hashed filenames...",
            "to clean deterministic files like...",
            "styles dot CSS...",
            "application dot JS...",
            "and application underscore BG dot WASM.",
            "A tiny change.",
            "But one that demonstrates an important engineering principle...",
            "sometimes the best solution...",
            "depends entirely on the environment you are deploying into.",
        ],
    },
    Chapter {
        id: "outro",
        title: "Outro - Ship the Lesson",
        audio: "./assets/audio/lesson3-outro.mp3",
        lines: &[
            "Today...",
            "we learned something larger than Rust.",
            "We learned that software is not only about writing code.",
            "It is about understanding systems.",
            "Pipelines.",
            "Automation.",
            "Hosting environments.",
            "Deployment strategies.",
            "And the invisible engineering decisions that allow applications to reliably reach users.",
            "We explored how four simple files can become a live WebAssembly application.",
            "We saw how Trunk automates the assembly process.",
            "And we discovered how even a single configuration option...",
            "can dramatically improve deployment workflows in the right environment.",
            "These are the kinds of lessons that transform experimentation...",
            "into engineering.",
            "And perhaps most importantly...",
            "you are no longer just learning how to build applications.",
            "You are learning how to ship them.",
            "From the MikeGyver Studio...",
            "where ideas become software...",
            "and software becomes reality...",
            "this has been Rust iPhone Compiler...",
            "Lesson Three.",
            "And the journey...",
            "is only beginning.",
        ],
    },
];

const INDEX_HTML_CODE: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <meta name="color-scheme" content="dark" />
  <meta name="theme-color" content="#0b1020" />

  <title>Rust Lesson 3 - Deploy Pipeline</title>

  <link data-trunk rel="css" href="styles.css" />
  <link data-trunk rel="rust" />
</head>
<body>
  <div id="app"></div>
</body>
</html>"##;

const CARGO_TOML_CODE: &str = r#"[package]
name = "rust-lesson-3-deploy-pipeline"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }"#;

const STYLES_CSS_CODE: &str = r##":root {
  --bg: #0b1020;
  --panel: rgba(255,255,255,0.08);
  --panel-strong: rgba(255,255,255,0.13);
  --border: rgba(255,255,255,0.18);
  --text: #f8fafc;
  --muted: #cbd5e1;
  --accent: #38bdf8;
  --accent2: #facc15;
  --green: #22c55e;
  --danger: #fb7185;
}

html,
body {
  margin: 0;
  min-height: 100%;
  background:
    radial-gradient(circle at top left, rgba(56,189,248,0.20), transparent 34%),
    radial-gradient(circle at top right, rgba(250,204,21,0.16), transparent 34%),
    linear-gradient(180deg, #10172a 0%, var(--bg) 70%);
  color: var(--text);
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

* {
  box-sizing: border-box;
}

a {
  color: var(--accent);
}

.app-shell {
  width: min(1100px, calc(100% - 32px));
  margin: 0 auto;
  padding: 28px 0 48px;
}

.hero {
  padding: 34px;
  border: 1px solid var(--border);
  border-radius: 28px;
  background: rgba(255,255,255,0.07);
  box-shadow: 0 24px 80px rgba(0,0,0,0.42);
}

.badge {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  padding: 8px 13px;
  border-radius: 999px;
  background: rgba(56,189,248,0.14);
  border: 1px solid rgba(56,189,248,0.25);
  color: #e0f2fe;
  font-weight: 700;
  font-size: 0.9rem;
}

h1 {
  margin: 18px 0 10px;
  font-size: clamp(2.1rem, 7vw, 4.8rem);
  line-height: 0.95;
  letter-spacing: -0.06em;
}

.hero p {
  max-width: 850px;
  color: var(--muted);
  font-size: 1.12rem;
  line-height: 1.7;
}

.nav {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 24px;
}

.nav a,
.button {
  text-decoration: none;
  color: var(--text);
  background: rgba(255,255,255,0.10);
  border: 1px solid var(--border);
  padding: 10px 14px;
  border-radius: 999px;
  font-weight: 700;
}

.grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 18px;
  margin-top: 22px;
}

.chapter {
  border: 1px solid var(--border);
  border-radius: 24px;
  background: var(--panel);
  padding: 24px;
}

.chapter h2 {
  margin: 0 0 10px;
  font-size: clamp(1.35rem, 4vw, 2rem);
}

.chapter p,
.chapter li {
  color: var(--muted);
  line-height: 1.7;
  font-size: 1rem;
}

.audio-box {
  margin: 16px 0;
  padding: 15px;
  border-radius: 18px;
  background: rgba(0,0,0,0.25);
  border: 1px solid rgba(255,255,255,0.12);
}

.audio-box strong {
  display: block;
  margin-bottom: 10px;
  color: #fff;
}

audio {
  width: 100%;
}

.transcript {
  margin-top: 18px;
}

.transcript p {
  margin: 0 0 12px;
  color: var(--muted);
  line-height: 1.75;
  font-size: 1.02rem;
}

.source-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 18px;
}

.source-card {
  border: 1px solid var(--border);
  border-radius: 22px;
  background: rgba(0,0,0,0.22);
  overflow: hidden;
}

.source-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  padding: 14px 16px;
  background: rgba(255,255,255,0.08);
  border-bottom: 1px solid rgba(255,255,255,0.13);
}

.source-header strong {
  color: #fff;
}

.source-header span {
  color: var(--muted);
  font-size: 0.9rem;
}

.code-panel {
  padding: 16px;
  background: rgba(0,0,0,0.40);
  overflow-x: auto;
}

pre {
  margin: 0;
  color: #e2e8f0;
  font-size: 0.92rem;
  line-height: 1.55;
  white-space: pre;
}

.pipeline {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-top: 18px;
}

.step {
  padding: 16px;
  border-radius: 18px;
  background: var(--panel-strong);
  border: 1px solid var(--border);
  text-align: center;
}

.step-number {
  width: 34px;
  height: 34px;
  display: inline-grid;
  place-items: center;
  margin-bottom: 8px;
  border-radius: 999px;
  background: var(--accent2);
  color: #111827;
  font-weight: 900;
}

.callout {
  margin-top: 20px;
  padding: 18px;
  border-radius: 20px;
  border: 1px solid rgba(34,197,94,0.35);
  background: rgba(34,197,94,0.10);
}

.warning {
  border-color: rgba(251,113,133,0.35);
  background: rgba(251,113,133,0.10);
}

.footer {
  margin-top: 30px;
  text-align: center;
  color: var(--muted);
  opacity: 0.85;
}

@media (max-width: 760px) {
  .hero,
  .chapter {
    padding: 22px;
  }

  .pipeline {
    grid-template-columns: 1fr;
  }

  .nav a {
    width: 100%;
    text-align: center;
  }

  .source-header {
    align-items: flex-start;
    flex-direction: column;
  }
}"##;

const MAIN_RS_CODE: &str = r#"use yew::prelude::*;

// This file powers the Lesson 3 tutorial app.
//
// It renders:
// - the lesson narration text
// - inline MP3 audio players
// - the deployment pipeline explanation
// - full source-code panels for index.html, styles.css, Cargo.toml, and src/main.rs
//
// In the live lesson app, this source-code panel is generated by src/main.rs itself.
// That means the file is both the lesson engine and part of the lesson content."#;

const SOURCE_FILES: [SourceFile; 4] = [
    SourceFile {
        title: "index.html",
        language: "HTML",
        code: INDEX_HTML_CODE,
    },
    SourceFile {
        title: "styles.css",
        language: "CSS",
        code: STYLES_CSS_CODE,
    },
    SourceFile {
        title: "Cargo.toml",
        language: "TOML",
        code: CARGO_TOML_CODE,
    },
    SourceFile {
        title: "src/main.rs",
        language: "Rust",
        code: MAIN_RS_CODE,
    },
];

#[function_component(App)]
fn app() -> Html {
    html! {
        <main class="app-shell">
            <section class="hero" id="top">
                <div class="badge">{ "🦀 Rust iPhone Compiler • Lesson 3" }</div>

                <h1>{ "From Rust Code to Live Website" }</h1>

                <p>
                    { "This lesson teaches the deployment pipeline behind a Rust/Yew WebAssembly app: how four source files become a live website through Trunk, GitHub Actions, and Hostek/IIS hosting." }
                </p>

                <div class="nav">
                    <a href="#intro">{ "Intro" }</a>
                    <a href="#part1">{ "Four Files" }</a>
                    <a href="#part2">{ "Trunk" }</a>
                    <a href="#part3">{ "filehash = false" }</a>
                    <a href="#source-code">{ "Source Code" }</a>
                    <a href="/rust-lesson-3-deploy-pipeline/rust-lesson-3-deploy-pipeline.zip">
                        { "⬇ Download Lesson ZIP" }
                    </a>
                    <a href="#outro">{ "Outro" }</a>
                </div>

                <div class="pipeline">
                    <div class="step">
                        <div class="step-number">{ "1" }</div>
                        <strong>{ "Write" }</strong>
                        <p>{ "Create four files." }</p>
                    </div>

                    <div class="step">
                        <div class="step-number">{ "2" }</div>
                        <strong>{ "Build" }</strong>
                        <p>{ "Trunk compiles WASM." }</p>
                    </div>

                    <div class="step">
                        <div class="step-number">{ "3" }</div>
                        <strong>{ "Deploy" }</strong>
                        <p>{ "GitHub uploads dist." }</p>
                    </div>

                    <div class="step">
                        <div class="step-number">{ "4" }</div>
                        <strong>{ "Run" }</strong>
                        <p>{ "The app loads online." }</p>
                    </div>
                </div>
            </section>

            <section class="grid">
                { for CHAPTERS.iter().map(render_chapter) }
            </section>

            <section class="chapter">
                <h2>{ "The Key Workflow Improvement" }</h2>

                <p>
                    { "For this lesson, the GitHub Actions workflow creates a temporary Trunk.toml file before building. That file tells Trunk not to generate hashed output filenames." }
                </p>

                <div class="code-panel">
                    <pre>{ r#"[build]
filehash = false"# }</pre>
                </div>

                <div class="callout">
                    <strong>{ "Why this matters:" }</strong>
                    <p>
                        { "The build now produces clean deployment files such as styles.css, rust_lesson_3_deploy_pipeline.js, and rust_lesson_3_deploy_pipeline_bg.wasm. That makes FTP deployment cleaner and easier to reason about." }
                    </p>
                </div>

                <div class="callout warning">
                    <strong>{ "Strategic note:" }</strong>
                    <p>
                        { "Hashed files are not wrong. They are excellent for many CDN workflows. But for this Hostek/IIS teaching pipeline, predictable filenames are easier for students to understand and easier to manage." }
                    </p>
                </div>
            </section>

            <section class="chapter" id="source-code">
                <h2>{ "Full Lesson 3 Source Code" }</h2>

                <p>
                    { "These are the four files used in the Rust iPhone Compiler pattern. Students can review the structure, compare the files to the live app, and understand how the lesson was built." }
                </p>

                <div class="callout">
                    <strong>{ "Download the complete lesson project:" }</strong>
                    <p>
                        <a href="/rust-lesson-3-deploy-pipeline/rust-lesson-3-deploy-pipeline.zip">
                            { "/rust-lesson-3-deploy-pipeline/rust-lesson-3-deploy-pipeline.zip" }
                        </a>
                    </p>
                </div>

                <div class="source-grid">
                    { for SOURCE_FILES.iter().map(render_source_file) }
                </div>
            </section>

            <section class="chapter">
                <h2>{ "Expected Deployment Output" }</h2>

                <p>{ "After a successful build, the dist folder should contain clean non-hashed files." }</p>

                <div class="code-panel">
                    <pre>{ r#"index.html
styles.css
rust_lesson_3_deploy_pipeline.js
rust_lesson_3_deploy_pipeline_bg.wasm
web.config"# }</pre>
                </div>
            </section>

            <footer class="footer">
                <p>{ "MikeGyver Studio • Rust iPhone Compiler Tutorial Series" }</p>
                <p><a href="#top">{ "Back to top" }</a></p>
            </footer>
        </main>
    }
}

fn render_chapter(chapter: &Chapter) -> Html {
    html! {
        <article class="chapter" id={chapter.id}>
            <h2>{ chapter.title }</h2>

            <div class="audio-box">
                <strong>{ "Listen to this chapter" }</strong>
                <audio controls=true preload="none">
                    <source src={chapter.audio} type="audio/mpeg" />
                    { "Your browser does not support the audio element." }
                </audio>
            </div>

            <div class="transcript">
                { for chapter.lines.iter().map(|line| html! { <p>{ *line }</p> }) }
            </div>
        </article>
    }
}

fn render_source_file(file: &SourceFile) -> Html {
    html! {
        <article class="source-card">
            <div class="source-header">
                <strong>{ file.title }</strong>
                <span>{ file.language }</span>
            </div>

            <div class="code-panel">
                <pre>{ file.code }</pre>
            </div>
        </article>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}