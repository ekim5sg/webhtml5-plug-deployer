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
"{opening}

{title}

{scripture_ref}
“{scripture_text}”

Reflection / Teaching
{reflection}

Application to Daily Life
{application}

Closing Encouragement
{encouragement}

Prayer
{prayer}

{signature}",
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

fn chars_count(text: &str) -> usize {
    text.chars().count()
}

#[function_component(App)]
fn app() -> Html {
    let title = use_state(|| "Walking in the Peace of Christ".to_string());
    let scripture_ref = use_state(|| "Philippians 4:6–7 (NLT)".to_string());
    let scripture_text = use_state(|| {
        "Don’t worry about anything; instead, pray about everything. Tell God what you need, and thank Him for all He has done. Then you will experience God’s peace, which exceeds anything we can understand. His peace will guard your