use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[cfg(not(target_arch = "wasm32"))]
const PIXEL_SCALE: u32 = 16;
#[cfg(target_arch = "wasm32")]
const PIXEL_SCALE: u32 = 12;

pub struct Gfx {
    pub window: Window,
    pixels: Pixels,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

impl Gfx {
    pub async fn new(width: u32, height: u32) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        // physical window size = virtual size × scale
        let physical_size = PhysicalSize::new(width * PIXEL_SCALE, height * PIXEL_SCALE);

        let window = WindowBuilder::new()
            .with_title("chip-8")
            .with_inner_size(physical_size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        // On the web, winit creates a detached <canvas>; size it and attach it
        // to the host page (into #ch8-canvas-container if present, else body).
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas();
            canvas.set_width(physical_size.width);
            canvas.set_height(physical_size.height);
            let doc = web_sys::window().unwrap().document().unwrap();
            let el = web_sys::Element::from(canvas);
            match doc.get_element_by_id("ch8-canvas-container") {
                Some(container) => container.append_child(&el).unwrap(),
                None => doc.body().unwrap().append_child(&el).unwrap(),
            };
        }

        // SurfaceTexture uses the physical (window) pixels,
        // but the 'logical' pixel buffer stays at width×height
        let surface_texture =
            SurfaceTexture::new(physical_size.width, physical_size.height, &window);

        // build_async works on both native and web; on the web `Pixels::new`
        // cannot block on the wgpu adapter request.
        let pixels = PixelsBuilder::new(width, height, surface_texture)
            .build_async()
            .await
            .unwrap();

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

    #[allow(dead_code)]
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
