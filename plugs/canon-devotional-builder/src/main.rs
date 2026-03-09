use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

const OPENING_LINE: &str = "Today, God in His Word.";
const CLOSING_SIGNATURE: &str = "On behalf of Pastor Noel,\nBrother Mike";

fn build_devotional_text(
    title: &str,
    scripture_ref: &str,
    scripture_text: &str,
    reflection: &str,
    application: &str,
    encouragement: &str,
    prayer: &str,
) -> String {
    format!(
        "{opening}\n\n{title}\n\n{scripture_ref}\n\"{scripture_text}\"\n\nShort Reflection / Teaching\n{reflection}\n\nApplication to Daily Life\n{application}\n\nClosing Encouragement\n{encouragement}\n\nPrayer\n{prayer}\n\n{signature}",
        opening = OPENING_LINE,
        title = title.trim(),
        scripture_ref = scripture_ref.trim(),
        scripture_text = scripture_text.trim(),
        reflection = reflection.trim(),
        application = application.trim(),
        encouragement = encouragement.trim(),
        prayer = prayer.trim(),
        signature = CLOSING_SIGNATURE,
    )
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn char_count(text: &str) -> usize {
    text.chars().count()
}

#[function_component(App)]
fn app() -> Html {
    let title = use_state(|| "Walking in the Peace of Christ".to_string());
    let scripture_ref = use_state(|| "Philippians 4:6-7 (NLT)".to_string());
    let scripture_text = use_state(|| "Don’t worry about anything. Instead, pray about everything. Tell God what you need, and thank Him for all He has done.".to_string());
    let reflection = use_state(|| "The Lord does not call us to carry our burdens alone. He invites us to bring every concern to Him in prayer and to trust that His peace is greater than our fear.".to_string());
    let application = use_state(|| "Today, pause before reacting to your worries. Bring them before the Lord in prayer, and choose to trust Him one step at a time.".to_string());
    let encouragement = use_state(|| "Whatever you are facing today, God is near, God is listening, and God is able to sustain you.".to_string());
    let prayer = use_state(|| "Heavenly Father, thank You for Your presence and peace. Teach us to pray instead of worry, to trust instead of fear, and to rest in Your unfailing love. In Jesus’ name, Amen.".to_string());

    let on_title_input = {
        let title = title.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            title.set(input.value());
        })
    };

    let on_scripture_ref_input = {
        let scripture_ref = scripture_ref.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            scripture_ref.set(input.value());
        })
    };

    let on_scripture_text_input = {
        let scripture_text = scripture_text.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            scripture_text.set(input.value());
        })
    };

    let on_reflection_input = {
        let reflection = reflection.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            reflection.set(input.value());
        })
    };

    let on_application_input = {
        let application = application.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            application.set(input.value());
        })
    };

    let on_encouragement_input = {
        let encouragement = encouragement.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            encouragement.set(input.value());
        })
    };

    let on_prayer_input = {
        let prayer = prayer.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            prayer.set(input.value());
        })
    };

    let on_load_sample = {
        let title = title.clone();
        let scripture_ref = scripture_ref.clone();
        let scripture_text = scripture_text.clone();
        let reflection = reflection.clone();
        let application = application.clone();
        let encouragement = encouragement.clone();
        let prayer = prayer.clone();
        Callback::from(move |_| {
            title.set("Walking in the Peace of Christ".to_string());
            scripture_ref.set("Philippians 4:6-7 (NLT)".to_string());
            scripture_text.set("Don’t worry about anything; instead, pray about everything. Tell God what you need, and thank Him for all He has done.".to_string());
            reflection.set("The Lord gently redirects our anxious hearts toward prayer. He does not minimize our burdens, but He invites us to bring them to Him and trust His care.".to_string());
            application.set("Take one concern that is weighing on you today and turn it into a prayer. Ask the Lord for peace, wisdom, and strength for the next step.".to_string());
            encouragement.set("God’s peace is still available to His children today. Stay near to Him, and let your heart rest in His promises.".to_string());
            prayer.set("Heavenly Father, thank You for caring for us. Help us to pray with faith, live with peace, and trust You in every circumstance. In Jesus’ name, Amen.".to_string());
        })
    };

    let on_clear_form = {
        let title = title.clone();
        let scripture_ref = scripture_ref.clone();
        let scripture_text = scripture_text.clone();
        let reflection = reflection.clone();
        let application = application.clone();
        let encouragement = encouragement.clone();
        let prayer = prayer.clone();
        Callback::from(move |_| {
            title.set(String::new());
            scripture_ref.set(String::new());
            scripture_text.set(String::new());
            reflection.set(String::new());
            application.set(String::new());
            encouragement.set(String::new());
            prayer.set(String::new());
        })
    };

    let devotional_text = build_devotional_text(
        &title,
        &scripture_ref,
        &scripture_text,
        &reflection,
        &application,
        &encouragement,
        &prayer,
    );

    let total_words = word_count(&devotional_text);
    let total_chars = char_count(&devotional_text);

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="eyebrow">{"MikeGyver Studio • Canon Devotional Builder"}</div>
                <h1>{"Daily Church Devotional Composer"}</h1>
                <p>{"Build your Monday through Friday devotional in the canon format, with the opening line and signature already standardized."}</p>
            </section>

            <div class="layout">
                <section class="card">
                    <h2>{"Compose the devotional"}</h2>

                    <div class="grid">
                        <div class="field">
                            <label for="title">{"Title / Theme"}</label>
                            <input id="title" type="text" value={(*title).clone()} oninput={on_title_input} placeholder="Walking in the Peace of Christ" />
                        </div>

                        <div class="field">
                            <label for="scripture_ref">{"Scripture Passage Reference"}</label>
                            <input id="scripture_ref" type="text" value={(*scripture_ref).clone()} oninput={on_scripture_ref_input} placeholder="Philippians 4:6-7 (NLT)" />
                        </div>

                        <div class="field">
                            <label for="scripture_text">{"Scripture Passage Text"}</label>
                            <textarea id="scripture_text" value={(*scripture_text).clone()} oninput={on_scripture_text_input} placeholder="Enter the full verse text here." />
                        </div>

                        <div class="field">
                            <label for="reflection">{"Short Reflection / Teaching"}</label>
                            <textarea id="reflection" class="textarea-tall" value={(*reflection).clone()} oninput={on_reflection_input} placeholder="Write the devotional reflection here." />
                        </div>

                        <div class="field">
                            <label for="application">{"Application to Daily Life"}</label>
                            <textarea id="application" value={(*application).clone()} oninput={on_application_input} placeholder="How should the reader apply this today?" />
                        </div>

                        <div class="field">
                            <label for="encouragement">{"Closing Encouragement"}</label>
                            <textarea id="encouragement" value={(*encouragement).clone()} oninput={on_encouragement_input} placeholder="Write a short closing encouragement." />
                        </div>

                        <div class="field">
                            <label for="prayer">{"Prayer"}</label>
                            <textarea id="prayer" value={(*prayer).clone()} oninput={on_prayer_input} placeholder="Write the closing prayer." />
                        </div>
                    </div>

                    <div class="toolbar">
                        <button class="primary" type="button" onclick={on_load_sample}>{"Load Sample"}</button>
                        <button type="button" onclick={on_clear_form}>{"Clear Form"}</button>
                    </div>
                </section>

                <aside class="card">
                    <h2>{"Live preview"}</h2>

                    <div class="stats">
                        <div class="stat">
                            <div class="stat-label">{"Words"}</div>
                            <div class="stat-value">{total_words}</div>
                        </div>
                        <div class="stat">
                            <div class="stat-label">{"Characters"}</div>
                            <div class="stat-value">{total_chars}</div>
                        </div>
                    </div>

                    <div class="preview output-box">{devotional_text}</div>

                    <p class="note">{"This preview follows your canon format with the fixed opening line and the standard closing signature for Pastor Noel’s weekday devotionals."}</p>
                </aside>
            </div>

            <div class="footer-space"></div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}