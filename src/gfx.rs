use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const PIXEL_SCALE: u32 = 16;

pub struct Gfx {
    pub window: Window,
    pixels: Pixels,
    width: u32,
    height: u32,
}

impl Gfx {
    pub fn new(width: u32, height: u32) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        // physical window size = virtual size × scale
        let physical_size = PhysicalSize::new(width * PIXEL_SCALE, height * PIXEL_SCALE);

        let window = WindowBuilder::new()
            .with_title("chip-8")
            .with_inner_size(physical_size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        // SurfaceTexture uses the physical (window) pixels,
        // but the 'logical' pixel buffer stays at width×height
        let surface_texture =
            SurfaceTexture::new(physical_size.width, physical_size.height, &window);

        let pixels = Pixels::new(width, height, surface_texture).unwrap();

        (
            Gfx {
                window,
                pixels,
                width,
                height,
            },
            event_loop,
        )
    }

    pub fn render(&mut self) {
        self.pixels.render().unwrap();
    }

    pub fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    pub fn chip8_display(&mut self, display_buffer: [bool; 2048]) {
        let frame = self.pixels.frame_mut();
        for (i, &pixel) in display_buffer.iter().enumerate() {
            let base = i * 4;
            let color = if pixel { 255 } else { 0 };
            frame[base..base + 4].copy_from_slice(&[color, color, color, 255]);
        }
    }
}

fn _rst(frame: &mut [u8]) {
    let black = [0, 0, 0, 255].repeat(frame.len() / 4);
    frame.copy_from_slice(&black)
}
