use yew::prelude::*;

struct Chapter {
    id: &'static str,
    title: &'static str,
    audio: &'static str,
    body: &'static str,
}

const CHAPTERS: [Chapter; 5] = [
    Chapter {
        id: "intro",
        title: "Intro - From Rust Code to Live Website",
        audio: "./assets/audio/lesson3-intro.mp3",
        body: "Lesson 3 moves from building Rust apps to shipping Rust apps. The goal is not just to compile WebAssembly, but to understand the path from source code to a working public website.",
    },
    Chapter {
        id: "part1",
        title: "Part 1 - The Four File Pattern",
        audio: "./assets/audio/lesson3-part1.mp3",
        body: "The Rust iPhone Compiler keeps the learning path simple: index.html, styles.css, Cargo.toml, and src/main.rs. This gives beginners a complete app without overwhelming them with project structure.",
    },
    Chapter {
        id: "part2",
        title: "Part 2 - What Trunk Does",
        audio: "./assets/audio/lesson3-part2.mp3",
        body: "Trunk connects the HTML, CSS, Rust, JavaScript glue code, and WebAssembly output into a browser-ready package. It is the bridge between Rust source code and a deployable website.",
    },
    Chapter {
        id: "part3",
        title: "Part 3 - Clean Deployment Files",
        audio: "./assets/audio/lesson3-part3.mp3",
        body: "By setting filehash = false, the workflow creates predictable filenames. This is useful for FTP and IIS hosting because old hashed files do not linger after repeated deployments.",
    },
    Chapter {
        id: "outro",
        title: "Outro - Ship the Lesson",
        audio: "./assets/audio/lesson3-outro.mp3",
        body: "The bigger lesson is strategic: developers should understand the build pipeline, the hosting environment, and the reason behind every deployment choice. This is how a learning project becomes real software.",
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

            <p>{ chapter.body }</p>
        </article>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}