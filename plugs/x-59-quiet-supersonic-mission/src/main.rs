use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use js_sys::Math;
use web_sys::{window, KeyboardEvent};
use yew::prelude::*;

const PLANE_X: f64 = 18.0;
const TICK_MS: u32 = 50;

#[derive(Clone, PartialEq)]
enum Mode {
    Colin,
    Luan,
}

impl Mode {
    fn label(&self) -> &'static str {
        match self {
            Self::Colin => "Colin Mode",
            Self::Luan => "Luan Mode",
        }
    }

    fn subtitle(&self) -> &'static str {
        match self {
            Self::Colin => "Smoother skies, more hearts, easier mission flow",
            Self::Luan => "Faster action, tougher turbulence, ace-pilot energy",
        }
    }

    fn hearts(&self) -> i32 {
        match self {
            Self::Colin => 5,
            Self::Luan => 3,
        }
    }

    fn spawn_bias(&self) -> f64 {
        match self {
            Self::Colin => 0.72,
            Self::Luan => 0.92,
        }
    }
}

#[derive(Clone, PartialEq)]
enum ObjectKind {
    Data,
    Turbulence,
    Boost,
}

#[derive(Clone, PartialEq)]
struct FlyingObject {
    id: u32,
    x: f64,
    y: f64,
    kind: ObjectKind,
}

#[derive(Clone, PartialEq)]
struct CardInfo {
    title: &'static str,
    desc: &'static str,
}

fn cards() -> Vec<CardInfo> {
    vec![
        CardInfo {
            title: "Card 1 — Rotate",
            desc: "Build speed, lift off, and get the X-59 airborne.",
        },
        CardInfo {
            title: "Card 2 — Gear Up",
            desc: "Retract the landing gear after takeoff.",
        },
        CardInfo {
            title: "Card 3 — Climb",
            desc: "Capture 20,000 feet and settle into test altitude.",
        },
        CardInfo {
            title: "Card 4 — Hold Speed",
            desc: "Stabilize around 460 mph in smooth flight.",
        },
        CardInfo {
            title: "Card 5 — Step Five",
            desc: "Keep the aircraft steady long enough to clear Step Five.",
        },
        CardInfo {
            title: "Card 8 — Data Run",
            desc: "Collect 5 data stars while dodging turbulence.",
        },
        CardInfo {
            title: "Return & Land",
            desc: "Gear down, slow down, descend, and land safely.",
        },
    ]
}

#[derive(Clone, PartialEq)]
struct GameState {
    running: bool,
    mission_done: bool,
    crashed: bool,
    mode: Mode,
    altitude: f64,
    speed: f64,
    throttle: f64,
    pitch: i32,
    gear_down: bool,
    hearts: i32,
    score: i32,
    card: usize,
    stable_ticks: u32,
    data_collected: u32,
    ticks: u64,
    objects: Vec<FlyingObject>,
    next_id: u32,
    comms: Vec<String>,
    plane_y: f64,
}

impl GameState {
    fn new(mode: Mode) -> Self {
        let mut s = Self {
            running: false,
            mission_done: false,
            crashed: false,
            mode: mode.clone(),
            altitude: 0.0,
            speed: 0.0,
            throttle: 28.0,
            pitch: 0,
            gear_down: true,
            hearts: mode.hearts(),
            score: 0,
            card: 0,
            stable_ticks: 0,
            data_collected: 0,
            ticks: 0,
            objects: vec![],
            next_id: 1,
            comms: vec![],
            plane_y: 84.0,
        };
        s.log("Mission brief loaded. Bring up throttle and prepare to rotate.");
        s
    }

    fn log(&mut self, msg: impl Into<String>) {
        self.comms.push(msg.into());
        if self.comms.len() > 7 {
            self.comms.remove(0);
        }
    }

    fn reset_for_mode(&mut self, mode: Mode) {
        *self = Self::new(mode);
    }

