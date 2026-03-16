use gloo::timers::callback::Timeout;
use js_sys::{Array, Math, Uint8Array};
use wasm_bindgen::JsCast;
use web_sys::{
    window, Blob, BlobPropertyBag, HtmlAnchorElement, HtmlAudioElement, Url,
};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum OutcomeKind {
    Success,
    Warning,
    Failure,
}

#[derive(Clone, PartialEq)]
struct BuildOutcome {
    verdict: String,
    detail: String,
    kind: OutcomeKind,
    badge: String,
}

#[derive(Clone, PartialEq)]
enum ThemeMode {
    Night,
    Day,
}

#[derive(Clone, PartialEq)]
enum Difficulty {
    ParkingLot,
    SchoolPickup,
    RushHour,
    IphoneSafariDevMode,
    ConferenceWifi,
}

impl Difficulty {
    fn label(&self) -> &'static str {
        match self {
            Self::ParkingLot => "Parking Lot",
            Self::SchoolPickup => "School Pickup",
            Self::RushHour => "Rush Hour",
            Self::IphoneSafariDevMode => "iPhone Safari Dev Mode",
            Self::ConferenceWifi => "Conference Wi-Fi",
        }
    }

    fn from_value(v: &str) -> Self {
        match v {
            "school_pickup" => Self::SchoolPickup,
            "rush_hour" => Self::RushHour,
            "iphone_safari_dev_mode" => Self::IphoneSafariDevMode,
            "conference_wifi" => Self::ConferenceWifi,
            _ => Self::ParkingLot,
        }
    }

    fn as_value(&self) -> &'static str {
        match self {
            Self::ParkingLot => "parking_lot",
            Self::SchoolPickup => "school_pickup",
            Self::RushHour => "rush_hour",
            Self::IphoneSafariDevMode => "iphone_safari_dev_mode",
            Self::ConferenceWifi => "conference_wifi",
        }
    }

    fn success_bias(&self) -> f64 {
        match self {
            Self::ParkingLot => 0.50,
            Self::SchoolPickup => 0.42,
            Self::RushHour => 0.33,
            Self::IphoneSafariDevMode => 0.24,
            Self::ConferenceWifi => 0.16,
        }
    }

    fn warning_bias(&self) -> f64 {
        match self {
            Self::ParkingLot => 0.30,
            Self::SchoolPickup => 0.34,
            Self::RushHour => 0.37,
            Self::IphoneSafariDevMode => 0.40,
            Self::ConferenceWifi => 0.42,
        }
    }

    fn battery_drain_range(&self) -> (u32, u32) {
        match self {
            Self::ParkingLot => (4, 9),
            Self::SchoolPickup => (6, 11),
            Self::RushHour => (8, 13),
            Self::IphoneSafariDevMode => (9, 15),
            Self::ConferenceWifi => (10, 17),
        }
    }

    fn safari_range(&self) -> (u32, u32) {
        match self {
            Self::ParkingLot => (52, 92),
            Self::SchoolPickup => (42, 84),
            Self::RushHour => (30, 72),
            Self::IphoneSafariDevMode => (18, 58),
            Self::ConferenceWifi => (12, 48),
        }
    }

    fn progress_range(&self) -> (u32, u32) {
        match self {
            Self::ParkingLot => (80, 100),
            Self::SchoolPickup => (74, 96),
            Self::RushHour => (70, 92),
            Self::IphoneSafariDevMode => (66, 88),
            Self::ConferenceWifi => (60, 84),
        }
    }
}

fn random_index(len: usize) -> usize {
    ((Math::random() * len as f64).floor() as usize).min(len.saturating_sub(1))
}

fn random_pick(items: &[&str]) -> String {
    items[random_index(items.len())].to_string()
}

fn random_range(min: u32, max: u32) -> u32 {
    min + ((Math::random() * ((max - min + 1) as f64)).floor() as u32)
}

fn phase_for_progress(progress: u32) -> String {
    match progress {
        0..=14 => "Warming Up the Parking Lot Runtime".to_string(),
        15..=34 => "Negotiating with Dependencies".to_string(),
        35..=59 => "Borrow Checker Emotional Review".to_string(),
        60..=84 => "Mobile Safari Resistance Phase".to_string(),
        85..=99 => "Final Courage Linking".to_string(),
        _ => "Finished or Philosophically Finished".to_string(),
    }
}

fn make_outcome(difficulty: &Difficulty) -> BuildOutcome {
    let roll = Math::random();

    let successes = [
        (
            "Build succeeded.",
            "Finished dev profile with 3 warnings, 2 miracles, and zero surrender.",
            "🏁 Carpool Lane Champion",
        ),
        (
            "Build succeeded with swagger.",
            "The app compiled before your battery lost the will to continue.",
            "🦀 Mobile Rust Survivor",
        ),
        (
            "Build finished.",
            "Against several odds and one emotional weather front, the compile completed.",
            "📱 Thumb-Driven Build Hero",
        ),
    ];

    let warnings = [
        (
            "Build succeeded with emotional warnings.",
            "Your code is fine, but the borrow checker would like to discuss boundaries.",
            "⚠️ Technically Victorious",
        ),
        (
            "Build technically passed.",
            "The output works, though mobile Safari remains personally offended.",
            "😅 Barely Street Legal",
        ),
        (
            "Build completed with caution.",
            "No fatal errors occurred, but the compiler did sigh audibly.",
            "🛠️ Warned But Unbroken",
        ),
    ];

    let failures = [
        (
            "Build failed.",
            "error: borrowed value does not believe in you anymore",
            "💥 Closure Casualty",
        ),
        (
            "Build halted.",
            "error[E0382]: your confidence was moved into a closure",
            "📉 Borrow Checker Victim",
        ),
        (
            "Build failed dramatically.",
            "error: expected resilience, found semicolon",
            "🫠 Syntax Ambush Survivor",
        ),
    ];

    let success_cutoff = difficulty.success_bias();
    let warning_cutoff = success_cutoff + difficulty.warning_bias();

    if roll < success_cutoff {
        let (verdict, detail, badge) = successes[random_index(successes.len())];
        BuildOutcome {
            verdict: verdict.to_string(),
            detail: detail.to_string(),
            kind: OutcomeKind::Success,
            badge: badge.to_string(),
        }
    } else if roll < warning_cutoff {
        let (verdict, detail, badge) = warnings[random_index(warnings.len())];
        BuildOutcome {
            verdict: verdict.to_string(),
            detail: detail.to_string(),
            kind: OutcomeKind::Warning,
            badge: badge.to_string(),
        }
    } else {
        let (verdict, detail, badge) = failures[random_index(failures.len())];
        BuildOutcome {
            verdict: verdict.to_string(),
            detail: detail.to_string(),
            kind: OutcomeKind::Failure,
            badge: badge.to_string(),
        }
    }
}

