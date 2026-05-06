use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main class="card">
            <h1>{ "Trunk filehash = false test" }</h1>
            <p>{ "If the workflow worked, your dist files should have clean names." }</p>
            <div class="badge">{ "Expected: index.html, styles.css, filehash-test.js, filehash-test_bg.wasm" }</div>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}