use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use web_sys::window;
use yew::prelude::*;

const WORLD_WIDTH: f64 = 1400.0;
const WORLD_HEIGHT: f64 = 900.0;

const EARTH_X: f64 = 260.0;
const EARTH_Y: f64 = 470.0;
const EARTH_R: f64 = 85.0;

const MOON_X: f64 = 1120.0;
const MOON_Y: f64 = 360.0;
const MOON_R: f64 = 62.0;

const TRAIL_MAX: usize = 120;
const SPLASHDOWN_RING_R: f64 = EARTH_R + 110.0;
const SHIP_TOUCH_R: f64 = 16.0;
const SPLASHDOWN_TOUCH_DISTANCE: f64 = EARTH_R + SHIP_TOUCH_R;
const SPLASHDOWN_MAX_SPEED: f64 = 105.0;

fn log_msg(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

#[derive(Clone, PartialEq)]
struct Ship {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    fuel: f64,
    angle: f64,
}

impl Default for Ship {
    fn default() -> Self {
        Self {
            x: 420.0,
            y: 470.0,
            vx: 0.0,
            vy: 0.0,
            fuel: 100.0,
            angle: 0.0,
        }
    }
}

#[derive(Clone, PartialEq)]
struct TrailPoint {
    x: f64,
    y: f64,
}

#[derive(Clone, PartialEq)]
struct Star {
    id: usize,
    x: f64,
    y: f64,
    size: f64,
    opacity: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    ToMoon,
    ToHome,
}

#[derive(Clone, PartialEq, Default)]
struct KeyState {
    forward: bool,
    slow: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    burn: bool,
}

#[derive(Clone, PartialEq)]
struct GameState {
    ship: Ship,
    phase: Phase,
    started: bool,
    message: String,
    burn_meter: f64,
    reached_moon: bool,
    landed_home: bool,
    score: i32,
    trail: Vec<TrailPoint>,
    keys: KeyState,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            ship: Ship::default(),
            phase: Phase::ToMoon,
            started: false,
            message: "Tap Start Mission, then use the touch controls to fly Orion.".to_string(),
            burn_meter: 0.0,
            reached_moon: false,
            landed_home: false,
            score: 0,
            trail: vec![],
            keys: KeyState::default(),
        }
    }
}

fn clamp(n: f64, min: f64, max: f64) -> f64 {
    n.max(min).min(max)
}

fn dist_xy(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

fn speed(ship: &Ship) -> f64 {
    (ship.vx.powi(2) + ship.vy.powi(2)).sqrt()
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::ToMoon => "Moon Trip",
        Phase::ToHome => "Home Trip",
    }
}

fn phase_badge_class(phase: Phase) -> &'static str {
    match phase {
        Phase::ToMoon => "badge badge-phase-moon",
        Phase::ToHome => "badge badge-phase-home",
    }
}

fn percent_bar(value: f64, fill: &str) -> Html {
    let width = clamp(value, 0.0, 100.0);
    html! {
        <div class="bar-track">
            <div class="bar-fill" style={format!("width:{width}%;background:{fill};")}></div>
        </div>
    }
}

fn make_stars() -> Vec<Star> {
    (0..90)
        .map(|i| Star {
            id: i,
            x: ((i * 173) as f64) % WORLD_WIDTH,
            y: ((i * 97) as f64) % WORLD_HEIGHT,
            size: ((i % 3) + 1) as f64,
            opacity: 0.35 + (((i * 11) % 10) as f64 / 20.0),
        })
        .collect()
}