fn make_excuse() -> String {
    let excuses = [
        "The compiler would like to remind you that ideal conditions are a myth.",
        "This build was delayed by signal strength, thumb accuracy, and destiny.",
        "Mobile Safari has reviewed your ambition and requested additional paperwork.",
        "The borrow checker isn't angry. It's just disappointed in your assumptions.",
        "Your code may be correct, but the environment prefers drama.",
        "This build is powered by caffeine, faith, and one unstable parking lot signal.",
        "The app compiled only because it feared your persistence.",
        "The semicolon was innocent. The closure was not.",
    ];

    random_pick(&excuses)
}

fn make_motivation() -> String {
    let quotes = [
        "Not every build needs perfect conditions. Some just need one more honest try.",
        "The workstation is optional. The stubbornness is not.",
        "Sometimes progress is just refusing to close the tab.",
        "The carpool lane may not respect your workflow, but your workflow can still win.",
        "You do not need ideal hardware to create memorable software.",
        "A weird build story today becomes developer folklore tomorrow.",
        "Finishing under imperfect conditions is still finishing.",
    ];

    random_pick(&quotes)
}

fn rust_chaos_line() -> String {
    let lines = [
        "error[E0507]: cannot move out of optimism because it is borrowed by reality",
        "warning: variable `weekend_plans` is never used",
        "error[E0382]: motivation was moved into closure",
        "help: consider borrowing courage instead",
        "warning: function `sleep_schedule` is never used",
        "error[E0277]: trait bound `Coffee: Compile` is not satisfied",
        "warning: mobile Safari remains legally skeptical",
        "note: required by a bound in `finish_the_build_anyway`",
        "error[E0599]: no method named `inner_peace` found for struct `Developer`",
        "help: try `clone()` on your dreams before reuse",
    ];

    random_pick(&lines)
}

fn build_log_script(
    progress: u32,
    phase: &str,
    drain: u32,
    outcome: &BuildOutcome,
    chaos_mode: bool,
    difficulty: &Difficulty,
) -> Vec<String> {
    let openings = [
        "INFO  starting cargo build in the carpool lane...",
        "INFO  syncing with thumb-powered dev infrastructure...",
        "INFO  initializing mobile determination runtime...",
        "INFO  checking whether this is a good idea... inconclusive",
    ];

    let targets = [
        "INFO  target = wasm32-unknown-unknown",
        "INFO  target = wasm32-unknown-unknown-and-pray",
        "INFO  target = iphone-browser-release-ish",
        "INFO  target = mobile-safari-with-concerns",
    ];

    let telemetry = [
        "INFO  locating signal... found intermittent competence",
        "INFO  keyboard accuracy reduced by parking lot conditions",
        "INFO  steering wheel desk mode engaged",
        "INFO  outside temperature ignored in favor of progress",
        "INFO  one child asked 'is it done yet?'",
    ];

    let warnings = [
        "warning: function `sleep_schedule` is never used",
        "warning: variable `weekend_rest` is assigned to, but never read",
        "warning: optimism does not implement `Copy`",
        "warning: mobile Safari remains legally skeptical",
        "warning: this build contains traces of caffeine and audacity",
    ];

    let steps = [
        "INFO  checking cached crates... surprisingly respectable",
        "INFO  compiling yew components...",
        "INFO  linking against unreasonable persistence...",
        "INFO  validating CSS against one suspicious browser tab...",
        "INFO  borrowing courage mutably for final push...",
        "INFO  optimizing release profile spiritually, not technically...",
    ];

    let mut script = vec![
        random_pick(&openings),
        random_pick(&targets),
        format!("INFO  difficulty = {}", difficulty.label()),
        random_pick(&telemetry),
        random_pick(&steps),
        format!("INFO  phase = {phase}"),
        format!("INFO  progress = {progress}%"),
        random_pick(&warnings),
        format!("INFO  battery drain this run = -{}%", drain),
    ];

    if chaos_mode {
        script.insert(4, rust_chaos_line());
        script.insert(7, rust_chaos_line());
    }

    match outcome.kind {
        OutcomeKind::Success => {
            script.push("INFO  finished build pipeline".to_string());
            if chaos_mode {
                script.push(rust_chaos_line());
            }
            script.push(format!("SUCCESS  {}", outcome.detail));
        }
        OutcomeKind::Warning => {
            script.push("INFO  finished build pipeline with interpretive confidence".to_string());
            if chaos_mode {
                script.push(rust_chaos_line());
            }
            script.push(format!("WARN  {}", outcome.detail));
        }
        OutcomeKind::Failure => {
            script.push(format!("ERROR  {}", outcome.detail));
            script.push("HELP  suggestion: clone your dreams before use".to_string());
            if chaos_mode {
                script.push(rust_chaos_line());
            }
            script.push("HELP  alternate suggestion: blame the closure respectfully".to_string());
        }
    }

    script
}

fn add_body_theme_class(theme: &ThemeMode, screenshot_mode: bool) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(body) = doc.body() {
            let theme_class = match theme {
                ThemeMode::Night => "theme-night",
                ThemeMode::Day => "theme-day",
            };

            if screenshot_mode {
                body.set_class_name(&format!("{theme_class} screenshot-mode"));
            } else {
                body.set_class_name(theme_class);
            }
        }
    }
}

fn write_le_u16(bytes: &mut Vec<u8>, v: u16) {
    bytes.push((v & 0xff) as u8);
    bytes.push((v >> 8) as u8);
}

