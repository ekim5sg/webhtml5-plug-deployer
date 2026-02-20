use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main style="font-family: system-ui; padding: 24px;">
            <h1>{"This Is Cool ✅"}</h1>
            <p>{"Your webhtml5 plug deployer pipeline is live."}</p>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