fn tick_game(game: &mut GameState, dt: f64) {
    if !game.started {
        return;
    }

    let thrust = 340.0;
    let strafe = 240.0;
    let friction = 0.991;
    let burn_power = 520.0;
    let fuel_use = 11.0;
    let can_use_fuel = game.ship.fuel > 0.0;

    if game.keys.forward && can_use_fuel {
        game.ship.vx += thrust * dt;
        game.ship.angle = 0.0;
        game.ship.fuel = clamp(game.ship.fuel - fuel_use * dt, 0.0, 100.0);
    }

    if game.keys.slow && can_use_fuel {
        game.ship.vx -= thrust * dt;
        game.ship.angle = 180.0;
        game.ship.fuel = clamp(game.ship.fuel - fuel_use * dt * 0.85, 0.0, 100.0);
    }

    if game.keys.left && can_use_fuel {
        game.ship.vx -= strafe * dt;
        game.ship.angle = 180.0;
        game.ship.fuel = clamp(game.ship.fuel - fuel_use * dt * 0.65, 0.0, 100.0);
    }

    if game.keys.right && can_use_fuel {
        game.ship.vx += strafe * dt;
        game.ship.angle = 0.0;
        game.ship.fuel = clamp(game.ship.fuel - fuel_use * dt * 0.65, 0.0, 100.0);
    }

    if game.keys.up && can_use_fuel {
        game.ship.vy -= strafe * dt;
        game.ship.angle = 270.0;
        game.ship.fuel = clamp(game.ship.fuel - fuel_use * dt * 0.65, 0.0, 100.0);
    }

    if game.keys.down && can_use_fuel {
        game.ship.vy += strafe * dt;
        game.ship.angle = 90.0;
        game.ship.fuel = clamp(game.ship.fuel - fuel_use * dt * 0.65, 0.0, 100.0);
    }

    if game.keys.burn && can_use_fuel {
        let direction = if game.phase == Phase::ToMoon { 1.0 } else { -1.0 };
        game.ship.vx += direction * burn_power * dt;
        game.ship.fuel = clamp(game.ship.fuel - fuel_use * dt * 1.7, 0.0, 100.0);
        game.burn_meter = clamp(game.burn_meter + 32.0 * dt, 0.0, 100.0);
    } else {
        game.burn_meter = clamp(game.burn_meter - 20.0 * dt, 0.0, 100.0);
    }

    let moon_pull = 24.0 / dist_xy(game.ship.x, game.ship.y, MOON_X, MOON_Y).max(120.0);
    let earth_pull = 20.0 / dist_xy(game.ship.x, game.ship.y, EARTH_X, EARTH_Y).max(120.0);

    if game.phase == Phase::ToMoon {
        game.ship.vx += ((MOON_X - game.ship.x) / 400.0) * moon_pull * dt;
        game.ship.vy += ((MOON_Y - game.ship.y) / 400.0) * moon_pull * dt;
    } else {
        game.ship.vx += ((EARTH_X - game.ship.x) / 400.0) * earth_pull * dt;
        game.ship.vy += ((EARTH_Y - game.ship.y) / 400.0) * earth_pull * dt;
    }

    game.ship.vx *= friction;
    game.ship.vy *= friction;

    game.ship.x += game.ship.vx * dt;
    game.ship.y += game.ship.vy * dt;

    game.ship.x = clamp(game.ship.x, 20.0, WORLD_WIDTH - 20.0);
    game.ship.y = clamp(game.ship.y, 20.0, WORLD_HEIGHT - 20.0);

    game.trail.push(TrailPoint {
        x: game.ship.x,
        y: game.ship.y,
    });

    if game.trail.len() > TRAIL_MAX {
        let remove_count = game.trail.len() - TRAIL_MAX;
        game.trail.drain(0..remove_count);
    }

    let moon_distance = dist_xy(game.ship.x, game.ship.y, MOON_X, MOON_Y);
    let earth_center_distance = dist_xy(game.ship.x, game.ship.y, EARTH_X, EARTH_Y);
    let earth_distance = (earth_center_distance - EARTH_R).max(0.0);
    let current_speed = speed(&game.ship);

    if game.phase == Phase::ToMoon && moon_distance < MOON_R + 78.0 && !game.reached_moon {
        game.reached_moon = true;
        game.score += 50 + game.ship.fuel.round() as i32;
        game.phase = Phase::ToHome;
        game.message = "Nice Moon flyby! Now head back and touch Earth gently for splashdown.".to_string();
    }

    if game.phase == Phase::ToHome
        && earth_center_distance <= SPLASHDOWN_TOUCH_DISTANCE
        && current_speed < SPLASHDOWN_MAX_SPEED
        && !game.landed_home
    {
        game.landed_home = true;
        game.started = false;
        game.ship.vx = 0.0;
        game.ship.vy = 0.0;
        game.score += 100 + (game.ship.fuel * 1.5).round() as i32;
        game.message = "Splashdown success! Tap Start Mission to play again.".to_string();
    }

    if game.phase == Phase::ToHome && !game.landed_home {
        if earth_center_distance <= SPLASHDOWN_TOUCH_DISTANCE && current_speed >= SPLASHDOWN_MAX_SPEED {
            game.message = format!(
                "Too fast for splashdown. Slow under {:.0}! Current speed: {:.0}",
                SPLASHDOWN_MAX_SPEED,
                current_speed
            );
        } else if earth_distance <= SPLASHDOWN_RING_R - EARTH_R {
            game.message = "You are lined up. Now touch Earth gently.".to_string();
        } else {
            game.message = "Guide Orion into the splashdown ring around Earth.".to_string();
        }
    }

    if game.ship.fuel <= 0.0 && !game.landed_home {
        game.message = "Fuel is empty! Coast carefully and use small touch adjustments.".to_string();
    }
}

