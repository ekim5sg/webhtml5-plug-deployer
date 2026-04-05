use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use js_sys::Math;
use web_sys::window;
use yew::prelude::*;

const TICK_MS: u32 = 250;
const HISTORY_MAX: usize = 64;
const MOBILE_BREAKPOINT: f64 = 900.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MissionPhase {
    Launch,
    OrbitCheckout,
    Tli,
    CoastOut,
    LunarFlyby,
    CoastHome,
    ReturnBurn,
    Reentry,
}

impl MissionPhase {
    fn all() -> [MissionPhase; 8] {
        [
            MissionPhase::Launch,
            MissionPhase::OrbitCheckout,
            MissionPhase::Tli,
            MissionPhase::CoastOut,
            MissionPhase::LunarFlyby,
            MissionPhase::CoastHome,
            MissionPhase::ReturnBurn,
            MissionPhase::Reentry,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            MissionPhase::Launch => "Launch",
            MissionPhase::OrbitCheckout => "Orbit Checkout",
            MissionPhase::Tli => "Trans-Lunar Injection",
            MissionPhase::CoastOut => "Outbound Coast",
            MissionPhase::LunarFlyby => "Lunar Flyby",
            MissionPhase::CoastHome => "Return Coast",
            MissionPhase::ReturnBurn => "Return Burn",
            MissionPhase::Reentry => "Reentry",
        }
    }

