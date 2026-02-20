use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main style="font-family: system-ui, -apple-system, Segoe UI, Roboto, Arial, sans-serif; padding: 24px;">
            <h1>{"Who Is Your Crew?"}</h1>
            <p>{"Trunk + Yew build is working. Next: plug in the real crew selector UI."}</p>
            <ul>
                <li>{"✅ Trunk builds to dist/"}</li>
                <li>{"✅ Public URL set to /who-is-your-crew/"}</li>
                <li>{"✅ Hostek IIS web.config copied on deploy"}</li>
            </ul>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