    fn set_pitch(&mut self, pitch: i32) {
        self.pitch = pitch.clamp(-1, 1);
    }

    fn throttle_up(&mut self) {
        self.throttle = (self.throttle + 6.0).clamp(0.0, 100.0);
    }

    fn throttle_down(&mut self) {
        self.throttle = (self.throttle - 6.0).clamp(0.0, 100.0);
    }

    fn toggle_gear(&mut self) {
        self.gear_down = !self.gear_down;
        if self.gear_down {
            self.log("Gear down.");
        } else {
            self.log("Gear in three, two, one. Now.");
        }
    }

    fn start(&mut self) {
        self.running = true;
        self.mission_done = false;
        self.crashed = false;
        self.log(format!("{} engaged. Quiet skies mission is go.", self.mode.label()));
    }

    fn restart(&mut self) {
        let mode = self.mode.clone();
        *self = Self::new(mode);
        self.running = true;
        self.log("Reset complete. Try the mission again.");
    }

    fn plane_class(&self) -> &'static str {
        match self.pitch {
            1 => "plane climb",
            -1 => "plane dive",
            _ => "plane level",
        }
    }

    fn phase_name(&self) -> &'static str {
        match self.card {
            0 => "Takeoff Roll",
            1 => "Gear Up",
            2 => "Climb",
            3 => "Hold 460",
            4 => "Step Five",
            5 => "Card Eight",
            _ => "Return & Land",
        }
    }
}

fn rand_range(min: f64, max: f64) -> f64 {
    min + Math::random() * (max - min)
}

fn spawn_object(game: &mut GameState) {
    let roll = Math::random();
    let y = rand_range(16.0, 78.0);

    let kind = if game.card == 5 {
        if roll < 0.56 {
            ObjectKind::Data
        } else if roll < 0.80 {
            ObjectKind::Boost
        } else {
            ObjectKind::Turbulence
        }
    } else {
        let turbulence_bias = game.mode.spawn_bias();
        if roll > turbulence_bias {
            ObjectKind::Boost
        } else if roll > 0.42 {
            ObjectKind::Turbulence
        } else {
            ObjectKind::Data
        }
    };

    game.objects.push(FlyingObject {
        id: game.next_id,
        x: 108.0,
        y,
        kind,
    });
    game.next_id += 1;
}

fn handle_input(game: &mut GameState, key: &str) {
    match key {
        "ArrowUp" | "w" | "W" => game.set_pitch(1),
        "ArrowDown" | "s" | "S" => game.set_pitch(-1),
        "ArrowLeft" | "a" | "A" => game.throttle_down(),
        "ArrowRight" | "d" | "D" => game.throttle_up(),
        " " => {
            if !game.running && !game.mission_done && !game.crashed {
                game.start();
            } else {
                game.set_pitch(0);
            }
        }
        "g" | "G" => game.toggle_gear(),
        "l" | "L" => game.set_pitch(0),
        _ => {}
    }
}

