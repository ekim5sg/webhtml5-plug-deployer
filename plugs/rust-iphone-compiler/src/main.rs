use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main style="font-family: system-ui; padding: 24px;">
            <h1>"Rust iPhone Compiler"</h1>
            <p>"https://www.webhtml5.info/rust-iphone-compiler/"</p>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
