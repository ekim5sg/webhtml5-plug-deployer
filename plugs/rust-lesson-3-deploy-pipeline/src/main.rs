use yew::prelude::*;

struct Chapter {
    id: &'static str,
    title: &'static str,
    audio: &'static str,
    lines: &'static [&'static str],
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

            <div class="transcript">
                { for chapter.lines.iter().map(|line| html! { <p>{ *line }</p> }) }
            </div>
        </article>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}