fn set_control(keys: &mut KeyState, name: &str, pressed: bool) {
    match name {
        "forward" => keys.forward = pressed,
        "slow" => keys.slow = pressed,
        "left" => keys.left = pressed,
        "right" => keys.right = pressed,
        "up" => keys.up = pressed,
        "down" => keys.down = pressed,
        "burn" => keys.burn = pressed,
        _ => {}
    }
}

fn control_active(keys: &KeyState, name: &str) -> bool {
    match name {
        "forward" => keys.forward,
        "slow" => keys.slow,
        "left" => keys.left,
        "right" => keys.right,
        "up" => keys.up,
        "down" => keys.down,
        "burn" => keys.burn,
        _ => false,
    }
}

#[derive(Properties, PartialEq)]
struct PadButtonProps {
    label: AttrValue,
    control: AttrValue,
    class_name: AttrValue,
    game_ref: UseMutRefHandle<GameState>,
    force_render: Callback<()>,
    active: bool,
}

#[function_component(PadButton)]
fn pad_button(props: &PadButtonProps) -> Html {
    let on_press = {
        let game_ref = props.game_ref.clone();
        let force_render = props.force_render.clone();
        let control = props.control.to_string();
        Callback::from(move |_| {
            let mut game = game_ref.borrow_mut();
            set_control(&mut game.keys, &control, true);
            if !game.started && !game.landed_home {
                game.started = true;
                game.message = "Mission started from touch controls. Fly to the Moon!".to_string();
            }
            force_render.emit(());
        })
    };

    let on_release = {
        let game_ref = props.game_ref.clone();
        let force_render = props.force_render.clone();
        let control = props.control.to_string();
        Callback::from(move |_| {
            let mut game = game_ref.borrow_mut();
            set_control(&mut game.keys, &control, false);
            force_render.emit(());
        })
    };

    let classes = if props.active {
        format!("pad-btn {} active", props.class_name)
    } else {
        format!("pad-btn {}", props.class_name)
    };

    html! {
        <button
            class={classes}
            onpointerdown={on_press.clone()}
            onpointerup={on_release.clone()}
            onpointerleave={on_release.clone()}
            onpointercancel={on_release.clone()}
            ontouchstart={on_press}
            ontouchend={on_release.clone()}
            ontouchcancel={on_release}
        >
            {props.label.clone()}
        </button>
    }
}

