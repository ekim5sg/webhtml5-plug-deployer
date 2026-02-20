use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main style="font-family: system-ui; padding: 24px;">
            <h1>{"Hello Plug"}</h1>
            <p>{"If you can see this, your plug deployer pipeline works."}</p>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