fn write_le_u32(bytes: &mut Vec<u8>, v: u32) {
    bytes.push((v & 0xff) as u8);
    bytes.push(((v >> 8) & 0xff) as u8);
    bytes.push(((v >> 16) & 0xff) as u8);
    bytes.push(((v >> 24) & 0xff) as u8);
}

fn generate_wav_bytes(freq_hz: f32, duration_ms: u32, volume: f32) -> Vec<u8> {
    let sample_rate: u32 = 22050;
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let total_samples = (sample_rate as f32 * (duration_ms as f32 / 1000.0)) as usize;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = (total_samples * 2) as u32;

    let mut bytes = Vec::with_capacity(44 + data_size as usize);
    bytes.extend_from_slice(b"RIFF");
    write_le_u32(&mut bytes, 36 + data_size);
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    write_le_u32(&mut bytes, 16);
    write_le_u16(&mut bytes, 1);
    write_le_u16(&mut bytes, channels);
    write_le_u32(&mut bytes, sample_rate);
    write_le_u32(&mut bytes, byte_rate);
    write_le_u16(&mut bytes, block_align);
    write_le_u16(&mut bytes, bits_per_sample);
    bytes.extend_from_slice(b"data");
    write_le_u32(&mut bytes, data_size);

    let amp = (i16::MAX as f32 * volume).round() as i16;
    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let sample = (f32::sin(2.0 * std::f32::consts::PI * freq_hz * t) * amp as f32) as i16;
        write_le_u16(&mut bytes, sample as u16);
    }

    bytes
}

fn play_tone(freq_hz: f32, duration_ms: u32, volume: f32) {
    let Some(win) = window() else { return; };
    let Some(doc) = win.document() else { return; };

    let wav = generate_wav_bytes(freq_hz, duration_ms, volume);
    let arr = Uint8Array::new_with_length(wav.len() as u32);
    arr.copy_from(&wav);

    let parts = Array::new();
    parts.push(&arr.buffer());

    let bag = BlobPropertyBag::new();
    bag.set_type("audio/wav");

    let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &bag) else {
        return;
    };

    let Ok(url) = Url::create_object_url_with_blob(&blob) else {
        return;
    };

    let Ok(el) = doc.create_element("audio") else {
        let _ = Url::revoke_object_url(&url);
        return;
    };

    let Ok(audio) = el.dyn_into::<HtmlAudioElement>() else {
        let _ = Url::revoke_object_url(&url);
        return;
    };

    audio.set_src(&url);
    let _ = audio.play();

    Timeout::new(duration_ms + 400, move || {
        let _ = Url::revoke_object_url(&url);
    })
    .forget();
}

fn play_outcome_sound(kind: &OutcomeKind) {
    match kind {
        OutcomeKind::Success => {
            play_tone(660.0, 140, 0.22);
            Timeout::new(150, move || play_tone(880.0, 180, 0.22)).forget();
        }
        OutcomeKind::Warning => {
            play_tone(520.0, 160, 0.20);
            Timeout::new(170, move || play_tone(460.0, 180, 0.18)).forget();
        }
        OutcomeKind::Failure => {
            play_tone(320.0, 220, 0.22);
            Timeout::new(210, move || play_tone(220.0, 260, 0.20)).forget();
        }
    }
}

fn trigger_text_download(filename: &str, content: &str) {
    let Some(win) = window() else { return; };
    let Some(doc) = win.document() else { return; };

    let bytes = Uint8Array::from(content.as_bytes());
    let parts = Array::new();
    parts.push(&bytes.buffer());

    let bag = BlobPropertyBag::new();
    bag.set_type("text/plain;charset=utf-8");

    let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &bag) else {
        return;
    };

    let Ok(url) = Url::create_object_url_with_blob(&blob) else {
        return;
    };

    let Ok(el) = doc.create_element("a") else {
        let _ = Url::revoke_object_url(&url);
        return;
    };

    let Ok(anchor) = el.dyn_into::<HtmlAnchorElement>() else {
        let _ = Url::revoke_object_url(&url);
        return;
    };

    anchor.set_href(&url);
    anchor.set_download(filename);
    let _ = anchor.style().set_property("display", "none");

    if let Some(body) = doc.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    }

    let _ = Url::revoke_object_url(&url);
}

#[derive(Clone)]
struct BuildPlan {
    signal: String,
    target: String,
    mood: String,
    resistance: u32,
    drain: u32,
    progress: u32,
    phase: String,
    outcome: BuildOutcome,
    script: Vec<String>,
    staged_progress_values: Vec<u32>,
}

fn make_build_plan(difficulty: &Difficulty, chaos_mode: bool) -> BuildPlan {
    let signal_options = [
        "1 bar + stubbornness",
        "2 bars + faith",
        "parking-lot Wi-Fi aura",
        "surprisingly decent LTE",
        "held together by hope",
    ];

    let target_options = [
        "wasm32-unknown-unknown",
        "mobile-safari-experimental",
        "iphone-browser-release-ish",
        "wasm32-unknown-unknown-and-pray",
        "yew-prod-mobile-chaos",
    ];

    let mood_options = [
        "Judgmental but fair",
        "Cautiously impressed",
        "Not mad, just strict",
        "Emotionally unavailable",
        "Stern, glowing, inevitable",
    ];

    let (drain_min, drain_max) = difficulty.battery_drain_range();
    let (safari_min, safari_max) = difficulty.safari_range();
    let (progress_min, progress_max) = difficulty.progress_range();

    let final_progress = random_range(progress_min, progress_max);
    let phase = phase_for_progress(final_progress);
    let outcome = make_outcome(difficulty);
    let drain = random_range(drain_min, drain_max);

    let staged = vec![
        5u32,
        12,
        21,
        33,
        46,
        58,
        69,
        79,
        90,
        final_progress,
    ];

    let script = build_log_script(final_progress, &phase, drain, &outcome, chaos_mode, difficulty);

    BuildPlan {
        signal: random_pick(&signal_options),
        target: random_pick(&target_options),
        mood: random_pick(&mood_options),
        resistance: random_range(safari_min, safari_max),
        drain,
        progress: final_progress,
        phase,
        outcome,
        script,
        staged_progress_values: staged,
    }
}

