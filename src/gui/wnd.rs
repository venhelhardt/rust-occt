use std::time::Instant;

use piston_window::{
    text, Glyphs, OpenGL, OpenGLWindow, PistonWindow, RenderArgs, RenderEvent, TextureSettings,
    Transformed, WindowSettings,
};

pub struct Wnd {
    wnd: Option<PistonWindow>,
    glyphs: Option<Glyphs>,
    fps: (u32, u32),
}

impl Drop for Wnd {
    fn drop(&mut self) {}
}

impl Wnd {
    pub fn new() -> Wnd {
        Wnd {
            wnd: None,
            glyphs: None,
            fps: (1, 1),
        }
    }

    pub fn initialize(
        &mut self,
        w: u32,
        h: u32,
        fps_num: u32,
        fps_den: u32,
        title: &str,
    ) -> Result<bool, String> {
        let mut wnd: PistonWindow = match WindowSettings::new(title, [w, h])
            .exit_on_esc(true)
            .graphics_api(OpenGL::V3_2)
            .build()
        {
            Ok(w) => w,
            Err(e) => return Err(e.to_string()),
        };

        gl::load_with(|symbol| wnd.window.get_proc_address(symbol) as *const _);

        let font_data: &[u8] = include_bytes!("assets/UbuntuMono-Regular.ttf");
        let font: rusttype::Font<'static> = match rusttype::Font::try_from_bytes(font_data) {
            Some(f) => f,
            None => return Err("Failed to load font".to_string()),
        };

        let glyphs = Glyphs::from_font(font, wnd.create_texture_context(), TextureSettings::new());

        wnd.window.make_current();

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        self.wnd = Some(wnd);
        self.glyphs = Some(glyphs);
        self.fps = (fps_num, fps_den);

        Ok(true)
    }

    pub fn run<F>(&mut self, mut cb: F)
    where
        F: FnMut(u32, &mut WndGfx),
    {
        let start_time = Instant::now();

        while let Some(event) = self.wnd.as_mut().unwrap().next() {
            if let Some(args) = event.render_args() {
                self.wnd.as_mut().unwrap().window.make_current();

                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    gl::Enable(gl::DEPTH_TEST);
                }

                let ticks_per_second = 1000000u128;
                let frame_tick = self.fps.1 as u128 * ticks_per_second / self.fps.0 as u128;
                let ticks_elapsed =
                    start_time.elapsed().as_micros() * ticks_per_second / 1000000u128;
                let frame_num = ticks_elapsed / frame_tick;
                let mut gfx = WndGfx {
                    wnd: self.wnd.as_mut().unwrap(),
                    glyphs: self.glyphs.as_mut().unwrap(),
                    render_args: args,
                };

                cb(frame_num as u32, &mut gfx);

                let wnd = self.wnd.as_mut().unwrap();
                let device = &mut wnd.device;

                self.glyphs.as_mut().unwrap().factory.encoder.flush(device);
                wnd.encoder.flush(device);
            }
        }
    }
}

pub struct WndGfx<'a> {
    wnd: &'a mut PistonWindow,
    glyphs: &'a mut Glyphs,
    render_args: RenderArgs,
}

impl WndGfx<'_> {
    pub fn get_window_size(&self) -> (u32, u32) {
        (
            self.render_args.window_size[0] as u32,
            self.render_args.window_size[1] as u32,
        )
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font_size: u32,
        pos_x: f32,
        pos_y: f32,
        color: (f32, f32, f32, f32),
    ) {
        let _ = self.wnd.g2d.draw(
            &mut self.wnd.encoder,
            &self.wnd.output_color,
            &self.wnd.output_stencil,
            self.render_args.viewport(),
            |ctx, gfx| {
                let _ = text::Text::new_color([color.0, color.1, color.2, color.3], font_size)
                    .draw(
                        text,
                        self.glyphs,
                        &ctx.draw_state,
                        ctx.transform.trans(pos_x as f64, pos_y as f64),
                        gfx,
                    );
            },
        );
    }
}
