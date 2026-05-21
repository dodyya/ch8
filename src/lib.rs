mod chip8;
mod gfx;
mod instruction;

use web_time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

// CHIP-8 runs its 60 Hz delay/sound timers and a faster CPU clock. Both are
// driven from inside the winit event loop so the wasm build needs no threads.
const TIMER_HZ: f64 = 60.0;
const CPU_HZ: f64 = 600.0;

// The OctoJam ROMs bundled into the binary, so the web build needs no
// filesystem. The `?rom=` query parameter selects among the keys below.
const ROMS: &[(&str, &[u8])] = &[
    ("octojam6title", include_bytes!("../roms/octojam6title.ch8")),
    ("octo1", include_bytes!("../roms/octo1.ch8")),
    ("octo9", include_bytes!("../roms/octo9.ch8")),
    ("octo10", include_bytes!("../roms/octo10.ch8")),
];

// Default ROM when no (or an unrecognized) `?rom=` value is supplied.
#[allow(dead_code)]
const DEFAULT_ROM: &str = "octojam6title";

#[allow(dead_code)]
fn rom_bytes(key: &str) -> &'static [u8] {
    ROMS.iter()
        .find(|(name, _)| *name == key)
        .or_else(|| ROMS.iter().find(|(name, _)| *name == DEFAULT_ROM))
        .map(|(_, data)| *data)
        .unwrap()
}

/// Map a physical keyboard key to a CHIP-8 hex keypad index.
/// Layout: 1 2 3 4 / Q W E R / A S D F / Z X C V.
fn keypad_index(key: VirtualKeyCode) -> Option<u8> {
    use VirtualKeyCode::*;
    Some(match key {
        Key1 => 0x1,
        Key2 => 0x2,
        Key3 => 0x3,
        Key4 => 0xC,
        Q => 0x4,
        W => 0x5,
        E => 0x6,
        R => 0xD,
        A => 0x7,
        S => 0x8,
        D => 0x9,
        F => 0xE,
        Z => 0xA,
        X => 0x0,
        C => 0xB,
        V => 0xF,
        _ => return None,
    })
}

pub async fn run(rom: &[u8]) {
    let mut emu = chip8::Chip8::new();
    emu.load(rom);

    let (mut gfx, event_loop) = gfx::Gfx::new(64, 32).await;

    let cpu_period = Duration::from_secs_f64(1.0 / CPU_HZ);
    let timer_period = Duration::from_secs_f64(1.0 / TIMER_HZ);
    let mut cpu_accumulator = Duration::ZERO;
    let mut timer_accumulator = Duration::ZERO;
    let mut last_instant = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                let now = Instant::now();
                let dt = now.duration_since(last_instant);
                last_instant = now;

                // Cap dt so a paused tab does not spin through a huge backlog.
                let dt = dt.min(Duration::from_millis(100));
                cpu_accumulator += dt;
                timer_accumulator += dt;

                while cpu_accumulator >= cpu_period {
                    emu.execute_cycle();
                    cpu_accumulator -= cpu_period;
                }
                while timer_accumulator >= timer_period {
                    emu.update_timers();
                    timer_accumulator -= timer_period;
                }

                gfx.chip8_display(emu.display_buffer);
                gfx.render();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode.and_then(keypad_index) {
                        emu.set_key(key, input.state == ElementState::Pressed);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Entry point for the browser build. Runs automatically once the wasm module
// is initialized; native builds use `main.rs` instead.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    // Pick the ROM from the `?rom=` query parameter (e.g. ?rom=octo1).
    let rom_key = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|search| {
            search
                .trim_start_matches('?')
                .split('&')
                .find_map(|pair| pair.strip_prefix("rom="))
                .map(|v| v.to_string())
        })
        .unwrap_or_else(|| DEFAULT_ROM.to_string());

    let rom = rom_bytes(&rom_key);
    wasm_bindgen_futures::spawn_local(async move {
        run(rom).await;
    });
}