    fn short_label(&self) -> &'static str {
        match self {
            MissionPhase::Launch => "Launch",
            MissionPhase::OrbitCheckout => "Checkout",
            MissionPhase::Tli => "TLI",
            MissionPhase::CoastOut => "Coast Out",
            MissionPhase::LunarFlyby => "Flyby",
            MissionPhase::CoastHome => "Coast Home",
            MissionPhase::ReturnBurn => "Return Burn",
            MissionPhase::Reentry => "Reentry",
        }
    }

    fn duration_s(&self) -> f64 {
        match self {
            MissionPhase::Launch => 150.0,
            MissionPhase::OrbitCheckout => 210.0,
            MissionPhase::Tli => 160.0,
            MissionPhase::CoastOut => 300.0,
            MissionPhase::LunarFlyby => 120.0,
            MissionPhase::CoastHome => 300.0,
            MissionPhase::ReturnBurn => 120.0,
            MissionPhase::Reentry => 140.0,
        }
    }

    fn guidance_apollo(&self, p: f64) -> &'static str {
        match self {
            MissionPhase::Launch => {
                if p < 0.35 {
                    "ASC GUID"
                } else if p < 0.8 {
                    "S-IVB STG"
                } else {
                    "INSERT"
                }
            }
            MissionPhase::OrbitCheckout => {
                if p < 0.5 {
                    "CSM CHK"
                } else {
                    "ORB OPS"
                }
            }
            MissionPhase::Tli => {
                if p < 0.2 {
                    "PREP TLI"
                } else if p < 0.85 {
                    "BURN EXEC"
                } else {
                    "POST TLI"
                }
            }
            MissionPhase::CoastOut => {
                if p < 0.4 {
                    "PTC INIT"
                } else {
                    "CST NAV"
                }
            }
            MissionPhase::LunarFlyby => {
                if p < 0.5 {
                    "LOI TRK"
                } else {
                    "FREE RET"
                }
            }
            MissionPhase::CoastHome => "CST NAV",
            MissionPhase::ReturnBurn => {
                if p < 0.8 {
                    "TEI EXEC"
                } else {
                    "RBURN END"
                }
            }
            MissionPhase::Reentry => {
                if p < 0.35 {
                    "ENTRY IF"
                } else if p < 0.7 {
                    "CM ENTRY"
                } else {
                    "CHUTE DEP"
                }
            }
        }
    }

    fn guidance_orion(&self, p: f64) -> &'static str {
        match self {
            MissionPhase::Launch => {
                if p < 0.35 {
                    "Ascent Guidance"
                } else if p < 0.8 {
                    "Core Stage Flight"
                } else {
                    "Orbit Insertion"
                }
            }
            MissionPhase::OrbitCheckout => {
                if p < 0.5 {
                    "Systems Checkout"
                } else {
                    "Parking Orbit Ops"
                }
            }
            MissionPhase::Tli => {
                if p < 0.2 {
                    "Burn Preparation"
                } else if p < 0.85 {
                    "TLI Burn Active"
                } else {
                    "Burn Complete"
                }
            }
            MissionPhase::CoastOut => {
                if p < 0.4 {
                    "Attitude Stabilization"
                } else {
                    "Deep Space Navigation"
                }
            }
            MissionPhase::LunarFlyby => {
                if p < 0.5 {
                    "Flyby Geometry"
                } else {
                    "Free Return Track"
                }
            }
            MissionPhase::CoastHome => "Return Navigation",
            MissionPhase::ReturnBurn => {
                if p < 0.8 {
                    "Return Burn Active"
                } else {
                    "Return Burn Complete"
                }
            }
            MissionPhase::Reentry => {
                if p < 0.35 {
                    "Entry Interface"
                } else if p < 0.7 {
                    "Guided Reentry"
                } else {
                    "Parachute Sequence"
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MobilePanel {
    Apollo,
    Center,
    Orion,
}

#[derive(Clone, PartialEq)]
struct MissionState {
    mission_time_s: f64,
    phase: MissionPhase,
    phase_progress: f64,
    altitude_km: f64,
    velocity_kps: f64,
    downrange_km: f64,
    distance_from_earth_km: f64,
    distance_to_moon_km: f64,
    fuel_pct: f64,
    power_pct: f64,
    cabin_temp_c: f64,
    pitch_deg: f64,
    yaw_deg: f64,
    roll_deg: f64,
    comm_link_pct: f64,
}

#[derive(Clone, PartialEq)]
struct ApolloDisplay {
    phase: &'static str,
    met: String,
    vel_kps: String,
    alt_km: String,
    downrange_km: String,
    fuel_pct: String,
    power_pct: String,
    temp_c: String,
    pitch: String,
    yaw: String,
    roll: String,
    guidance: &'static str,
    comm: &'static str,
    log_line: &'static str,
    banner: &'static str,
    lamp_guid: bool,
    lamp_comm: bool,
    lamp_prop: bool,
}

#[derive(Clone, PartialEq)]
struct OrionDisplay {
    phase: &'static str,
    met: String,
    velocity: String,
    altitude: String,
    downrange: String,
    earth_distance: String,
    moon_distance: String,
    propellant: String,
    battery: String,
    temp: String,
    attitude: String,
    guidance: &'static str,
    comm: &'static str,
    log_line: &'static str,
    banner: &'static str,
    mode_chip: &'static str,
    nav_chip: &'static str,
    power_chip: &'static str,
    overlay_distance_earth: String,
    overlay_distance_moon: String,
    overlay_velocity: String,
    overlay_met: String,
    overlay_caption: &'static str,
}

#[derive(Clone, PartialEq)]
struct LogEntry {
    met: String,
    text: String,
}

#[derive(Clone, Copy, PartialEq)]
struct ChartPoint {
    value: f64,
}

#[derive(Clone, PartialEq)]
struct HistoryState {
    altitude: Vec<ChartPoint>,
    velocity: Vec<ChartPoint>,
    comm: Vec<ChartPoint>,
}

fn window_width() -> f64 {
    window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(1200.0)
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    v.max(lo).min(hi)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn ease_in_out(t: f64) -> f64 {
    let t = clamp(t, 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn km_to_miles(km: f64) -> f64 {
    km * 0.621_371
}

fn kps_to_mph(kps: f64) -> f64 {
    kps * 2236.936
}

fn format_met_overlay(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as i64;
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let minutes = (total % 3_600) / 60;
    format!("{}D : {}H : {}M", days, hours, minutes)
}

fn mission_duration_total() -> f64 {
    MissionPhase::all().iter().map(|p| p.duration_s()).sum()
}

fn phase_start_time(target: MissionPhase) -> f64 {
    let mut total = 0.0;
    for phase in MissionPhase::all() {
        if phase == target {
            break;
        }
        total += phase.duration_s();
    }
    total
}

fn find_phase_and_progress(t: f64) -> (MissionPhase, f64) {
    let mut remaining = t;
    for phase in MissionPhase::all() {
        let d = phase.duration_s();
        if remaining <= d {
            return (phase, clamp(remaining / d, 0.0, 1.0));
        }
        remaining -= d;
    }
    (MissionPhase::Reentry, 1.0)
}

fn tiny_noise(amplitude: f64) -> f64 {
    (Math::random() - 0.5) * 2.0 * amplitude
}

fn format_met(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as i64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn build_mission_state(t: f64) -> MissionState {
    let total = mission_duration_total();
    let mission_time_s = clamp(t, 0.0, total);
    let (phase, raw_p) = find_phase_and_progress(mission_time_s);
    let p = ease_in_out(raw_p);

    let (
        altitude_km,
        velocity_kps,
        downrange_km,
        distance_from_earth_km,
        distance_to_moon_km,
        fuel_pct,
        power_pct,
        cabin_temp_c,
        pitch_deg,
        yaw_deg,
        roll_deg,
        comm_link_pct,
    ) = match phase {
        MissionPhase::Launch => {
            let alt = lerp(0.0, 185.0, p) + tiny_noise(0.8);
            let vel = lerp(0.0, 7.8, p) + tiny_noise(0.03);
            let dr = lerp(0.0, 2200.0, p) + tiny_noise(8.0);
            let earth = alt;
            let moon = 384400.0 - earth;
            let fuel = lerp(100.0, 84.0, p) + tiny_noise(0.2);
            let power = lerp(100.0, 99.0, p) + tiny_noise(0.05);
            let temp = lerp(22.0, 24.0, p) + tiny_noise(0.15);
            let pitch = lerp(90.0, 10.0, p) + tiny_noise(0.6);
            let yaw = lerp(0.0, 1.2, p) + tiny_noise(0.2);
            let roll = lerp(0.0, 3.5, p) + tiny_noise(0.3);
            let comm = lerp(94.0, 98.0, p) + tiny_noise(0.2);
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
        MissionPhase::OrbitCheckout => {
            let alt = 185.0 + tiny_noise(1.5);
            let vel = 7.78 + tiny_noise(0.03);
            let dr = lerp(2200.0, 9000.0, p) + tiny_noise(14.0);
            let earth = alt;
            let moon = 384400.0 - earth;
            let fuel = lerp(84.0, 82.0, p) + tiny_noise(0.15);
            let power = lerp(99.0, 97.5, p) + tiny_noise(0.08);
            let temp = lerp(24.0, 23.0, p) + tiny_noise(0.1);
            let pitch = lerp(10.0, 0.0, p) + tiny_noise(0.3);
            let yaw = lerp(1.2, 0.2, p) + tiny_noise(0.15);
            let roll = lerp(3.5, 0.4, p) + tiny_noise(0.2);
            let comm = lerp(98.0, 99.0, p) + tiny_noise(0.15);
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
        MissionPhase::Tli => {
            let alt = lerp(185.0, 320.0, p) + tiny_noise(1.2);
            let vel = lerp(7.8, 10.9, p) + tiny_noise(0.05);
            let dr = lerp(9000.0, 21000.0, p) + tiny_noise(18.0);
            let earth = lerp(185.0, 22000.0, p) + tiny_noise(20.0);
            let moon = 384400.0 - earth;
            let fuel = lerp(82.0, 65.0, p) + tiny_noise(0.2);
            let power = lerp(97.5, 96.0, p) + tiny_noise(0.08);
            let temp = lerp(23.0, 24.5, p) + tiny_noise(0.12);
            let pitch = lerp(0.0, -4.0, p) + tiny_noise(0.35);
            let yaw = lerp(0.2, 0.0, p) + tiny_noise(0.12);
            let roll = lerp(0.4, 1.0, p) + tiny_noise(0.18);
            let comm = lerp(99.0, 97.0, p) + tiny_noise(0.18);
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
        MissionPhase::CoastOut => {
            let alt = lerp(320.0, 5000.0, p) + tiny_noise(6.0);
            let vel = lerp(10.9, 1.4, p) + tiny_noise(0.04);
            let dr = lerp(21000.0, 180000.0, p) + tiny_noise(40.0);
            let earth = lerp(22000.0, 320000.0, p) + tiny_noise(75.0);
            let moon = clamp(384400.0 - earth, 0.0, 384400.0);
            let fuel = lerp(65.0, 61.0, p) + tiny_noise(0.12);
            let power = lerp(96.0, 92.0, p) + tiny_noise(0.09);
            let temp = lerp(24.5, 22.8, p) + tiny_noise(0.12);
            let pitch = lerp(-4.0, 0.0, p) + tiny_noise(0.2);
            let yaw = lerp(0.0, 0.4, p) + tiny_noise(0.1);
            let roll = lerp(1.0, 359.0, p) + tiny_noise(0.35);
            let comm = lerp(97.0, 93.0, p) + tiny_noise(0.25);
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
        MissionPhase::LunarFlyby => {
            let alt = lerp(5000.0, 9000.0, p) + tiny_noise(6.0);
            let vel = lerp(1.4, 2.2, p) + tiny_noise(0.03);
            let dr = lerp(180000.0, 215000.0, p) + tiny_noise(35.0);
            let earth = lerp(320000.0, 384400.0, p) + tiny_noise(40.0);
            let moon = clamp(384400.0 - earth, 0.0, 384400.0);
            let fuel = lerp(61.0, 60.0, p) + tiny_noise(0.08);
            let power = lerp(92.0, 91.0, p) + tiny_noise(0.08);
            let temp = lerp(22.8, 23.4, p) + tiny_noise(0.1);
            let pitch = lerp(0.0, 14.0, p) + tiny_noise(0.2);
            let yaw = lerp(0.4, -0.5, p) + tiny_noise(0.1);
            let roll = lerp(359.0, 2.0, p) + tiny_noise(0.3);
            let comm = lerp(93.0, 90.0, p) + tiny_noise(0.3);
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
        MissionPhase::CoastHome => {
            let alt = lerp(9000.0, 3000.0, p) + tiny_noise(6.0);
            let vel = lerp(2.2, 1.7, p) + tiny_noise(0.03);
            let dr = lerp(215000.0, 380000.0, p) + tiny_noise(45.0);
            let earth = lerp(384400.0, 90000.0, p) + tiny_noise(85.0);
            let moon = clamp(384400.0 - earth, 0.0, 384400.0);
            let fuel = lerp(60.0, 56.0, p) + tiny_noise(0.12);
            let power = lerp(91.0, 87.0, p) + tiny_noise(0.09);
            let temp = lerp(23.4, 22.2, p) + tiny_noise(0.1);
            let pitch = lerp(14.0, -2.0, p) + tiny_noise(0.2);
            let yaw = lerp(-0.5, 0.2, p) + tiny_noise(0.1);
            let roll = lerp(2.0, 356.0, p) + tiny_noise(0.35);
            let comm = lerp(90.0, 95.0, p) + tiny_noise(0.24);
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
        MissionPhase::ReturnBurn => {
            let alt = lerp(3000.0, 1200.0, p) + tiny_noise(3.0);
            let vel = lerp(1.7, 3.1, p) + tiny_noise(0.03);
            let dr = lerp(380000.0, 402000.0, p) + tiny_noise(28.0);
            let earth = lerp(90000.0, 22000.0, p) + tiny_noise(40.0);
            let moon = clamp(384400.0 - earth, 0.0, 384400.0);
            let fuel = lerp(56.0, 52.0, p) + tiny_noise(0.1);
            let power = lerp(87.0, 85.0, p) + tiny_noise(0.07);
            let temp = lerp(22.2, 23.2, p) + tiny_noise(0.1);
            let pitch = lerp(-2.0, -12.0, p) + tiny_noise(0.2);
            let yaw = lerp(0.2, 0.0, p) + tiny_noise(0.08);
            let roll = lerp(356.0, 1.0, p) + tiny_noise(0.28);
            let comm = lerp(95.0, 96.0, p) + tiny_noise(0.18);
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
        MissionPhase::Reentry => {
            let alt = lerp(120.0, 2.0, p) + tiny_noise(1.0);
            let vel = lerp(11.1, 0.25, p) + tiny_noise(0.06);
            let dr = lerp(402000.0, 405500.0, p) + tiny_noise(12.0);
            let earth = alt;
            let moon = 384400.0 - earth;
            let fuel = lerp(52.0, 50.0, p) + tiny_noise(0.05);
            let power = lerp(85.0, 83.0, p) + tiny_noise(0.06);
            let temp = lerp(23.2, 27.0, p) + tiny_noise(0.2);
            let pitch = lerp(-12.0, 84.0, p) + tiny_noise(0.8);
            let yaw = lerp(0.0, 0.5, p) + tiny_noise(0.15);
            let roll = lerp(1.0, 0.0, p) + tiny_noise(0.22);
            let comm = if p < 0.35 {
                lerp(96.0, 8.0, p / 0.35) + tiny_noise(0.5)
            } else if p < 0.7 {
                4.0 + tiny_noise(1.0)
            } else {
                lerp(30.0, 98.0, (p - 0.7) / 0.3) + tiny_noise(0.6)
            };
            (alt, vel, dr, earth, moon, fuel, power, temp, pitch, yaw, roll, comm)
        }
    };

    MissionState {
        mission_time_s,
        phase,
        phase_progress: raw_p,
        altitude_km: clamp(altitude_km, 0.0, 999_999.0),
        velocity_kps: clamp(velocity_kps, 0.0, 99.0),
        downrange_km: clamp(downrange_km, 0.0, 999_999.0),
        distance_from_earth_km: clamp(distance_from_earth_km, 0.0, 999_999.0),
        distance_to_moon_km: clamp(distance_to_moon_km, 0.0, 999_999.0),
        fuel_pct: clamp(fuel_pct, 0.0, 100.0),
        power_pct: clamp(power_pct, 0.0, 100.0),
        cabin_temp_c,
        pitch_deg,
        yaw_deg,
        roll_deg: if roll_deg < 0.0 { 360.0 + roll_deg } else { roll_deg },
        comm_link_pct: clamp(comm_link_pct, 0.0, 100.0),
    }
}

fn apollo_comm_label(phase: MissionPhase, pct: f64) -> &'static str {
    if phase == MissionPhase::Reentry && pct < 15.0 {
        "LOS"
    } else if pct < 55.0 {
        "WEAK"
    } else {
        "ACQ"
    }
}

fn orion_comm_label(phase: MissionPhase, pct: f64) -> &'static str {
    if phase == MissionPhase::Reentry && pct < 15.0 {
        "Blackout Expected"
    } else if pct < 55.0 {
        "Degraded Link"
    } else {
        "Nominal Link"
    }
}

fn apollo_log_line(phase: MissionPhase, p: f64) -> &'static str {
    match phase {
        MissionPhase::Launch => {
            if p < 0.25 {
                "LIFTOFF COMMITTED"
            } else if p < 0.6 {
                "BOOSTER PERFORMANCE NOMINAL"
            } else {
                "EARTH PARKING ORBIT APPROACHING"
            }
        }
        MissionPhase::OrbitCheckout => {
            if p < 0.5 { "CSM SYSTEMS CHECK IN WORK" } else { "ORBIT OPS STABLE" }
        }
        MissionPhase::Tli => {
            if p < 0.2 {
                "PREPARING TLI BURN"
            } else if p < 0.85 {
                "S-IVB BURN IN PROGRESS"
            } else {
                "TLI COMPLETE — FREE RETURN TRACK"
            }
        }
        MissionPhase::CoastOut => {
            if p < 0.4 { "PTC ROLL ESTABLISHED" } else { "MIDCOURSE NAV UPDATE" }
        }
        MissionPhase::LunarFlyby => {
            if p < 0.5 { "LUNAR PERICYNTHION APPROACH" } else { "FREE RETURN HOMEBOUND" }
        }
        MissionPhase::CoastHome => {
            if p < 0.5 { "DEEP SPACE TRACKING STEADY" } else { "ENTRY TARGET REFINE" }
        }
        MissionPhase::ReturnBurn => {
            if p < 0.8 { "RETURN BURN HOLDING PROFILE" } else { "RETURN BURN COMPLETE" }
        }
        MissionPhase::Reentry => {
            if p < 0.35 {
                "ENTRY INTERFACE"
            } else if p < 0.7 {
                "EXPECTED COM BLACKOUT"
            } else {
                "CHUTES GOOD — SPLASHDOWN NEXT"
            }
        }
    }
}

fn orion_log_line(phase: MissionPhase, p: f64) -> &'static str {
    match phase {
        MissionPhase::Launch => {
            if p < 0.25 {
                "Vehicle committed to ascent corridor."
            } else if p < 0.6 {
                "Ascent performance remains within family."
            } else {
                "Orbit insertion sequence approaching."
            }
        }
        MissionPhase::OrbitCheckout => {
            if p < 0.5 { "Avionics, power, and thermal checks continue." } else { "Parking orbit operations stabilized." }
        }
        MissionPhase::Tli => {
            if p < 0.2 {
                "Trans-lunar injection burn prep complete."
            } else if p < 0.85 {
                "TLI burn active. Guidance residuals low."
            } else {
                "Outbound trajectory established."
            }
        }
        MissionPhase::CoastOut => {
            if p < 0.4 { "Attitude stabilization complete." } else { "Deep-space navigation update converged." }
        }
        MissionPhase::LunarFlyby => {
            if p < 0.5 { "Closest approach geometry tightening." } else { "Free-return corridor confirmed homeward." }
        }
        MissionPhase::CoastHome => {
            if p < 0.5 { "Return coast consumables remain healthy." } else { "Entry targeting solution refined." }
        }
        MissionPhase::ReturnBurn => {
            if p < 0.8 { "Return burn active and holding expected profile." } else { "Return burn complete." }
        }
        MissionPhase::Reentry => {
            if p < 0.35 {
                "Entry interface crossing."
            } else if p < 0.7 {
                "Communications blackout expected during plasma phase."
            } else {
                "Parachute sequence underway."
            }
        }
    }
}

fn apollo_banner(phase: MissionPhase) -> &'static str {
    match phase {
        MissionPhase::Launch => "Apollo-era instrumentation compresses launch into terse ascent guidance cues and staged commit calls.",
        MissionPhase::OrbitCheckout => "Parking orbit looks procedural here: systems checks, orbital ops, no broadcast flair.",
        MissionPhase::Tli => "The same departure burn becomes terse burn execution language on the Apollo side.",
        MissionPhase::CoastOut => "Deep space is conveyed through stable nav labels and restrained telemetry movement.",
        MissionPhase::LunarFlyby => "Closest approach is expressed as tracking geometry and free-return confidence.",
        MissionPhase::CoastHome => "Return coast remains sparse, procedural, and consumables-aware.",
        MissionPhase::ReturnBurn => "Return burn is shown as another controlled execution profile, not a cinematic event.",
        MissionPhase::Reentry => "Reentry becomes entry interface, blackout expectation, and chute deployment milestones.",
    }
}

fn orion_banner(phase: MissionPhase) -> &'static str {
    match phase {
        MissionPhase::Launch => "Modern Orion presentation blends internal flight software cues with audience-friendly mission language.",
        MissionPhase::OrbitCheckout => "Parking orbit adds systems context, healthier status chips, and clearer mission framing.",
        MissionPhase::Tli => "The same burn is rendered as a modern departure event with guidance and mission overlay context.",
        MissionPhase::CoastOut => "Outbound coast becomes a clean deep-space broadcast moment with distance and elapsed-time graphics.",
        MissionPhase::LunarFlyby => "Lunar flyby is rendered as geometry, mission optics, and audience-readable telemetry.",
        MissionPhase::CoastHome => "Return coast emphasizes navigation confidence, health, and long-range tracking continuity.",
        MissionPhase::ReturnBurn => "Return burn is framed as corridor shaping and Earth approach setup.",
        MissionPhase::Reentry => "Reentry adds public-facing blackout expectations while retaining technical GN&C state.",
    }
}

fn orion_overlay_caption(phase: MissionPhase) -> &'static str {
    match phase {
        MissionPhase::Launch => "Orion is centered and climbing under powered ascent.",
        MissionPhase::OrbitCheckout => "Orion is centered and stabilized in parking orbit for systems checkout.",
        MissionPhase::Tli => "Orion is centered as translunar injection pushes the vehicle beyond low Earth orbit.",
        MissionPhase::CoastOut => "Orion is centered. Orion's solar array wings are unfurled and swept back.",
        MissionPhase::LunarFlyby => "Orion is centered as it sweeps through lunar flyby geometry.",
        MissionPhase::CoastHome => "Orion is centered on the return leg with deep-space tracking active.",
        MissionPhase::ReturnBurn => "Orion is centered as return burn reshapes the Earth approach corridor.",
        MissionPhase::Reentry => "Orion is centered on final return with entry and recovery sequence underway.",
    }
}

fn to_apollo_display(state: &MissionState) -> ApolloDisplay {
    ApolloDisplay {
        phase: state.phase.short_label(),
        met: format_met(state.mission_time_s),
        vel_kps: format!("{:.2} km/s", state.velocity_kps),
        alt_km: format!("{:.0} km", state.altitude_km),
        downrange_km: format!("{:.0} km", state.downrange_km),
        fuel_pct: format!("{:.0}%", state.fuel_pct),
        power_pct: format!("{:.0}%", state.power_pct),
        temp_c: format!("{:.1}°C", state.cabin_temp_c),
        pitch: format!("{:.1}°", state.pitch_deg),
        yaw: format!("{:.1}°", state.yaw_deg),
        roll: format!("{:.1}°", state.roll_deg),
        guidance: state.phase.guidance_apollo(state.phase_progress),
        comm: apollo_comm_label(state.phase, state.comm_link_pct),
        log_line: apollo_log_line(state.phase, state.phase_progress),
        banner: apollo_banner(state.phase),
        lamp_guid: true,
        lamp_comm: state.comm_link_pct > 50.0,
        lamp_prop: state.fuel_pct > 20.0,
    }
}

fn to_orion_display(state: &MissionState) -> OrionDisplay {
    OrionDisplay {
        phase: state.phase.label(),
        met: format_met(state.mission_time_s),
        velocity: format!("{:.2} km/s", state.velocity_kps),
        altitude: format!("{:.0} km", state.altitude_km),
        downrange: format!("{:.0} km", state.downrange_km),
        earth_distance: format!("{:.0} km", state.distance_from_earth_km),
        moon_distance: format!("{:.0} km", state.distance_to_moon_km),
        propellant: format!("{:.1}%", state.fuel_pct),
        battery: format!("{:.1}%", state.power_pct),
        temp: format!("{:.1}°C", state.cabin_temp_c),
        attitude: format!(
            "P {:.1}°  Y {:.1}°  R {:.1}°",
            state.pitch_deg, state.yaw_deg, state.roll_deg
        ),
        guidance: state.phase.guidance_orion(state.phase_progress),
        comm: orion_comm_label(state.phase, state.comm_link_pct),
        log_line: orion_log_line(state.phase, state.phase_progress),
        banner: orion_banner(state.phase),
        mode_chip: match state.phase {
            MissionPhase::Launch => "Ascent Mode",
            MissionPhase::OrbitCheckout => "Parking Orbit",
            MissionPhase::Tli => "Departure Burn",
            MissionPhase::CoastOut => "Deep Space Outbound",
            MissionPhase::LunarFlyby => "Flyby Geometry",
            MissionPhase::CoastHome => "Deep Space Return",
            MissionPhase::ReturnBurn => "Return Burn",
            MissionPhase::Reentry => "Entry / Recovery",
        },
        nav_chip: if state.phase == MissionPhase::Reentry && state.comm_link_pct < 15.0 {
            "Plasma Blackout"
        } else {
            "GN&C Stable"
        },
        power_chip: if state.power_pct > 85.0 { "Power Nominal" } else { "Power Conservation" },
        overlay_distance_earth: format!("{:.0} mi", km_to_miles(state.distance_from_earth_km)),
        overlay_distance_moon: format!("{:.0} mi", km_to_miles(state.distance_to_moon_km)),
        overlay_velocity: format!("{:.0} mph", kps_to_mph(state.velocity_kps)),
        overlay_met: format_met_overlay(state.mission_time_s),
        overlay_caption: orion_overlay_caption(state.phase),
    }
}

fn base_log_entries() -> Vec<LogEntry> {
    vec![LogEntry {
        met: "00:00:00".into(),
        text: "Simulation initialized. Shared mission model driving Apollo and Orion instrumentation.".into(),
    }]
}

fn base_history() -> HistoryState {
    HistoryState {
        altitude: vec![],
        velocity: vec![],
        comm: vec![],
    }
}

fn push_history(history: &mut HistoryState, state: &MissionState) {
    history.altitude.push(ChartPoint { value: state.altitude_km });
    history.velocity.push(ChartPoint { value: state.velocity_kps });
    history.comm.push(ChartPoint { value: state.comm_link_pct });

    if history.altitude.len() > HISTORY_MAX {
        history.altitude.remove(0);
    }
    if history.velocity.len() > HISTORY_MAX {
        history.velocity.remove(0);
    }
    if history.comm.len() > HISTORY_MAX {
        history.comm.remove(0);
    }
}

fn push_log(logs: &mut Vec<LogEntry>, state: &MissionState) {
    let phase = state.phase;
    let p = state.phase_progress;

    let interesting = match phase {
        MissionPhase::Launch => (p > 0.10 && p < 0.16) || (p > 0.55 && p < 0.61),
        MissionPhase::OrbitCheckout => p > 0.45 && p < 0.51,
        MissionPhase::Tli => (p > 0.08 && p < 0.14) || (p > 0.86 && p < 0.92),
        MissionPhase::CoastOut => (p > 0.35 && p < 0.41) || (p > 0.74 && p < 0.80),
        MissionPhase::LunarFlyby => p > 0.48 && p < 0.54,
        MissionPhase::CoastHome => (p > 0.20 && p < 0.26) || (p > 0.75 && p < 0.81),
        MissionPhase::ReturnBurn => (p > 0.10 && p < 0.16) || (p > 0.84 && p < 0.90),
        MissionPhase::Reentry => (p > 0.25 && p < 0.31) || (p > 0.74 && p < 0.80),
    };

    if !interesting {
        return;
    }

    let new_text = format!("{} | {}", phase.short_label(), orion_log_line(phase, p));
    let met = format_met(state.mission_time_s);

    let should_add = match logs.last() {
        Some(last) => last.text != new_text,
        None => true,
    };

    if should_add {
        logs.push(LogEntry { met, text: new_text });
        if logs.len() > 18 {
            let overflow = logs.len() - 18;
            logs.drain(0..overflow);
        }
    }
}

fn phase_from_index(index: usize) -> MissionPhase {
    match index {
        0 => MissionPhase::Launch,
        1 => MissionPhase::OrbitCheckout,
        2 => MissionPhase::Tli,
        3 => MissionPhase::CoastOut,
        4 => MissionPhase::LunarFlyby,
        5 => MissionPhase::CoastHome,
        6 => MissionPhase::ReturnBurn,
        _ => MissionPhase::Reentry,
    }
}

#[derive(Properties, PartialEq)]
struct SparklineProps {
    values: Vec<ChartPoint>,
    stroke_class: &'static str,
    label: &'static str,
}

#[function_component(Sparkline)]
fn sparkline(props: &SparklineProps) -> Html {
    let width = 320.0;
    let height = 110.0;
    let padding = 8.0;

    if props.values.is_empty() {
        return html! {
            <div class="chart-wrap">
                <div class="subline">{props.label}</div>
                <div class="chart-frame"></div>
            </div>
        };
    }

    let min = props.values.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
    let max = props.values.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < f64::EPSILON { 1.0 } else { max - min };

    let step_x = if props.values.len() <= 1 {
        width - 2.0 * padding
    } else {
        (width - 2.0 * padding) / (props.values.len() as f64 - 1.0)
    };

    let mut d = String::new();
    for (i, point) in props.values.iter().enumerate() {
        let x = padding + i as f64 * step_x;
        let normalized = (point.value - min) / range;
        let y = height - padding - normalized * (height - 2.0 * padding);
        if i == 0 {
            d.push_str(&format!("M {:.2} {:.2}", x, y));
        } else {
            d.push_str(&format!(" L {:.2} {:.2}", x, y));
        }
    }

    html! {
        <div class="chart-wrap">
            <div class="subline">{props.label}</div>
            <div class="chart-frame">
                <svg viewBox="0 0 320 110" aria-label={props.label}>
                    <line x1="8" y1="102" x2="312" y2="102" stroke="rgba(255,255,255,0.12)" stroke-width="1" />
                    <path
                        d={d}
                        fill="none"
                        stroke={props.stroke_class}
                        stroke-width="3"
                        stroke-linejoin="round"
                        stroke-linecap="round"
                    />
                </svg>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct PhaseJumpProps {
    current_phase_idx: usize,
    on_jump: Callback<usize>,
}

#[function_component(PhaseJumpButtons)]
fn phase_jump_buttons(props: &PhaseJumpProps) -> Html {
    html! {
        <div class="phase-jump-wrap">
            {
                MissionPhase::all().iter().enumerate().map(|(i, phase)| {
                    let on_jump = props.on_jump.clone();
                    let is_active = i == props.current_phase_idx;
                    html! {
                        <button
                            class={classes!("btn", "phase-jump-btn", if is_active { "active" } else { "" })}
                            onclick={Callback::from(move |_| on_jump.emit(i))}
                        >
                            {phase.short_label()}
                        </button>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    let time_s = use_state(|| 0.0_f64);
    let playing = use_state(|| true);
    let speed = use_state(|| 1.0_f64);
    let logs = use_state(base_log_entries);
    let history = use_state(base_history);
    let viewport_width = use_state(window_width);
    let mobile_panel = use_state(|| MobilePanel::Center);

    {
        let time_s = time_s.clone();
        let playing = playing.clone();
        let speed = speed.clone();

        use_effect_with((), move |_| {
            let interval = Interval::new(TICK_MS, move || {
                if *playing {
                    let dt = (TICK_MS as f64 / 1000.0) * *speed;
                    let total = mission_duration_total();
                    let mut next = *time_s + dt;
                    if next > total {
                        next = total;
                    }
                    time_s.set(next);
                }
            });
            move || drop(interval)
        });
    }

    {
        let viewport_width = viewport_width.clone();
        use_effect_with((), move |_| {
            let listener = window().map(|w| {
                EventListener::new(&w, "resize", move |_| {
                    viewport_width.set(window_width());
                })
            });
            move || drop(listener)
        });
    }

    {
        let logs = logs.clone();
        let history = history.clone();
        let mission_t = *time_s;

        use_effect_with(mission_t, move |t| {
            let state = build_mission_state(*t);

            let mut next_logs = (*logs).clone();
            push_log(&mut next_logs, &state);
            if next_logs != *logs {
                logs.set(next_logs);
            }

            let mut next_history = (*history).clone();
            push_history(&mut next_history, &state);
            history.set(next_history);

            || ()
        });
    }

    let state = build_mission_state(*time_s);
    let apollo = to_apollo_display(&state);
    let orion = to_orion_display(&state);
    let phase_idx = MissionPhase::all()
        .iter()
        .position(|p| *p == state.phase)
        .unwrap_or(0);

    let is_mobile = *viewport_width < MOBILE_BREAKPOINT;

    let on_toggle_play = {
        let playing = playing.clone();
        Callback::from(move |_| playing.set(!*playing))
    };

    let on_reset = {
        let time_s = time_s.clone();
        let logs = logs.clone();
        let history = history.clone();
        Callback::from(move |_| {
            time_s.set(0.0);
            logs.set(base_log_entries());
            history.set(base_history());
        })
    };

    let on_speed_1 = {
        let speed = speed.clone();
        Callback::from(move |_| speed.set(1.0))
    };
    let on_speed_5 = {
        let speed = speed.clone();
        Callback::from(move |_| speed.set(5.0))
    };
    let on_speed_15 = {
        let speed = speed.clone();
        Callback::from(move |_| speed.set(15.0))
    };

    let on_jump = {
        let time_s = time_s.clone();
        let logs = logs.clone();
        let history = history.clone();
        let playing = playing.clone();

        Callback::from(move |idx: usize| {
            let phase = phase_from_index(idx);
            let t = phase_start_time(phase);

            time_s.set(t);
            playing.set(false);

            let mut next = base_log_entries();
            next.push(LogEntry {
                met: format_met(t),
                text: format!("Jumped to phase: {}", phase.label()),
            });
            logs.set(next);
            history.set(base_history());
        })
    };

    let show_apollo = {
        let mp = mobile_panel.clone();
        Callback::from(move |_| mp.set(MobilePanel::Apollo))
    };
    let show_center = {
        let mp = mobile_panel.clone();
        Callback::from(move |_| mp.set(MobilePanel::Center))
    };
    let show_orion = {
        let mp = mobile_panel.clone();
        Callback::from(move |_| mp.set(MobilePanel::Orion))
    };

    let total_progress_pct = (*time_s / mission_duration_total()) * 100.0;
    let path_progress = total_progress_pct / 100.0;

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="hero-top">
                    <div class="title-block">
                        <h1>{"Apollo vs Orion — Side-by-Side Deep Space Flight Console"}</h1>
                        <p>
                            {"One shared mission timeline. Two instrumentation philosophies. Apollo-style telemetry on the left, Orion-style telemetry on the right."}
                        </p>
                        <div class="badge-row">
                            <span class="badge">{"Rust + Yew + WASM"}</span>
                            <span class="badge">{"Same-stage dual instrumentation"}</span>
                            <span class="badge">{"Mini charts + stronger trajectory"}</span>
                            {
                                if is_mobile {
                                    html! { <span class="badge">{"Mobile tabbed layout"}</span> }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                    </div>

                    <div class="controls">
                        <div class="control-group">
                            <button
                                class={classes!("btn", if *playing { "active" } else { "" })}
                                onclick={on_toggle_play.clone()}
                            >
                                { if *playing { "Pause" } else { "Play" } }
                            </button>
                            <button class="btn" onclick={on_reset}>{"Reset"}</button>
                        </div>

                        <div class="control-group">
                            <label>{"Speed"}</label>
                            <button class={classes!("btn", if (*speed - 1.0).abs() < f64::EPSILON { "active" } else { "" })} onclick={on_speed_1}>{"1×"}</button>
                            <button class={classes!("btn", if (*speed - 5.0).abs() < f64::EPSILON { "active" } else { "" })} onclick={on_speed_5}>{"5×"}</button>
                            <button class={classes!("btn", if (*speed - 15.0).abs() < f64::EPSILON { "active" } else { "" })} onclick={on_speed_15}>{"15×"}</button>
                        </div>

                        <div class="control-group" style="flex-direction:column; align-items:flex-start;">
                            <label style="margin-bottom:6px;">{"Jump to phase"}</label>
                            <PhaseJumpButtons current_phase_idx={phase_idx} on_jump={on_jump} />
                        </div>
                    </div>
                </div>
            </section>

            <section class={classes!("console-grid", if is_mobile { "mobile-console-grid" } else { "" })}>
                {
                    if is_mobile {
                        html! {
                            <>
                                <div class="mobile-switcher">
                                    <button
                                        class={classes!("btn", if *mobile_panel == MobilePanel::Apollo { "active" } else { "" })}
                                        onclick={show_apollo}
                                    >
                                        {"Apollo"}
                                    </button>
                                    <button
                                        class={classes!("btn", if *mobile_panel == MobilePanel::Center { "active" } else { "" })}
                                        onclick={show_center}
                                    >
                                        {"Center"}
                                    </button>
                                    <button
                                        class={classes!("btn", if *mobile_panel == MobilePanel::Orion { "active" } else { "" })}
                                        onclick={show_orion}
                                    >
                                        {"Orion"}
                                    </button>
                                </div>

                                {
                                    match *mobile_panel {
                                        MobilePanel::Apollo => html! {
                                            <ApolloPanel
                                                display={apollo.clone()}
                                                state={state.clone()}
                                                history={(*history).clone()}
                                            />
                                        },
                                        MobilePanel::Center => html! {
                                            <CenterColumn
                                                state={state.clone()}
                                                total_progress_pct={total_progress_pct}
                                                path_progress={path_progress}
                                                logs={(*logs).clone()}
                                            />
                                        },
                                        MobilePanel::Orion => html! {
                                            <OrionPanel
                                                display={orion.clone()}
                                                state={state.clone()}
                                                history={(*history).clone()}
                                            />
                                        },
                                    }
                                }
                            </>
                        }
                    } else {
                        html! {
                            <>
                                <ApolloPanel
                                    display={apollo.clone()}
                                    state={state.clone()}
                                    history={(*history).clone()}
                                />
                                <CenterColumn
                                    state={state.clone()}
                                    total_progress_pct={total_progress_pct}
                                    path_progress={path_progress}
                                    logs={(*logs).clone()}
                                />
                                <OrionPanel
                                    display={orion.clone()}
                                    state={state.clone()}
                                    history={(*history).clone()}
                                />
                            </>
                        }
                    }
                }
            </section>

            <div class="footer-line">
                {"V2: stronger trajectory ribbon, mini charts, phase banners, Orion broadcast overlay, and responsive mobile panel switching."}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ApolloPanelProps {
    display: ApolloDisplay,
    state: MissionState,
    history: HistoryState,
}

#[function_component(ApolloPanel)]
fn apollo_panel(props: &ApolloPanelProps) -> Html {
    let d = &props.display;
    let s = &props.state;
    let history = &props.history;

    html! {
        <div class="panel-shell apollo-shell">
            <div class="panel-header">
                <h2 class="panel-title">{"Apollo Console"}</h2>
                <p class="panel-subtitle">
                    {"Chunkier labels, coarser readouts, discrete status language, and Apollo-era style instrumentation."}
                </p>
            </div>

            <div class="panel-content">
                <div class="phase-banner apollo-banner">{d.banner}</div>

                <div class="card apollo-card">
                    <h3>{"Mission Status"}</h3>
                    <div class="big-value apollo-value">{d.phase}</div>
                    <div class="subline">{format!("MET {}", d.met)}</div>

                    <div class="signal-lamps">
                        <span class="lamp">
                            <span class={classes!("lamp-dot", if d.lamp_guid { "on" } else { "" })}></span>
                            {"GUID"}
                        </span>
                        <span class="lamp">
                            <span class={classes!("lamp-dot", if d.lamp_comm { "on" } else { "" })}></span>
                            {"COMM"}
                        </span>
                        <span class="lamp">
                            <span class={classes!("lamp-dot", if d.lamp_prop { "on" } else { "" })}></span>
                            {"PROP"}
                        </span>
                    </div>
                </div>

                <div class="card-grid">
                    <div class="card apollo-card">
                        <h3>{"VEL"}</h3>
                        <div class="big-value apollo-value">{d.vel_kps.clone()}</div>
                        <div class="subline">{"Inertial velocity"}</div>
                    </div>
                    <div class="card apollo-card">
                        <h3>{"ALT"}</h3>
                        <div class="big-value apollo-value">{d.alt_km.clone()}</div>
                        <div class="subline">{"Instant altitude"}</div>
                    </div>
                    <div class="card apollo-card">
                        <h3>{"Downrange"}</h3>
                        <div class="big-value apollo-value">{d.downrange_km.clone()}</div>
                        <div class="subline">{"Ground track distance"}</div>
                    </div>
                    <div class="card apollo-card">
                        <h3>{"GNC"}</h3>
                        <div class="big-value apollo-value">{d.guidance}</div>
                        <div class="subline">{format!("COMM {}", d.comm)}</div>
                    </div>
                </div>

                <div class="card apollo-card">
                    <h3>{"Guidance / Attitude"}</h3>
                    <div class="mini-grid">
                        <div class="mini-stat apollo-mini">
                            <div class="mini-label">{"PITCH"}</div>
                            <div class="mini-value">{d.pitch.clone()}</div>
                        </div>
                        <div class="mini-stat apollo-mini">
                            <div class="mini-label">{"YAW"}</div>
                            <div class="mini-value">{d.yaw.clone()}</div>
                        </div>
                        <div class="mini-stat apollo-mini">
                            <div class="mini-label">{"ROLL"}</div>
                            <div class="mini-value">{d.roll.clone()}</div>
                        </div>
                    </div>
                    <div class="subline" style="margin-top:10px;">{d.log_line}</div>
                </div>

                <div class="card apollo-card">
                    <h3>{"Consumables"}</h3>
                    <div class="progress-wrap">
                        <div class="progress-row">
                            <span>{"Fuel"}</span>
                            <span>{d.fuel_pct.clone()}</span>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill apollo-fill" style={format!("width:{:.1}%;", s.fuel_pct)}></div>
                        </div>
                    </div>
                    <div class="progress-wrap">
                        <div class="progress-row">
                            <span>{"Power"}</span>
                            <span>{d.power_pct.clone()}</span>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill apollo-fill" style={format!("width:{:.1}%;", s.power_pct)}></div>
                        </div>
                    </div>
                    <div class="progress-wrap">
                        <div class="progress-row">
                            <span>{"Cabin Temp"}</span>
                            <span>{d.temp_c.clone()}</span>
                        </div>
                    </div>
                </div>

                <div class="card apollo-card">
                    <h3>{"Flight Trend"}</h3>
                    <Sparkline values={history.altitude.clone()} stroke_class="#d9d66c" label="ALT trend" />
                    <Sparkline values={history.velocity.clone()} stroke_class="#d9d66c" label="VEL trend" />
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct OrionPanelProps {
    display: OrionDisplay,
    state: MissionState,
    history: HistoryState,
}

#[function_component(OrionPanel)]
fn orion_panel(props: &OrionPanelProps) -> Html {
    let d = &props.display;
    let s = &props.state;
    let history = &props.history;

    html! {
        <div class="panel-shell orion-shell">
            <div class="panel-header">
                <h2 class="panel-title">{"Orion Console"}</h2>
                <p class="panel-subtitle">
                    {"Cleaner typography, richer diagnostics, denser telemetry, and software-defined modern mission status."}
                </p>
            </div>

            <div class="panel-content">
                <div class="phase-banner orion-banner">{d.banner}</div>

                <div class="card orion-card">
                    <h3>{"Mission Mode"}</h3>
                    <div class="big-value orion-value">{d.phase}</div>
                    <div class="subline">{format!("MET {}", d.met)}</div>
                    <div class="orion-status-row">
                        <span class="status-chip">{d.mode_chip}</span>
                        <span class="status-chip">{d.nav_chip}</span>
                        <span class="status-chip">{d.power_chip}</span>
                    </div>
                </div>

                <div class="card orion-card">
                    <h3>{"Broadcast Overlay"}</h3>
                    <div class="overlay-readout">
                        <div class="overlay-caption">{d.overlay_caption}</div>
                        <div class="overlay-metrics">
                            {format!(
                                "On-screen overlays read: Distance to Earth: {}. Distance to the Moon: {}. Velocity: {}. Mission Elapsed Time: {}",
                                d.overlay_distance_earth,
                                d.overlay_distance_moon,
                                d.overlay_velocity,
                                d.overlay_met
                            )}
                        </div>
                    </div>
                </div>

                <div class="card-grid">
                    <div class="card orion-card">
                        <h3>{"Inertial Velocity"}</h3>
                        <div class="big-value orion-value">{d.velocity.clone()}</div>
                        <div class="subline">{"Guidance-referenced solution"}</div>
                    </div>
                    <div class="card orion-card">
                        <h3>{"Altitude"}</h3>
                        <div class="big-value orion-value">{d.altitude.clone()}</div>
                        <div class="subline">{"Current flight altitude"}</div>
                    </div>
                    <div class="card orion-card">
                        <h3>{"Earth Distance"}</h3>
                        <div class="big-value orion-value">{d.earth_distance.clone()}</div>
                        <div class="subline">{"Range from Earth reference"}</div>
                    </div>
                    <div class="card orion-card">
                        <h3>{"Moon Distance"}</h3>
                        <div class="big-value orion-value">{d.moon_distance.clone()}</div>
                        <div class="subline">{"Range to lunar reference"}</div>
                    </div>
                </div>

                <div class="card orion-card">
                    <h3>{"GN&C / Communications"}</h3>
                    <div class="big-value orion-value" style="font-size:1.35rem;">{d.guidance}</div>
                    <div class="subline" style="margin-top:8px;">{d.comm}</div>
                    <div class="subline" style="margin-top:10px;">{d.log_line}</div>
                </div>

                <div class="card orion-card">
                    <h3>{"Vehicle Health"}</h3>
                    <div class="mini-grid">
                        <div class="mini-stat orion-mini">
                            <div class="mini-label">{"Propellant"}</div>
                            <div class="mini-value">{d.propellant.clone()}</div>
                        </div>
                        <div class="mini-stat orion-mini">
                            <div class="mini-label">{"Battery"}</div>
                            <div class="mini-value">{d.battery.clone()}</div>
                        </div>
                        <div class="mini-stat orion-mini">
                            <div class="mini-label">{"Cabin"}</div>
                            <div class="mini-value">{d.temp.clone()}</div>
                        </div>
                    </div>
                    <div class="progress-wrap">
                        <div class="progress-row">
                            <span>{"Propellant Reserve"}</span>
                            <span>{d.propellant.clone()}</span>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill orion-fill" style={format!("width:{:.1}%;", s.fuel_pct)}></div>
                        </div>
                    </div>
                    <div class="progress-wrap">
                        <div class="progress-row">
                            <span>{"Power Bus"}</span>
                            <span>{d.battery.clone()}</span>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill orion-fill" style={format!("width:{:.1}%;", s.power_pct)}></div>
                        </div>
                    </div>
                    <div class="subline" style="margin-top:10px;">{d.attitude.clone()}</div>
                    <div class="subline" style="margin-top:6px;">{format!("Downrange {}", d.downrange)}</div>
                </div>

                <div class="card orion-card">
                    <h3>{"Modern Telemetry Trends"}</h3>
                    <Sparkline values={history.altitude.clone()} stroke_class="#73d7ff" label="Altitude trend" />
                    <Sparkline values={history.velocity.clone()} stroke_class="#73d7ff" label="Velocity trend" />
                    <Sparkline values={history.comm.clone()} stroke_class="#73d7ff" label="Comm link trend" />
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct CenterColumnProps {
    state: MissionState,
    total_progress_pct: f64,
    path_progress: f64,
    logs: Vec<LogEntry>,
}

#[function_component(CenterColumn)]
fn center_column(props: &CenterColumnProps) -> Html {
    let state = &props.state;

    let outbound = props.path_progress.min(0.5) / 0.5;
    let inbound = if props.path_progress > 0.5 {
        (props.path_progress - 0.5) / 0.5
    } else {
        0.0
    };

    let current_x = if props.path_progress <= 0.5 {
        36.0 + outbound * 148.0
    } else {
        184.0 - inbound * 148.0
    };

    let current_y = if props.path_progress <= 0.5 {
        210.0 - outbound * 150.0
    } else {
        60.0 + inbound * 158.0
    };

    html! {
        <div class="center-column">
            <div class="phase-card">
                <div class="center-label">{"Current Phase"}</div>
                <div class="phase-name">{state.phase.label()}</div>
                <div class="met">{format_met(state.mission_time_s)}</div>
                <div class="subline" style="margin-top:8px;">
                    {format!("Mission progress {:.0}%", props.total_progress_pct)}
                </div>
                <div class="progress-wrap">
                    <div class="progress-bar">
                        <div class="progress-fill orion-fill" style={format!("width:{:.1}%;", props.total_progress_pct)}></div>
                    </div>
                </div>
            </div>

            <div class="center-card">
                <div class="center-label">{"Mission Stages"}</div>
                <div class="phase-list" style="margin-top:10px;">
                    {
                        MissionPhase::all().iter().map(|phase| {
                            html! {
                                <div class={classes!("phase-pill", if *phase == state.phase { "active" } else { "" })}>
                                    {phase.label()}
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>

            <div class="center-card">
                <div class="center-label">{"Trajectory View"}</div>
                <div class="path-box" style="margin-top:10px;">
                    <svg viewBox="0 0 220 260" aria-label="trajectory diagram">
                        <defs>
                            <radialGradient id="earthGlowV2" cx="50%" cy="50%" r="50%">
                                <stop offset="0%" stop-color="rgba(115,215,255,0.95)" />
                                <stop offset="100%" stop-color="rgba(115,215,255,0.15)" />
                            </radialGradient>
                            <radialGradient id="moonGlowV2" cx="50%" cy="50%" r="50%">
                                <stop offset="0%" stop-color="rgba(240,240,255,0.95)" />
                                <stop offset="100%" stop-color="rgba(240,240,255,0.18)" />
                            </radialGradient>
                        </defs>

                        <circle cx="36" cy="210" r="18" fill="url(#earthGlowV2)" />
                        <circle cx="184" cy="60" r="12" fill="url(#moonGlowV2)" />

                        <path
                            d="M 36 210 Q 85 55 184 60"
                            fill="none"
                            stroke="rgba(125,242,255,0.55)"
                            stroke-width="3"
                            stroke-dasharray="5 5"
                        />
                        <path
                            d="M 184 60 Q 148 170 36 218"
                            fill="none"
                            stroke="rgba(255,255,255,0.26)"
                            stroke-width="2.5"
                            stroke-dasharray="4 4"
                        />

                        <circle
                            cx={format!("{:.2}", current_x)}
                            cy={format!("{:.2}", current_y)}
                            r="5.8"
                            fill="rgba(125,242,255,0.95)"
                        />

                        <text x="18" y="242" fill="rgba(255,255,255,0.65)" font-size="11">{"Earth"}</text>
                        <text x="172" y="38" fill="rgba(255,255,255,0.65)" font-size="11">{"Moon"}</text>
                    </svg>
                </div>
                <div class="traj-legend">
                    <div>{format!("Distance from Earth: {:.0} km", state.distance_from_earth_km)}</div>
                    <div>{format!("Distance to Moon: {:.0} km", state.distance_to_moon_km)}</div>
                </div>
            </div>

            <div class="center-card">
                <div class="center-label">{"Comparison Notes"}</div>
                <div class="compare-grid" style="margin-top:10px;">
                    <div class="compare-row">
                        <div class="compare-title">{"Same phase"}</div>
                        <div class="compare-values">
                            {"Both sides are locked to the same mission stage and physics-driven telemetry state."}
                        </div>
                    </div>
                    <div class="compare-row">
                        <div class="compare-title">{"Apollo style"}</div>
                        <div class="compare-values">
                            {"Abbreviated, procedural, stepped, and console-first."}
                        </div>
                    </div>
                    <div class="compare-row">
                        <div class="compare-title">{"Orion style"}</div>
                        <div class="compare-values">
                            {"Software-rich, audience-readable, mission-broadcast aware."}
                        </div>
                    </div>
                </div>
            </div>

            <div class="center-card">
                <div class="center-label">{"Event Log"}</div>
                <div class="log-wrap">
                    {
                        props.logs.iter().rev().map(|entry| {
                            html! {
                                <div class="log-item">
                                    <div class="log-time">{entry.met.clone()}</div>
                                    <div class="log-text">{entry.text.clone()}</div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>

            <div class="bottom-note">
                {"This V2 build keeps one shared simulated mission state and transforms it into two distinct display languages. Apollo remains terse and procedural. Orion remains modern, graphic-rich, and broadcast-friendly."}
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}