#[function_component(App)]
fn app() -> Html {
    let game_ref = use_mut_ref(GameState::default);
    let stars = use_state(make_stars);
    let render_tick = use_state(|| 0_u64);

    let force_render = {
        let render_tick = render_tick.clone();
        Callback::from(move |_| {
            render_tick.set(*render_tick + 1);
        })
    };

    {
        let game_ref = game_ref.clone();
        let force_render = force_render.clone();

        use_effect_with((), move |_| {
            let blur_listener = window().map(|win| {
                let game_ref = game_ref.clone();
                let force_render = force_render.clone();
                EventListener::new(&win, "blur", move |_| {
                    let mut game = game_ref.borrow_mut();
                    game.keys = KeyState::default();
                    force_render.emit(());
                })
            });

            move || {
                drop(blur_listener);
            }
        });
    }

    {
        let game_ref = game_ref.clone();
        let force_render = force_render.clone();

        use_effect_with((), move |_| {
            log_msg("creating permanent mobile game loop interval");

            let interval = Interval::new(16, move || {
                let mut should_render = false;

                {
                    let mut game = game_ref.borrow_mut();
                    if game.started {
                        tick_game(&mut game, 0.016);
                        should_render = true;
                    }
                }

                if should_render {
                    force_render.emit(());
                }
            });

            move || {
                drop(interval);
            }
        });
    }

    let on_start = {
        let game_ref = game_ref.clone();
        let force_render = force_render.clone();
        Callback::from(move |_| {
            let mut game = game_ref.borrow_mut();

            if game.landed_home {
                *game = GameState::default();
                game.started = true;
                game.message = "New mission started! Fly to the Moon, then return for splashdown.".to_string();
            } else {
                game.started = true;
                game.message = "Mission started! Use the touch controls below to fly Orion.".to_string();
            }

            force_render.emit(());
        })
    };

    let on_reset = {
        let game_ref = game_ref.clone();
        let force_render = force_render.clone();
        Callback::from(move |_| {
            *game_ref.borrow_mut() = GameState::default();
            force_render.emit(());
        })
    };

    let game = game_ref.borrow().clone();

    let moon_distance =
        (dist_xy(game.ship.x, game.ship.y, MOON_X, MOON_Y) - MOON_R).max(0.0).round();
    let earth_distance =
        (dist_xy(game.ship.x, game.ship.y, EARTH_X, EARTH_Y) - EARTH_R).max(0.0).round();
    let current_speed = speed(&game.ship).round();

    let start_label = if game.landed_home {
        "Play Again"
    } else {
        "Start Mission"
    };

    let debug_text = format!(
        "started={} | x={:.1} y={:.1} | vx={:.2} vy={:.2} | speed={:.1}",
        game.started,
        game.ship.x,
        game.ship.y,
        game.ship.vx,
        game.ship.vy,
        speed(&game.ship)
    );

    html! {
        <div class="app-shell">
            <div class="layout">
                <section class="card main-card">
                    <div class="header-row">
                        <div>
                            <div class="title">{"🚀 Fly the Orion Mobile"}</div>
                            <div class="subtitle">
                                {"A touch-friendly Rust + Yew Moon mission for iPhone and mobile browsers."}
                            </div>
                        </div>

                        <div class="badges">
                            <div class={phase_badge_class(game.phase)}>
                                {format!("Phase: {}", phase_label(game.phase))}
                            </div>
                            <div class="badge badge-score">
                                {format!("Score: {}", game.score)}
                            </div>
                        </div>
                    </div>

                    <div class="main-content">
                        <div class="mission-message">
                            {game.message.clone()}
                            <div style="margin-top:8px;color:#94a3b8;font-size:12px;">{debug_text}</div>
                        </div>

                        <div class="space-box">
                            <svg class="space-svg" viewBox={format!("0 0 {} {}", WORLD_WIDTH, WORLD_HEIGHT)}>
                                {for stars.iter().map(|s| html! {
                                    <circle
                                        key={s.id.to_string()}
                                        cx={s.x.to_string()}
                                        cy={s.y.to_string()}
                                        r={s.size.to_string()}
                                        fill="white"
                                        opacity={s.opacity.to_string()}
                                    />
                                })}

                                {for game.trail.iter().enumerate().map(|(i, p)| {
                                    let opacity = if game.trail.is_empty() {
                                        0.0
                                    } else {
                                        i as f64 / game.trail.len() as f64
                                    };

                                    html! {
                                        <circle
                                            key={format!("trail-{}-{}-{}", i, p.x, p.y)}
                                            cx={p.x.to_string()}
                                            cy={p.y.to_string()}
                                            r="2"
                                            fill="#67e8f9"
                                            opacity={opacity.to_string()}
                                        />
                                    }
                                })}

                                {if game.phase == Phase::ToHome && !game.landed_home {
                                    html! {
                                        <>
                                            <circle
                                                cx={EARTH_X.to_string()}
                                                cy={EARTH_Y.to_string()}
                                                r={SPLASHDOWN_RING_R.to_string()}
                                                fill="none"
                                                stroke="#7dd3fc"
                                                stroke-dasharray="10 10"
                                                stroke-width="3"
                                                opacity="0.35"
                                            />
                                            <text
                                                x={(EARTH_X - 74.0).to_string()}
                                                y={(EARTH_Y - SPLASHDOWN_RING_R - 12.0).to_string()}
                                                fill="#7dd3fc"
                                                font-size="18"
                                                font-weight="700"
                                                opacity="0.8"
                                            >
                                                {"Splashdown Zone"}
                                            </text>
                                        </>
                                    }
                                } else {
                                    html! {}
                                }}

                                <circle cx={EARTH_X.to_string()} cy={EARTH_Y.to_string()} r={EARTH_R.to_string()} fill="#2563eb" />
                                <circle cx={(EARTH_X - 18.0).to_string()} cy={(EARTH_Y - 14.0).to_string()} r="18" fill="#22c55e" opacity="0.85" />
                                <circle cx={(EARTH_X + 20.0).to_string()} cy={(EARTH_Y + 24.0).to_string()} r="12" fill="#22c55e" opacity="0.75" />
                                <text x={(EARTH_X - 28.0).to_string()} y={(EARTH_Y + 120.0).to_string()} fill="#bfdbfe" font-size="26" font-weight="700">{"Earth"}</text>

                                <circle cx={MOON_X.to_string()} cy={MOON_Y.to_string()} r={MOON_R.to_string()} fill="#d1d5db" />
                                <circle cx={(MOON_X - 16.0).to_string()} cy={(MOON_Y - 8.0).to_string()} r="9" fill="#9ca3af" />
                                <circle cx={(MOON_X + 14.0).to_string()} cy={(MOON_Y + 12.0).to_string()} r="7" fill="#9ca3af" />
                                <circle cx={(MOON_X - 2.0).to_string()} cy={(MOON_Y + 20.0).to_string()} r="6" fill="#9ca3af" />
                                <text x={(MOON_X - 24.0).to_string()} y={(MOON_Y + 100.0).to_string()} fill="#e5e7eb" font-size="26" font-weight="700">{"Moon"}</text>

                                {if game.phase == Phase::ToMoon {
                                    html! {
                                        <path
                                            d={format!("M {} {} Q 720 200 {} {}", EARTH_X + 90.0, EARTH_Y - 20.0, MOON_X - 80.0, MOON_Y + 10.0)}
                                            fill="none"
                                            stroke="#38bdf8"
                                            stroke-dasharray="12 12"
                                            stroke-width="4"
                                            opacity="0.45"
                                        />
                                    }
                                } else {
                                    html! {
                                        <path
                                            d={format!("M {} {} Q 780 760 {} {}", MOON_X - 40.0, MOON_Y + 80.0, EARTH_X + 95.0, EARTH_Y + 5.0)}
                                            fill="none"
                                            stroke="#f59e0b"
                                            stroke-dasharray="12 12"
                                            stroke-width="4"
                                            opacity="0.45"
                                        />
                                    }
                                }}

                                <g transform={format!("translate({}, {}) rotate({}) scale(1.2)", game.ship.x, game.ship.y, game.ship.angle)}>
                                    <polygon points="16,0 -10,10 -4,0 -10,-10" fill="#f8fafc" stroke="#0f172a" stroke-width="2" />
                                    <polygon points="-4,0 -14,5 -12,0 -14,-5" fill="#93c5fd" opacity="0.9" />
                                    <polygon points="-10,10 -18,18 -7,8" fill="#f59e0b" />
                                    <polygon points="-10,-10 -18,-18 -7,-8" fill="#f59e0b" />
                                    <circle cx="2" cy="0" r="3.5" fill="#60a5fa" />
                                </g>

                                {if game.burn_meter > 5.0 {
                                    html! {
                                        <g>
                                            <circle
                                                cx={(game.ship.x - 22.0).to_string()}
                                                cy={game.ship.y.to_string()}
                                                r={(14.0 + game.burn_meter / 12.0).to_string()}
                                                fill="#fb923c"
                                                opacity="0.35"
                                            />
                                            <circle
                                                cx={(game.ship.x - 28.0).to_string()}
                                                cy={game.ship.y.to_string()}
                                                r={(8.0 + game.burn_meter / 18.0).to_string()}
                                                fill="#fde68a"
                                                opacity="0.55"
                                            />
                                        </g>
                                    }
                                } else {
                                    html! {}
                                }}
                            </svg>
                        </div>

                        <div class="stats-grid">
                            <div class="panel-mini">
                                <div class="stat-label">{"Fuel"}</div>
                                <div class="stat-value">{format!("{}%", game.ship.fuel.round())}</div>
                                {percent_bar(game.ship.fuel, "linear-gradient(90deg,#22c55e,#38bdf8)")}
                            </div>

                            <div class="panel-mini">
                                <div class="stat-label">{"Speed"}</div>
                                <div class="stat-value">{current_speed.to_string()}</div>
                                <div class="stat-note">{"Touch Earth gently under the safe speed target."}</div>
                            </div>

                            <div class="panel-mini">
                                <div class="stat-label">{"Burn"}</div>
                                <div class="stat-value">{format!("{}%", game.burn_meter.round())}</div>
                                {percent_bar(game.burn_meter, "linear-gradient(90deg,#f59e0b,#fb923c)")}
                            </div>
                        </div>

                        <div class="mobile-pad">
                            <div class="pad-group">
                                <div class="pad-title">{"Directional Controls"}</div>
                                <div class="dpad-grid">
                                    <div></div>
                                    <PadButton
                                        label="↑"
                                        control="up"
                                        class_name=""
                                        game_ref={game_ref.clone()}
                                        force_render={force_render.clone()}
                                        active={control_active(&game.keys, "up")}
                                    />
                                    <div></div>

                                    <PadButton
                                        label="←"
                                        control="left"
                                        class_name=""
                                        game_ref={game_ref.clone()}
                                        force_render={force_render.clone()}
                                        active={control_active(&game.keys, "left")}
                                    />
                                    <PadButton
                                        label="→ GO"
                                        control="forward"
                                        class_name="forward"
                                        game_ref={game_ref.clone()}
                                        force_render={force_render.clone()}
                                        active={control_active(&game.keys, "forward")}
                                    />
                                    <PadButton
                                        label="→"
                                        control="right"
                                        class_name=""
                                        game_ref={game_ref.clone()}
                                        force_render={force_render.clone()}
                                        active={control_active(&game.keys, "right")}
                                    />

                                    <div></div>
                                    <PadButton
                                        label="↓"
                                        control="down"
                                        class_name=""
                                        game_ref={game_ref.clone()}
                                        force_render={force_render.clone()}
                                        active={control_active(&game.keys, "down")}
                                    />
                                    <div></div>
                                </div>
                            </div>

                            <div class="pad-group">
                                <div class="pad-title">{"Action Controls"}</div>
                                <div class="action-grid">
                                    <PadButton
                                        label="SLOW"
                                        control="slow"
                                        class_name="slow"
                                        game_ref={game_ref.clone()}
                                        force_render={force_render.clone()}
                                        active={control_active(&game.keys, "slow")}
                                    />
                                    <PadButton
                                        label="🔥 BURN"
                                        control="burn"
                                        class_name="burn"
                                        game_ref={game_ref.clone()}
                                        force_render={force_render.clone()}
                                        active={control_active(&game.keys, "burn")}
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                <section class="side-column">
                    <div class="card side-card">
                        <div class="section-title">{"⭐ Mission Coach"}</div>

                        <div class="distance-grid">
                            <div class="kpi-box">
                                <div class="kpi-label">{"Distance to Moon"}</div>
                                <div class="kpi-value">{moon_distance.to_string()}</div>
                            </div>

                            <div class="kpi-box">
                                <div class="kpi-label">{"Distance to Earth"}</div>
                                <div class="kpi-value">{earth_distance.to_string()}</div>
                            </div>
                        </div>

                        <div class="button-row">
                            <button class="button button-primary" onclick={on_start}>
                                {start_label}
                            </button>
                            <button class="button button-secondary" onclick={on_reset}>
                                {"Reset"}
                            </button>
                        </div>
                    </div>

                    <div class="card side-card">
                        <div class="section-title">{"How to Play"}</div>
                        <div class="control-list">
                            <div class="control-item">
                                <div class="control-title">{"GO"}</div>
                                <div class="control-text">{"Push Orion forward toward the Moon or back home."}</div>
                            </div>
                            <div class="control-item">
                                <div class="control-title">{"SLOW"}</div>
                                <div class="control-text">{"Use this near Earth to reduce speed for splashdown."}</div>
                            </div>
                            <div class="control-item">
                                <div class="control-title">{"← → ↑ ↓"}</div>
                                <div class="control-text">{"Fine-tune your position with side and vertical thrusters."}</div>
                            </div>
                            <div class="control-item">
                                <div class="control-title">{"BURN"}</div>
                                <div class="control-text">{"Big boost for deep-space travel. Use carefully."}</div>
                            </div>
                        </div>
                    </div>

                    <div class="card side-card">
                        <div class="section-title">{"Mission Goals"}</div>
                        <div class="goal-list">
                            <div class="goal-item">{"🌕 Reach the Moon and complete the flyby."}</div>
                            <div class="goal-item">{"🌍 Return to Earth and line up with the splashdown ring."}</div>
                            <div class="goal-item">{format!("🚀 Touch Earth under {:.0} speed for a safe splashdown.", SPLASHDOWN_MAX_SPEED)}</div>
                        </div>
                    </div>
                </section>
            </div>

            <div class="footer-note">
                {"Mobile version: touch-first controls for iPhone and small screens. Hosted separately for the best experience."}
            </div>
        </div>
    }
}

fn main() {
    log_msg("main(): rendering Fly the Orion Mobile");
    yew::Renderer::<App>::new().render();
}