fn schedule_build_run(
    plan: BuildPlan,
    logs: UseStateHandle<Vec<String>>,
    outcome: UseStateHandle<Option<BuildOutcome>>,
    build_count: UseStateHandle<u32>,
    progress: UseStateHandle<u32>,
    phase: UseStateHandle<String>,
    env_signal: UseStateHandle<String>,
    env_battery: UseStateHandle<u32>,
    env_target: UseStateHandle<String>,
    env_mood: UseStateHandle<String>,
    safari_resistance: UseStateHandle<u32>,
    is_building: UseStateHandle<bool>,
    sound_enabled: UseStateHandle<bool>,
    run_delay_ms: u32,
    on_finish: Option<Callback<()>>,
) {
    let start_logs = logs.clone();
    let start_phase = phase.clone();
    let start_progress = progress.clone();
    let start_env_signal = env_signal.clone();
    let start_env_target = env_target.clone();
    let start_env_mood = env_mood.clone();
    let start_env_battery = env_battery.clone();
    let start_safari = safari_resistance.clone();
    let start_is_building = is_building.clone();

    Timeout::new(run_delay_ms, move || {
        start_is_building.set(true);
        start_env_signal.set(plan.signal.clone());
        start_env_target.set(plan.target.clone());
        start_env_mood.set(plan.mood.clone());
        start_safari.set(plan.resistance);

        let current_battery = *start_env_battery;
        start_env_battery.set(current_battery.saturating_sub(plan.drain));

        start_progress.set(1);
        start_phase.set("Bootstrapping questionable brilliance".to_string());
        start_logs.set(vec![
            "INFO  preparing animated build sequence...".to_string(),
            "INFO  conditions accepted. consequences pending.".to_string(),
        ]);

        for (idx, line) in plan.script.clone().into_iter().enumerate() {
            let logs = logs.clone();
            let progress = progress.clone();
            let phase = phase.clone();
            let outcome = outcome.clone();
            let build_count = build_count.clone();
            let is_building = is_building.clone();
            let outcome_final = plan.outcome.clone();
            let phase_final = plan.phase.clone();
            let staged_progress_values = plan.staged_progress_values.clone();
            let final_progress = plan.progress;
            let sound_enabled = sound_enabled.clone();
            let on_finish = on_finish.clone();

            Timeout::new(run_delay_ms + (idx as u32) * 520 + 180, move || {
                let mut current = (*logs).clone();
                current.push(line);
                logs.set(current);

                let staged = staged_progress_values.get(idx).copied().unwrap_or(final_progress);
                progress.set(staged);
                phase.set(phase_for_progress(staged));

                if idx + 1 == staged_progress_values.len() {
                    progress.set(final_progress);
                    phase.set(phase_final);
                    outcome.set(Some(outcome_final.clone()));
                    build_count.set(*build_count + 1);
                    is_building.set(false);

                    if *sound_enabled {
                        play_outcome_sound(&outcome_final.kind);
                    }

                    if let Some(cb) = on_finish {
                        cb.emit(());
                    }
                }
            })
            .forget();
        }
    })
    .forget();
}

