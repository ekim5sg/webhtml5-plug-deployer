use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main class="page">
            <section class="card">
                <div class="topline">{ "Rust • Yew • WebAssembly" }</div>

                <h1 class="title">
                    { "Hello World," }<br />
                    <span class="accent">{ "From Rust" }</span>
                    { " and " }
                    <span class="accent">{ "MikeGyver Studio" }</span>
                </h1>

                <p class="subtitle">
                    { "A simple Yew WASM starter app compiled by your GitHub worker and ready for your Rust iPhone Compiler workflow." }
                </p>

                <div class="console">
                    <div class="console-header">
                        <span class="dot red"></span>
                        <span class="dot yellow"></span>
                        <span class="dot green"></span>
                        <span class="console-title">{ "terminal output" }</span>
                    </div>

                    <div class="console-body">
                        <div><span class="prompt">{ "$ cargo run" }</span></div>
                        <div><span class="output">{ "Hello World, From Rust and MikeGyver Studio!" }</span></div>
                    </div>
                </div>

                <div class="signature">
                    <span class="badge">{ "Built with Rust" }</span>
                    <span class="badge">{ "Rendered with Yew" }</span>
                    <span class="badge">{ "Powered by MikeGyver Studio" }</span>
                </div>

                <p class="footer-note">
                    { "This is a clean starter you can extend into a richer landing page, animated console, or code-demo showcase later." }
                </p>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}