fn step_game(game: &mut GameState) {
    if !game.running || game.mission_done || game.crashed {
        return;
    }

    game.ticks += 1;

    let gear_penalty = if game.gear_down { 30.0 } else { 0.0 };
    let pitch_penalty = (game.pitch.abs() as f64) * 10.0;
    let target_speed = (110.0 + game.throttle * 5.2 - gear_penalty - pitch_penalty).clamp(80.0, 620.0);

    game.speed += (target_speed - game.speed) * 0.065;

    let lift = ((game.speed - 150.0) / 330.0).clamp(0.0, 1.0);
    let base_sink = if game.altitude > 0.0 { 6.0 - lift * 4.0 } else { 0.0 };
    let pitch_delta = match game.pitch {
        1 => 52.0 * lift,
        -1 => -64.0 * (lift + 0.15),
        _ => -base_sink,
    };

    if game.altitude <= 0.0 && game.speed < 175.0 && game.pitch >= 0 {
        game.altitude = 0.0;
    } else {
        game.altitude = (game.altitude + pitch_delta).clamp(0.0, 30000.0);
    }

    if game.altitude <= 0.0 && game.card >= 2 && game.speed > 255.0 && !game.gear_down {
        game.crashed = true;
        game.running = false;
        game.log("Too hot for a safe runway return. Mission lost.");
        return;
    }

    if game.altitude > 0.0 && game.speed < 118.0 {
        game.hearts -= 1;
        game.speed = 140.0;
        game.stable_ticks = 0;
        game.log("Airspeed too low. Recovering from a wobble.");
        if game.hearts <= 0 {
            game.crashed = true;
            game.running = false;
            game.log("The X-59 could not recover. Mission failed.");
            return;
        }
    }

    game.plane_y = (84.0 - (game.altitude / 30000.0) * 70.0).clamp(9.0, 86.0);

    if game.ticks % 20 == 0 {
        spawn_object(game);
    }

    let world_speed = 1.25 + game.speed / 230.0;
    let mut remaining = Vec::with_capacity(game.objects.len());

    for mut obj in game.objects.clone() {
        obj.x -= world_speed;

        let hit = (obj.x - PLANE_X).abs() < 6.5 && (obj.y - game.plane_y).abs() < 8.5;

        if hit {
            match obj.kind {
                ObjectKind::Turbulence => {
                    game.hearts -= 1;
                    game.speed = (game.speed - 22.0).max(100.0);
                    game.stable_ticks = 0;
                    game.log("Uh, control shows some rough air. Hold speed just fine.");
                    if game.hearts <= 0 {
                        game.crashed = true;
                        game.running = false;
                        game.log("Too much turbulence. Mission failed.");
                        return;
                    }
                }
                ObjectKind::Data => {
                    game.score += 25;
                    if game.card == 5 {
                        game.data_collected += 1;
                        game.log(format!(
                            "Card Eight data packet captured: {}/5.",
                            game.data_collected
                        ));
                    } else {
                        game.log("Telemetry star captured.");
                    }
                }
                ObjectKind::Boost => {
                    game.score += 12;
                    game.throttle = (game.throttle + 8.0).clamp(0.0, 100.0);
                    game.log("Energy boost collected.");
                }
            }
        } else if obj.x > -8.0 {
            remaining.push(obj);
        }
    }

    game.objects = remaining;

    match game.card {
        0 => {
            if game.speed > 180.0 && game.altitude > 220.0 {
                game.card = 1;
                game.score += 75;
                game.stable_ticks = 0;
                game.log("Airborne. Retract the gear.");
            }
        }
        1 => {
            if !game.gear_down && game.altitude > 1000.0 {
                game.card = 2;
                game.score += 100;
                game.stable_ticks = 0;
                game.log("Nice. Climb and capture 20,000 feet.");
            }
        }
        2 => {
            if (19000.0..=21000.0).contains(&game.altitude) {
                game.stable_ticks += 1;
                if game.stable_ticks > 26 {
                    game.card = 3;
                    game.score += 125;
                    game.stable_ticks = 0;
                    game.log("Altitude captured. Now hold about 460 mph.");
                }
            } else {
                game.stable_ticks = 0;
            }
        }
        3 => {
            if (435.0..=485.0).contains(&game.speed) && (18500.0..=21500.0).contains(&game.altitude) {
                game.stable_ticks += 1;
                if game.stable_ticks > 80 {
                    game.card = 4;
                    game.score += 160;
                    game.stable_ticks = 0;
                    game.log("Control shows Step Five good for now.");
                }
            } else {
                game.stable_ticks = 0;
            }
        }
        4 => {
            if (430.0..=480.0).contains(&game.speed) && (18000.0..=22000.0).contains(&game.altitude) {
                game.stable_ticks += 1;
                if game.stable_ticks > 110 {
                    game.card = 5;
                    game.score += 200;
                    game.stable_ticks = 0;
                    game.log("Ready to proceed to Card Eight. Gather 5 data stars.");
                }
            } else {
                game.stable_ticks = 0;
            }
        }
        5 => {
            if game.data_collected >= 5 {
                game.card = 6;
                game.score += 250;
                game.log("Card Eight complete. Return to base and land.");
            }
        }
        _ => {
            if game.altitude < 2500.0 && game.speed < 220.0 && game.gear_down {
                game.mission_done = true;
                game.running = false;
                game.score += game.hearts.max(0) * 50;
                game.log("Mission complete. Quiet skies secured.");
            }
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let game = use_state(|| GameState::new(Mode::Colin));

    {
        let game = game.clone();
        use_effect(move || {
            let interval = Interval::new(TICK_MS, move || {
                let mut next = (*game).clone();
                step_game(&mut next);
                game.set(next);
            });

            move || drop(interval)
        });
    }

    {
        let game = game.clone();
        use_effect(move || {
            let listener = window().map(|win| {
                EventListener::new(&win, "keydown", move |event| {
                    if let Some(e) = event.dyn_ref::<KeyboardEvent>() {
                        let key = e.key();
                        if matches!(
                            key.as_str(),
                            "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | " " | "g" | "G" | "w" | "W" | "a" | "A" | "s" | "S" | "d" | "D" | "l" | "L"
                        ) {
                            e.prevent_default();
                        }
                        let mut next = (*game).clone();
                        handle_input(&mut next, &key);
                        game.set(next);
                    }
                })
            });

            move || drop(listener)
        });
    }

    let current = (*game).clone();
    let mission_cards = cards();

    let on_mode_colin = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.reset_for_mode(Mode::Colin);
            game.set(next);
        })
    };

    let on_mode_luan = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.reset_for_mode(Mode::Luan);
            game.set(next);
        })
    };

    let on_start = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.start();
            game.set(next);
        })
    };

    let on_restart = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.restart();
            game.set(next);
        })
    };

    let on_pitch_up = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.set_pitch(1);
            game.set(next);
        })
    };

    let on_pitch_level = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.set_pitch(0);
            game.set(next);
        })
    };

    let on_pitch_down = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.set_pitch(-1);
            game.set(next);
        })
    };

    let on_throttle_up = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.throttle_up();
            game.set(next);
        })
    };

    let on_throttle_down = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.throttle_down();
            game.set(next);
        })
    };

    let on_toggle_gear = {
        let game = game.clone();
        Callback::from(move |_| {
            let mut next = (*game).clone();
            next.toggle_gear();
            game.set(next);
        })
    };

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="eyebrow">{"MikeGyver Studio • Rust iPhone Compiler Build"}</div>
                <h1>{"X-59: Quiet Supersonic Mission"}</h1>
                <p>
                    {"A fun NASA-inspired flight test game for Colin and Luan. Take off, retract gear, climb to 20,000 feet, hold around 460 mph, clear Step Five, finish Card Eight, and bring the X-59 home."}
                </p>
            </section>

            <section class="grid-top">
                <div class="panel">
                    <div class="panel-header">
                        <div class="panel-title">{"Mission Setup"}</div>
                    </div>
                    <div class="panel-body">
                        <div class="mode-row">
                            <button class={classes!("chip-btn", (matches!(current.mode, Mode::Colin)).then_some("active"))} onclick={on_mode_colin}>
                                {"Colin Mode"}
                            </button>
                            <button class={classes!("chip-btn", (matches!(current.mode, Mode::Luan)).then_some("active"))} onclick={on_mode_luan}>
                                {"Luan Mode"}
                            </button>
                        </div>

                        <div class="main-actions">
                            <button class="main-btn primary" onclick={on_start}>{"Start Mission"}</button>
                            <button class="main-btn gold" onclick={on_restart}>{"Reset / Replay"}</button>
                        </div>

                        <div class="legend">
                            <div class="legend-item">
                                <span class="legend-dot data"></span>
                                {"Data Star = score + Card Eight progress"}
                            </div>
                            <div class="legend-item">
                                <span class="legend-dot turbulence"></span>
                                {"Turbulence = lose a heart"}
                            </div>
                            <div class="legend-item">
                                <span class="legend-dot boost"></span>
                                {"Boost = free energy bump"}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="panel">
                    <div class="panel-header">
                        <div class="panel-title">{"Pilot Help"}</div>
                    </div>
                    <div class="panel-body">
                        <div class="quick-help">
                            <div class="help-card">
                                <b>{"Pitch"}</b>
                                <span>{"Arrow Up / Down or the touch buttons below. Level out with Space or Level."}</span>
                            </div>
                            <div class="help-card">
                                <b>{"Throttle"}</b>
                                <span>{"Arrow Left / Right or A / D to manage speed."}</span>
                            </div>
                            <div class="help-card">
                                <b>{"Gear"}</b>
                                <span>{"Press G or tap Gear to retract after takeoff and lower before landing."}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="game-panel">
                <div class="stats-grid">
                    <div class="stat">
                        <div class="stat-label">{"Altitude"}</div>
                        <div class="stat-value cyan">{format!("{:.0} ft", current.altitude)}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Speed"}</div>
                        <div class="stat-value gold">{format!("{:.0} mph", current.speed)}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Throttle"}</div>
                        <div class="stat-value">{format!("{:.0}%", current.throttle)}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Hearts"}</div>
                        <div class={classes!("stat-value", if current.hearts <= 1 { "red" } else { "green" })}>
                            {format!("{}", current.hearts)}
                        </div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Phase"}</div>
                        <div class="stat-value cyan">{current.phase_name()}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Score"}</div>
                        <div class="stat-value">{current.score}</div>
                    </div>
                </div>

                <div class="controls-panel">
                    <div class="controls-grid">
                        <button class="ctrl-btn" onclick={on_pitch_up}>
                            {"⬆ Nose Up"}
                            <span>{"Climb"}</span>
                        </button>
                        <button class="ctrl-btn" onclick={on_pitch_level}>
                            {"⏺ Level"}
                            <span>{"Steady"}</span>
                        </button>
                        <button class="ctrl-btn" onclick={on_pitch_down}>
                            {"⬇ Nose Down"}
                            <span>{"Descend"}</span>
                        </button>
                        <button class="ctrl-btn" onclick={on_throttle_up}>
                            {"➕ Throttle"}
                            <span>{"Speed Up"}</span>
                        </button>
                        <button class="ctrl-btn" onclick={on_throttle_down}>
                            {"➖ Throttle"}
                            <span>{"Slow Down"}</span>
                        </button>
                    </div>

                    <div class="main-actions" style="margin-top:10px;">
                        <button class="main-btn" onclick={on_toggle_gear}>
                            { if current.gear_down { "Gear Down" } else { "Gear Up" } }
                        </button>
                    </div>
                </div>

                <div class="footer-note">
                    {format!(
                        "{} — {}",
                        current.mode.label(),
                        current.mode.subtitle()
                    )}
                </div>

                <div class="sky" style="margin-top:14px;">
                    <div class="horizon"></div>
                    <div class="runway"></div>

                    {
                        for current.objects.iter().map(|obj| {
                            let class_name = match obj.kind {
                                ObjectKind::Data => "object data",
                                ObjectKind::Turbulence => "object turbulence",
                                ObjectKind::Boost => "object boost",
                            };

                            html! {
                                <div
                                    key={obj.id}
                                    class={class_name}
                                    style={format!("left:{:.2}%; top:{:.2}%;", obj.x, obj.y)}
                                />
                            }
                        })
                    }

                    <div
                        class={current.plane_class()}
                        style={format!("top:{:.2}%;", current.plane_y)}
                    >
                        <div class="plane-tail"></div>
                        <div class="plane-wing"></div>
                        <div class="plane-body"></div>
                        <div class="plane-mark">{"X-59"}</div>
                        {
                            if current.gear_down {
                                html! { <div class="plane-gear"></div> }
                            } else {
                                html! {}
                            }
                        }
                    </div>

                    {
                        if !current.running && !current.mission_done && !current.crashed {
                            html! {
                                <div class="overlay">
                                    <div class="overlay-card">
                                        <h2>{"Ready for Quiet Supersonic Mission?"}</h2>
                                        <p>
                                            {"Start rolling, lift off, retract the gear, climb to 20,000 feet, hold around 460 mph, clear Step Five, grab 5 data stars for Card Eight, and then land safely."}
                                        </p>
                                        <div class="overlay-stats">
                                            <div class="overlay-badge">{current.mode.label()}</div>
                                            <div class="overlay-badge">{"Target altitude: 20,000 ft"}</div>
                                            <div class="overlay-badge">{"Target speed: ~460 mph"}</div>
                                        </div>
                                    </div>
                                </div>
                            }
                        } else if current.mission_done {
                            html! {
                                <div class="overlay">
                                    <div class="overlay-card">
                                        <h2>{"Mission Complete!"}</h2>
                                        <p>
                                            {"You brought the X-59 through the test cards and back to the runway. Quiet skies secured."}
                                        </p>
                                        <div class="overlay-stats">
                                            <div class="overlay-badge">{format!("Final score: {}", current.score)}</div>
                                            <div class="overlay-badge">{format!("Hearts left: {}", current.hearts)}</div>
                                            <div class="overlay-badge">{format!("Data stars: {}", current.data_collected)}</div>
                                        </div>
                                    </div>
                                </div>
                            }
                        } else if current.crashed {
                            html! {
                                <div class="overlay">
                                    <div class="overlay-card">
                                        <h2>{"Mission Failed"}</h2>
                                        <p>
                                            {"Too much turbulence or a bad return spoiled the run. Reset and try another flight card."}
                                        </p>
                                        <div class="overlay-stats">
                                            <div class="overlay-badge">{format!("Score: {}", current.score)}</div>
                                            <div class="overlay-badge">{format!("Card reached: {}", current.card + 1)}</div>
                                        </div>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </section>

            <section class="bottom-grid">
                <div class="panel">
                    <div class="panel-header">
                        <div class="panel-title">{"Flight Cards"}</div>
                    </div>
                    <div class="panel-body">
                        <div class="cards-list">
                            {
                                for mission_cards.iter().enumerate().map(|(idx, c)| {
                                    let row_class = if idx < current.card {
                                        "card-row done"
                                    } else if idx == current.card {
                                        "card-row active"
                                    } else {
                                        "card-row"
                                    };

                                    let status_class = if idx < current.card {
                                        "status-badge status-done"
                                    } else if idx == current.card {
                                        "status-badge status-active"
                                    } else {
                                        "status-badge status-up"
                                    };

                                    let status_text = if idx < current.card {
                                        "Done"
                                    } else if idx == current.card {
                                        "Active"
                                    } else {
                                        "Up Next"
                                    };

                                    html! {
                                        <div class={row_class}>
                                            <div class="card-top">
                                                <span>{c.title}</span>
                                                <span class={status_class}>{status_text}</span>
                                            </div>
                                            <div class="card-desc">{c.desc}</div>
                                        </div>
                                    }
                                })
                            }
                        </div>
                    </div>
                </div>

                <div class="panel">
                    <div class="panel-header">
                        <div class="panel-title">{"Comms & Mission Feed"}</div>
                    </div>
                    <div class="panel-body">
                        <div class="comms-list">
                            {
                                for current.comms.iter().rev().map(|msg| {
                                    html! { <div class="comms-item">{msg.clone()}</div> }
                                })
                            }
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}