#[function_component(App)]
fn app() -> Html {
    let logs = use_state(|| {
        vec![
            "Cargo Build in the Carpool Lane v5 ready.".to_string(),
            "Tap “Start Animated Build” to test your courage under mobile conditions.".to_string(),
        ]
    });

    let outcome = use_state(|| None as Option<BuildOutcome>);
    let build_count = use_state(|| 0u32);
    let progress = use_state(|| 0u32);
    let phase = use_state(|| "Awaiting unreasonable optimism".to_string());
    let excuse = use_state(make_excuse);
    let motivation = use_state(make_motivation);
    let theme = use_state(|| ThemeMode::Night);
    let copied = use_state(|| false);
    let is_building = use_state(|| false);
    let sound_enabled = use_state(|| true);
    let certificate_flash = use_state(|| false);
    let share_flash = use_state(|| false);
    let screenshot_mode = use_state(|| false);
    let chaos_mode = use_state(|| true);
    let demo_mode = use_state(|| false);
    let demo_count = use_state(|| 0u32);
    let legend_unlocked = use_state(|| false);
    let difficulty = use_state(|| Difficulty::ParkingLot);

    let env_signal = use_state(|| "2 bars + faith".to_string());
    let env_battery = use_state(|| 17u32);
    let env_target = use_state(|| "wasm32-unknown-unknown".to_string());
    let env_mood = use_state(|| "Judgmental but fair".to_string());
    let safari_resistance = use_state(|| 61u32);

    {
        let theme = theme.clone();
        let screenshot_mode = screenshot_mode.clone();
        use_effect_with((theme, screenshot_mode), move |(theme_mode, shot_mode)| {
            add_body_theme_class(theme_mode, *shot_mode);
            || ()
        });
    }

    let on_toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let next = match *theme {
                ThemeMode::Night => ThemeMode::Day,
                ThemeMode::Day => ThemeMode::Night,
            };
            theme.set(next);
        })
    };

    let on_toggle_sound = {
        let sound_enabled = sound_enabled.clone();
        Callback::from(move |_| {
            sound_enabled.set(!*sound_enabled);
        })
    };

    let on_toggle_screenshot = {
        let screenshot_mode = screenshot_mode.clone();
        Callback::from(move |_| {
            screenshot_mode.set(!*screenshot_mode);
        })
    };

    let on_toggle_chaos = {
        let chaos_mode = chaos_mode.clone();
        Callback::from(move |_| {
            chaos_mode.set(!*chaos_mode);
        })
    };

    let on_add_drama = {
        let logs = logs.clone();
        let excuse = excuse.clone();
        let motivation = motivation.clone();
        Callback::from(move |_| {
            let extras = [
                "warning: child in back seat requests snack-driven refactor",
                "INFO  a passing cloud briefly reduced compile morale",
                "warning: one typo introduced by heroic thumb reach",
                "INFO  rebuild triggered by mysterious CSS confidence issue",
                "warning: release notes now contain accidental wisdom",
                "ERROR  trunk served hot, browser responded cold",
                "INFO  this project has entered legendary anecdote territory",
                "warning: one div was emotionally misaligned",
                "INFO  steering wheel desk remains surprisingly stable",
            ];

            let mut current = (*logs).clone();
            current.push(random_pick(&extras));
            logs.set(current);
            excuse.set(make_excuse());
            motivation.set(make_motivation());
        })
    };

    let on_recharge = {
        let env_battery = env_battery.clone();
        let logs = logs.clone();
        Callback::from(move |_| {
            let boosted = (*env_battery + 18).min(100);
            env_battery.set(boosted);

            let mut current = (*logs).clone();
            current.push(format!("INFO  emergency recharge applied -> battery now {}%", boosted));
            logs.set(current);
        })
    };

    let on_copy_report = {
        let outcome = outcome.clone();
        let build_count = build_count.clone();
        let phase = phase.clone();
        let progress = progress.clone();
        let env_signal = env_signal.clone();
        let env_battery = env_battery.clone();
        let env_target = env_target.clone();
        let env_mood = env_mood.clone();
        let safari_resistance = safari_resistance.clone();
        let difficulty = difficulty.clone();
        let copied = copied.clone();

        Callback::from(move |_| {
            let result_text = if let Some(result) = &*outcome {
                format!(
                    "Cargo Build in the Carpool Lane v5\n\
                     Build #{}\n\
                     Difficulty: {}\n\
                     Verdict: {}\n\
                     Detail: {}\n\
                     Badge: {}\n\
                     Phase: {}\n\
                     Progress: {}%\n\
                     Signal: {}\n\
                     Battery: {}%\n\
                     Target: {}\n\
                     Compiler Mood: {}\n\
                     Safari Resistance: {}/100",
                    *build_count,
                    difficulty.label(),
                    result.verdict,
                    result.detail,
                    result.badge,
                    (*phase).clone(),
                    *progress,
                    (*env_signal).clone(),
                    *env_battery,
                    (*env_target).clone(),
                    (*env_mood).clone(),
                    *safari_resistance
                )
            } else {
                "Cargo Build in the Carpool Lane v5\nNo completed build report yet.".to_string()
            };

            if let Some(clipboard) = window().map(|w| w.navigator().clipboard()) {
                let _ = clipboard.write_text(&result_text);
                copied.set(true);

                let copied_reset = copied.clone();
                Timeout::new(1800, move || {
                    copied_reset.set(false);
                })
                .forget();
            }
        })
    };

    let on_download_certificate = {
        let outcome = outcome.clone();
        let build_count = build_count.clone();
        let phase = phase.clone();
        let progress = progress.clone();
        let env_signal = env_signal.clone();
        let env_battery = env_battery.clone();
        let env_target = env_target.clone();
        let env_mood = env_mood.clone();
        let safari_resistance = safari_resistance.clone();
        let difficulty = difficulty.clone();
        let certificate_flash = certificate_flash.clone();

        Callback::from(move |_| {
            let content = if let Some(result) = &*outcome {
                format!(
                    "CERTIFICATE OF IMPROBABLE COMPILATION\n\
                     ====================================\n\n\
                     This certifies that Build #{} of\n\
                     Cargo Build in the Carpool Lane v5\n\
                     was attempted under deeply mobile conditions.\n\n\
                     Difficulty: {}\n\
                     Verdict: {}\n\
                     Detail: {}\n\
                     Badge: {}\n\
                     Phase: {}\n\
                     Progress: {}%\n\
                     Signal: {}\n\
                     Battery: {}%\n\
                     Target: {}\n\
                     Compiler Mood: {}\n\
                     Safari Resistance: {}/100\n\n\
                     Official observation:\n\
                     The developer persisted despite battery anxiety,\n\
                     browser skepticism, and ambient carpool-lane chaos.\n\n\
                     Signed,\n\
                     The Department of Unreasonable Optimism\n",
                    *build_count,
                    difficulty.label(),
                    result.verdict,
                    result.detail,
                    result.badge,
                    (*phase).clone(),
                    *progress,
                    (*env_signal).clone(),
                    *env_battery,
                    (*env_target).clone(),
                    (*env_mood).clone(),
                    *safari_resistance
                )
            } else {
                "No build certificate available yet. Complete a build first.\n".to_string()
            };

            trigger_text_download("carpool-lane-build-certificate.txt", &content);
            certificate_flash.set(true);

            let certificate_flash_reset = certificate_flash.clone();
            Timeout::new(1600, move || {
                certificate_flash_reset.set(false);
            })
            .forget();
        })
    };

    let on_generate_share_card = {
        let share_flash = share_flash.clone();
        Callback::from(move |_| {
            share_flash.set(true);
            let share_flash_reset = share_flash.clone();
            Timeout::new(1600, move || {
                share_flash_reset.set(false);
            })
            .forget();
        })
    };

    let on_reset = {
        let logs = logs.clone();
        let outcome = outcome.clone();
        let progress = progress.clone();
        let phase = phase.clone();
        let excuse = excuse.clone();
        let motivation = motivation.clone();
        let env_signal = env_signal.clone();
        let env_battery = env_battery.clone();
        let env_target = env_target.clone();
        let env_mood = env_mood.clone();
        let safari_resistance = safari_resistance.clone();
        let copied = copied.clone();
        let is_building = is_building.clone();
        let certificate_flash = certificate_flash.clone();
        let share_flash = share_flash.clone();
        let demo_mode = demo_mode.clone();
        let demo_count = demo_count.clone();
        let legend_unlocked = legend_unlocked.clone();

        Callback::from(move |_| {
            logs.set(vec![
                "Cargo Build in the Carpool Lane v5 ready.".to_string(),
                "Tap “Start Animated Build” to test your courage under mobile conditions.".to_string(),
            ]);
            outcome.set(None);
            progress.set(0);
            phase.set("Awaiting unreasonable optimism".to_string());
            excuse.set(make_excuse());
            motivation.set(make_motivation());
            env_signal.set("2 bars + faith".to_string());
            env_battery.set(17);
            env_target.set("wasm32-unknown-unknown".to_string());
            env_mood.set("Judgmental but fair".to_string());
            safari_resistance.set(61);
            copied.set(false);
            is_building.set(false);
            certificate_flash.set(false);
            share_flash.set(false);
            demo_mode.set(false);
            demo_count.set(0);
            legend_unlocked.set(false);
        })
    };

    let on_start_build = {
        let logs = logs.clone();
        let outcome = outcome.clone();
        let build_count = build_count.clone();
        let progress = progress.clone();
        let phase = phase.clone();
        let excuse = excuse.clone();
        let motivation = motivation.clone();
        let env_signal = env_signal.clone();
        let env_battery = env_battery.clone();
        let env_target = env_target.clone();
        let env_mood = env_mood.clone();
        let safari_resistance = safari_resistance.clone();
        let is_building = is_building.clone();
        let sound_enabled = sound_enabled.clone();
        let difficulty = difficulty.clone();
        let chaos_mode = chaos_mode.clone();
        let demo_mode = demo_mode.clone();
        let demo_count = demo_count.clone();
        let legend_unlocked = legend_unlocked.clone();

        Callback::from(move |_| {
            if *is_building {
                return;
            }

            demo_mode.set(false);
            demo_count.set(0);
            legend_unlocked.set(false);

            outcome.set(None);
            excuse.set(make_excuse());
            motivation.set(make_motivation());

            let plan = make_build_plan(&difficulty, *chaos_mode);

            schedule_build_run(
                plan,
                logs.clone(),
                outcome.clone(),
                build_count.clone(),
                progress.clone(),
                phase.clone(),
                env_signal.clone(),
                env_battery.clone(),
                env_target.clone(),
                env_mood.clone(),
                safari_resistance.clone(),
                is_building.clone(),
                sound_enabled.clone(),
                0,
                None,
            );
        })
    };

    let on_demo_mode = {
        let logs = logs.clone();
        let outcome = outcome.clone();
        let build_count = build_count.clone();
        let progress = progress.clone();
        let phase = phase.clone();
        let excuse = excuse.clone();
        let motivation = motivation.clone();
        let env_signal = env_signal.clone();
        let env_battery = env_battery.clone();
        let env_target = env_target.clone();
        let env_mood = env_mood.clone();
        let safari_resistance = safari_resistance.clone();
        let is_building = is_building.clone();
        let sound_enabled = sound_enabled.clone();
        let difficulty = difficulty.clone();
        let chaos_mode = chaos_mode.clone();
        let demo_mode = demo_mode.clone();
        let demo_count = demo_count.clone();
        let legend_unlocked = legend_unlocked.clone();

        Callback::from(move |_| {
            if *is_building {
                return;
            }

            demo_mode.set(true);
            demo_count.set(0);
            legend_unlocked.set(false);
            outcome.set(None);
            excuse.set(make_excuse());
            motivation.set(make_motivation());

            let demo_count_after_1 = demo_count.clone();
            let demo_count_after_2 = demo_count.clone();
            let demo_count_after_3 = demo_count.clone();
            let legend_after_3 = legend_unlocked.clone();
            let demo_mode_after_3 = demo_mode.clone();

            let cb1 = Callback::from(move |_| {
                demo_count_after_1.set(1);
            });

            let cb2 = Callback::from(move |_| {
                demo_count_after_2.set(2);
            });

            let cb3 = Callback::from(move |_| {
                demo_count_after_3.set(3);
                legend_after_3.set(true);
                demo_mode_after_3.set(false);
            });

            let plan1 = make_build_plan(&difficulty, *chaos_mode);
            let plan2 = make_build_plan(&difficulty, *chaos_mode);
            let plan3 = make_build_plan(&difficulty, *chaos_mode);

            schedule_build_run(
                plan1,
                logs.clone(),
                outcome.clone(),
                build_count.clone(),
                progress.clone(),
                phase.clone(),
                env_signal.clone(),
                env_battery.clone(),
                env_target.clone(),
                env_mood.clone(),
                safari_resistance.clone(),
                is_building.clone(),
                sound_enabled.clone(),
                0,
                Some(cb1),
            );

            schedule_build_run(
                plan2,
                logs.clone(),
                outcome.clone(),
                build_count.clone(),
                progress.clone(),
                phase.clone(),
                env_signal.clone(),
                env_battery.clone(),
                env_target.clone(),
                env_mood.clone(),
                safari_resistance.clone(),
                is_building.clone(),
                sound_enabled.clone(),
                6200,
                Some(cb2),
            );

            schedule_build_run(
                plan3,
                logs.clone(),
                outcome.clone(),
                build_count.clone(),
                progress.clone(),
                phase.clone(),
                env_signal.clone(),
                env_battery.clone(),
                env_target.clone(),
                env_mood.clone(),
                safari_resistance.clone(),
                is_building.clone(),
                sound_enabled.clone(),
                12400,
                Some(cb3),
            );
        })
    };

    let on_change_difficulty = {
        let difficulty = difficulty.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlElement = e.target_unchecked_into();
            let value = js_sys::Reflect::get(&input, &"value".into())
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "parking_lot".to_string());
            difficulty.set(Difficulty::from_value(&value));
        })
    };

    let result_block = if let Some(result) = &*outcome {
        let class = match result.kind {
            OutcomeKind::Success => "result-value result-success",
            OutcomeKind::Warning => "result-value result-warning",
            OutcomeKind::Failure => "result-value result-danger",
        };

        let banner_class = match result.kind {
            OutcomeKind::Success => "verdict-banner success",
            OutcomeKind::Warning => "verdict-banner warning",
            OutcomeKind::Failure => "verdict-banner failure",
        };

        html! {
            <>
                <div class={banner_class}>
                    <div class="verdict-kicker">{"Final Verdict"}</div>
                    <div class={class.clone()}>{&result.verdict}</div>
                    <div class="verdict-headline">{format!("{} • Build #{}", result.badge, *build_count)}</div>
                    <div class="verdict-detail">{&result.detail}</div>
                </div>

                <div class="section-box">
                    <div class="section-label">{"Shareable Build Badge"}</div>
                    <div class="result-value">{format!("{} • Build #{}", result.badge, *build_count)}</div>
                    <div class="card-subtitle" style="margin-top:8px;">
                        {"Optimized for screenshots, demo reels, and highly specific LinkedIn humor."}
                    </div>
                </div>
            </>
        }
    } else {
        html! {
            <>
                <div class="section-box">
                    <div class="section-label">{"Latest Verdict"}</div>
                    <div class="result-value">{"No completed build yet."}</div>
                    <div class="card-subtitle" style="margin-top:8px;">
                        {"Everything remains theoretically under control."}
                    </div>
                </div>

                <div class="section-box">
                    <div class="section-label">{"Shareable Build Badge"}</div>
                    <div class="result-value">{"🕶️ Pre-Build Legend"}</div>
                    <div class="card-subtitle" style="margin-top:8px;">
                        {"Untested, unbroken, and still emotionally undefeated."}
                    </div>
                </div>
            </>
        }
    };

    let safari_class = if *safari_resistance >= 70 {
        "meter-value meter-good"
    } else if *safari_resistance >= 40 {
        "meter-value meter-warn"
    } else {
        "meter-value meter-danger"
    };

    let theme_label = match *theme {
        ThemeMode::Night => "🌙 Night Carpool Mode",
        ThemeMode::Day => "☀️ Day Carpool Mode",
    };

    let report_preview = if let Some(result) = &*outcome {
        format!(
            "build={} | difficulty={} | verdict={} | battery={} | signal={} | safari={}/100 | badge={}",
            *build_count,
            difficulty.label(),
            result.verdict,
            *env_battery,
            (*env_signal).clone(),
            *safari_resistance,
            result.badge
        )
    } else {
        "No build report yet. Start a run to generate developer folklore.".to_string()
    };

    let share_title = if let Some(result) = &*outcome {
        format!("{} • {}", result.badge, result.verdict)
    } else {
        "Pre-Build Legend • Awaiting glorious chaos".to_string()
    };

    let share_phase = (*phase).clone();
    let share_signal = (*env_signal).clone();
    let share_battery = format!("{}%", *env_battery);
    let share_browser = format!("{}/100", *safari_resistance);

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="hero-top">
                    <div>
                        <div class="eyebrow">{"🚗🦀 Legendary Edition, v5"}</div>
                        <h1>{"Cargo Build in the Carpool Lane v5"}</h1>
                        <p>
                            {"A polished fake compiler dashboard for deeply real developer energy: limited battery, questionable signal, mobile-browser drama, and the quiet refusal to give up. "}
                            {"Now with screenshot mode, demo mode, Rust chaos, and difficulty levels that make bad decisions feel configurable."}
                        </p>
                    </div>

                    <div class="hero-actions">
                        <button class="secondary" onclick={on_toggle_theme}>{theme_label}</button>
                        <button class="secondary" onclick={on_toggle_sound}>
                            {if *sound_enabled { "🔊 Sound On" } else { "🔇 Sound Off" }}
                        </button>
                        <button class="secondary" onclick={on_toggle_screenshot}>
                            {if *screenshot_mode { "🖼 Exit Screenshot Mode" } else { "📸 Screenshot Mode" }}
                        </button>
                        <button class="secondary" onclick={on_copy_report}>
                            {if *copied { "✅ Copied" } else { "📋 Copy Build Report" }}
                        </button>
                    </div>
                </div>

                <div class="hero-tags">
                    <span class="tag">{"Screenshot-ready UI"}</span>
                    <span class="tag">{"Demo Mode"}</span>
                    <span class="tag">{"Rust Chaos Logs"}</span>
                    <span class="tag">{"Difficulty Modes"}</span>
                </div>
            </section>

            <div class="layout">
                <div class="stack">
                    <section class="card">
                        <div class="card-header">
                            <h2 class="card-title">{"Mobile Build Conditions"}</h2>
                            <p class="card-subtitle">
                                {"Current field telemetry from a development environment powered by timing, grit, and very selective optimism."}
                            </p>
                        </div>

                        <div class="card-body">
                            <div class="badge-grid">
                                <div class="badge">
                                    <div class="badge-label">{"Signal"}</div>
                                    <div class="badge-value">{(*env_signal).clone()}</div>
                                </div>
                                <div class="badge">
                                    <div class="badge-label">{"Battery"}</div>
                                    <div class="badge-value">{format!("{}%", *env_battery)}</div>
                                </div>
                                <div class="badge">
                                    <div class="badge-label">{"Build Target"}</div>
                                    <div class="badge-value">{(*env_target).clone()}</div>
                                </div>
                                <div class="badge">
                                    <div class="badge-label">{"Compiler Mood"}</div>
                                    <div class="badge-value">{(*env_mood).clone()}</div>
                                </div>
                            </div>

                            <div class="section-box section-hide-on-shot">
                                <div class="section-label">{"Carpool Lane Difficulty"}</div>
                                <select onchange={on_change_difficulty} value={difficulty.as_value()}>
                                    <option value="parking_lot">{"Parking Lot"}</option>
                                    <option value="school_pickup">{"School Pickup"}</option>
                                    <option value="rush_hour">{"Rush Hour"}</option>
                                    <option value="iphone_safari_dev_mode">{"iPhone Safari Dev Mode"}</option>
                                    <option value="conference_wifi">{"Conference Wi-Fi"}</option>
                                </select>
                                <div class="card-subtitle" style="margin-top:8px;">
                                    {format!("Current difficulty: {}", difficulty.label())}
                                </div>
                            </div>

                            <div class="controls section-hide-on-shot">
                                <button class="primary" onclick={on_start_build} disabled={*is_building || *demo_mode}>
                                    {if *is_building { "⏳ Building..." } else { "🚗 Start Animated Build" }}
                                </button>
                                <button class="secondary" onclick={on_demo_mode} disabled={*is_building || *demo_mode}>
                                    {if *demo_mode { "🎬 Demo Running..." } else { "🎬 Demo Mode" }}
                                </button>
                                <button class="secondary" onclick={on_add_drama}>{"🎭 Add More Drama"}</button>
                                <button class="secondary" onclick={on_recharge}>{"🔋 Emergency Recharge"}</button>
                                <button class="secondary" onclick={on_toggle_chaos}>
                                    {if *chaos_mode { "🦀 Chaos On" } else { "🙂 Chaos Off" }}
                                </button>
                                <button class="ghost" onclick={on_reset}>{"↺ Reset"}</button>
                            </div>

                            <div class="metrics-grid">
                                <div class="metric">
                                    <div class="metric-label">{"Builds Attempted"}</div>
                                    <div class="metric-value">{*build_count}</div>
                                </div>
                                <div class="metric">
                                    <div class="metric-label">{"Current Progress"}</div>
                                    <div class="metric-value">{format!("{}%", *progress)}</div>
                                </div>
                                <div class="metric">
                                    <div class="metric-label">{"Terminal Status"}</div>
                                    <div class="metric-value">
                                        {if *demo_mode { "Auto Demo" } else if *is_building { "Live Chaos" } else { "Standing By" }}
                                    </div>
                                </div>
                            </div>

                            <div class="section-box">
                                <div class="section-label">{"Build Phase"}</div>
                                <div class="phase-value">{(*phase).clone()}</div>
                                <div class="progress-wrap">
                                    <div class="progress-track">
                                        <div class="progress-fill" style={format!("width:{}%;", *progress)}></div>
                                    </div>
                                    <div class="progress-caption">
                                        {"Measured in percent, interpreted in courage."}
                                    </div>
                                </div>
                            </div>

                            <div class="section-box section-hide-on-shot">
                                <div class="section-label">{"Compiler Excuse of the Day"}</div>
                                <div class="card-subtitle" style="margin-top:0;">{(*excuse).clone()}</div>
                            </div>

                            <div class="section-box">
                                <div class="section-label">{"Safari Resistance Meter"}</div>
                                <div class="meter-header">
                                    <span class="card-subtitle" style="margin:0;">{"How likely the browser is to cooperate"}</span>
                                    <span class={safari_class}>{format!("{} / 100", *safari_resistance)}</span>
                                </div>
                                <div class="progress-wrap">
                                    <div class="progress-track">
                                        <div class="progress-fill" style={format!("width:{}%;", *safari_resistance)}></div>
                                    </div>
                                </div>
                            </div>

                            {result_block}

                            {if *demo_mode {
                                html! {
                                    <div class="legend-banner">
                                        {format!("🎬 Demo Mode in progress • completed runs: {}", *demo_count)}
                                    </div>
                                }
                            } else if *legend_unlocked {
                                html! {
                                    <div class="legend-banner">
                                        {"🏆 Legend unlocked: Persistent Mobile Rust Developer"}
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    </section>

                    <section class="card">
                        <div class="card-header">
                            <h2 class="card-title">{"One More Honest Build"}</h2>
                            <p class="card-subtitle">
                                {"Motivation for developers building under gloriously imperfect conditions."}
                            </p>
                        </div>
                        <div class="card-body">
                            <div class="motivation-quote">{format!("“{}”", (*motivation).clone())}</div>

                            <div class="section-box">
                                <div class="section-label">{"Build Report Preview"}</div>
                                <div class="report-box">{report_preview}</div>
                            </div>

                            <div class="controls section-hide-on-shot">
                                <button class="secondary" onclick={on_download_certificate}>
                                    {if *certificate_flash { "🏆 Certificate Ready" } else { "📜 Download Build Certificate" }}
                                </button>
                                <button class="secondary" onclick={on_generate_share_card}>
                                    {if *share_flash { "✨ Share Card Refreshed" } else { "🖼 Generate Share Card" }}
                                </button>
                            </div>

                            <div class="share-preview">
                                <div class="share-preview-top">
                                    <div class="share-brand">{"MikeGyver Studio • Rust iPhone Compiler Lore"}</div>
                                    <div class="share-brand">{"v5"}</div>
                                </div>

                                <div class="share-title">{share_title}</div>

                                <div class="share-stats">
                                    <div class="share-stat">
                                        <div class="share-stat-label">{"Phase"}</div>
                                        <div class="share-stat-value">{share_phase}</div>
                                    </div>
                                    <div class="share-stat">
                                        <div class="share-stat-label">{"Signal"}</div>
                                        <div class="share-stat-value">{share_signal}</div>
                                    </div>
                                    <div class="share-stat">
                                        <div class="share-stat-label">{"Battery"}</div>
                                        <div class="share-stat-value">{share_battery}</div>
                                    </div>
                                </div>

                                <div class="share-stats">
                                    <div class="share-stat">
                                        <div class="share-stat-label">{"Safari Resistance"}</div>
                                        <div class="share-stat-value">{share_browser}</div>
                                    </div>
                                    <div class="share-stat">
                                        <div class="share-stat-label">{"Progress"}</div>
                                        <div class="share-stat-value">{format!("{}%", *progress)}</div>
                                    </div>
                                    <div class="share-stat">
                                        <div class="share-stat-label">{"Difficulty"}</div>
                                        <div class="share-stat-value">{difficulty.label()}</div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </section>
                </div>

                <section class="terminal">
                    <div class="terminal-topbar">
                        <div class="terminal-dots">
                            <span class="dot"></span>
                            <span class="dot"></span>
                            <span class="dot"></span>
                        </div>
                        <div class="terminal-title">{"carpool-lane-terminal-v5.log"}</div>
                    </div>

                    <div class="terminal-body">
                        {for logs.iter().map(|line| {
                            let class = if line.starts_with("ERROR") {
                                "log-line log-error"
                            } else if line.starts_with("WARN") || line.starts_with("warning") {
                                "log-line log-warn"
                            } else if line.starts_with("SUCCESS") {
                                "log-line log-good"
                            } else {
                                "log-line log-info"
                            };

                            html! { <div class={class}>{line}</div> }
                        })}
                    </div>
                </section>
            </div>

            <div class="footer-note">
                {"Pairs nicely with "}
                <span class="kbd">{"trunk serve"}</span>
                {" and the belief that weird constraints can still produce memorable software."}
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}