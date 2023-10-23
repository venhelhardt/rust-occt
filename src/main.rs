mod flask_model;
mod gfx;
mod gui;

use flask_model::FlaskModel;

fn main() {
    const FPS: (u32, u32) = (60, 1);
    const MODEL_BUFFER: u32 = 1 * FPS.0 / FPS.1;
    const BEAT_INTERVAL: u32 = (1 * FPS.0 / FPS.1) / 2;

    let mut wnd = gui::Wnd::new();

    wnd.initialize(1280, 720, FPS.0, FPS.1, "rust-occt")
        .unwrap();
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(0.0, 1.0, 2.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        glam::Vec3::new(0.0, 1.0, 0.0),
    );
    let proj = glam::Mat4::perspective_rh(0.785, 16.0 / 9.0, 0.1, 10.0);
    let light_pos = glam::Vec3::new(5.0, 5.0, 10.0);
    let max_threads_n = std::thread::available_parallelism().unwrap().get();
    let pool = std::rc::Rc::new(threadpool::ThreadPool::with_name(
        "ModelGenerator".to_string(),
        max_threads_n,
    ));
    let mut model = FlaskModel::new(pool.clone(), MODEL_BUFFER, BEAT_INTERVAL).unwrap();

    wnd.run(|frame_num: u32, gfx: &mut gui::WndGfx| -> () {
        // Draw model
        let rot_per_second = 0.25;
        let delta_deg = -1.0 * rot_per_second * 360.0 * FPS.1 as f32 / FPS.0 as f32;
        let rot_q = glam::Quat::from_rotation_y(frame_num as f32 * f32::to_radians(delta_deg));
        let model_ts = model.draw(
            &glam::Mat4::from_quat(rot_q),
            &view,
            &proj,
            &light_pos,
            frame_num,
        );

        // Draw stats
        gfx.draw_text_at_bottom_left(format!("Frame {:9}", frame_num).as_str(), 4);

        let total_milliseconds = frame_num * FPS.1 * 1000 / FPS.0;
        let minutes = total_milliseconds / 60000;
        let seconds = (total_milliseconds / 1000) % 60;
        let milliseconds = total_milliseconds % 1000;

        gfx.draw_text_at_bottom_left(
            format!("Time  {:02}:{:02}.{:03}", minutes, seconds, milliseconds).as_str(),
            3,
        );
        gfx.draw_text_at_bottom_left("Queue", 2);
        gfx.draw_text_at_bottom_left(
            format!(
                "  {}",
                if model_ts == frame_num {
                    ": in sync".to_string()
                } else if model_ts < frame_num {
                    format!(": behind {0}", frame_num - model_ts)
                } else {
                    format!(": ahead {0}", model_ts - frame_num)
                }
            )
            .as_str(),
            1,
        );
        gfx.draw_text_at_bottom_left(
            format!(
                "  : enqueued {}/{}",
                model.queue_size(),
                model.max_queue_size()
            )
            .as_str(),
            0,
        );
    });
}

trait TextDrawer {
    fn draw_text_at_bottom_left(&mut self, text: &str, line: u32);
}

impl TextDrawer for gui::WndGfx<'_> {
    fn draw_text_at_bottom_left(&mut self, text: &str, line: u32) {
        const FONT_SIZE: u32 = 32;
        const SPACING: f32 = 16.0;
        const TEXT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);

        self.draw_text(
            text,
            FONT_SIZE,
            SPACING,
            self.get_window_size().1 as f32
                - (FONT_SIZE * line) as f32
                - SPACING * (line + 1) as f32,
            TEXT_COLOR,
        );
    }
}
