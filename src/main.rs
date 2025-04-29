mod chip8;
mod gfx;
mod instruction;

use chip8::Chip8;
use gfx::Gfx;
use std::{
    fs,
    time::{Duration, Instant},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

const CPU_HZ: f64 = 12.0;
const path: &str = "roms/ibm.ch8";

fn main() {
    let mut chip8 = Chip8::new();
    let (mut gfx, event_loop) = Gfx::new(64, 32);

    let data = fs::read(path).expect("Failed to read ROM file");
    chip8.load(&data);

    let cpu_period = Duration::from_secs_f64(1.0 / CPU_HZ);
    let timer_period = Duration::from_secs_f64(1.0 / 60.0);

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

                cpu_accumulator += dt;
                timer_accumulator += dt;

                while cpu_accumulator >= cpu_period {
                    chip8.execute_cycle();
                    cpu_accumulator -= cpu_period;
                    gfx.chip8_display(chip8.display_buffer);
                    gfx.render();
                    // println!("Redrew");
                }

                // while timer_accumulator >= timer_period {
                //     // chip8.update_timers();
                //     timer_accumulator -= timer_period;
                // }
            }
            Event::RedrawRequested(_) => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
