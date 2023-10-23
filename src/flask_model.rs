use crate::gfx;

use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use gl::types::{GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid};

pub(super) struct FlaskModel {
    gen: ModelGenerator,
    fx: gfx::PhongFx,
    object_space: glam::Mat4,
    vao_ts: u32,
    vao: GLuint,
    vbo_pos: GLuint,
    vbo_norm: GLuint,
    vbo_tri: GLuint,
    vbo_tri_elements_n: GLuint,
}

impl Drop for FlaskModel {
    fn drop(&mut self) {
        if self.vao != 0 {
            let vao = self.vao;
            let vbos: [GLuint; 3] = [self.vbo_pos, self.vbo_norm, self.vbo_tri];

            self.vao_ts = 0;
            self.vao = 0;
            self.vbo_pos = 0;
            self.vbo_norm = 0;
            self.vbo_tri = 0;
            self.vbo_tri_elements_n = 0;

            unsafe {
                gl::DeleteVertexArrays(1, &vao as *const GLuint);
                gl::DeleteBuffers(vbos.len() as i32, vbos.as_ptr());
            }
        }
    }
}

impl FlaskModel {
    pub fn new(
        pool: std::rc::Rc<threadpool::ThreadPool>,
        max_queue_size: u32,
        beat_interval: u32,
    ) -> Result<Self, String> {
        let phong_fx = gfx::PhongFx::new()?;

        Ok(Self {
            gen: ModelGenerator::new(pool, max_queue_size, beat_interval),
            fx: phong_fx,
            object_space: glam::Mat4::IDENTITY,
            vao_ts: 0,
            vao: 0,
            vbo_pos: 0,
            vbo_norm: 0,
            vbo_tri: 0,
            vbo_tri_elements_n: 0,
        })
    }

    pub fn max_queue_size(&self) -> u32 {
        self.gen.max_queue_size()
    }

    pub fn queue_size(&self) -> u32 {
        self.gen.queue_size()
    }

    pub fn draw(
        &mut self,
        model: &glam::Mat4,
        view: &glam::Mat4,
        proj: &glam::Mat4,
        light_pos: &glam::Vec3,
        frame_num: u32,
    ) -> u32 {
        if let Some(model) = self.gen.dequeue(frame_num) {
            self.generate_vao(model);
        }

        if self.vao != 0 {
            self.fx.activate(
                &model.mul_mat4(&self.object_space),
                &view,
                &proj,
                &light_pos,
                || -> () {
                    unsafe {
                        gl::BindVertexArray(self.vao);
                        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vbo_tri);
                        gl::DrawElements(
                            gl::TRIANGLES,
                            self.vbo_tri_elements_n as GLint,
                            gl::UNSIGNED_INT,
                            std::ptr::null(),
                        );
                    }
                },
            );
        }

        self.vao_ts
    }

    fn generate_vao(&mut self, model: GeneratedModel) {
        if self.vao_ts == model.ts {
            return;
        }

        if self.vao == 0 {
            unsafe {
                let mut vbos: [GLuint; 3] = [0, 0, 0];

                gl::GenVertexArrays(1, &mut self.vao);
                gl::GenBuffers(vbos.len() as i32, vbos.as_mut_ptr());

                self.vbo_pos = vbos[0];
                self.vbo_norm = vbos[1];
                self.vbo_tri = vbos[2];
            }
        }

        self.vao_ts = model.ts;

        let mesh = model.mesh;
        let bbox = mesh.bbox();

        self.object_space = glam::Mat4::from_translation(
            glam::Vec3::new(
                -(bbox.max.x + bbox.min.x),
                -(bbox.max.y + bbox.min.y),
                -(bbox.max.z + bbox.min.z),
            ) * 0.5,
        );

        unsafe {
            gl::BindVertexArray(self.vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_pos);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.verts().count as usize * 3 * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                mesh.verts().ptr as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                self.fx.pos_loc_id(),
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * std::mem::size_of::<GLfloat>() as GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(self.fx.pos_loc_id());

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_norm);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.norms().count as usize * 3 * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                mesh.norms().ptr as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                self.fx.norm_loc_id(),
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * std::mem::size_of::<GLfloat>() as GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(self.fx.norm_loc_id());

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vbo_tri);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.tris().count as usize * 3 * std::mem::size_of::<GLuint>()) as GLsizeiptr,
                mesh.tris().ptr as *const GLvoid,
                gl::STATIC_DRAW,
            );
            self.vbo_tri_elements_n = mesh.tris().count * 3;
        }
    }
}

struct GeneratedModel {
    ts: u32,
    mesh: occt::MeshBlob,
}

struct ModelGenerator {
    pool: Rc<threadpool::ThreadPool>,
    max_queue_size: u32,
    beat_interval: u32,
    // Number of enqueued mesh generations
    enqueue_size: Arc<AtomicU32>,
    curr_model_sync: Arc<Mutex<(u32, Option<GeneratedModel>)>>,
}

impl Drop for ModelGenerator {
    fn drop(&mut self) {
        if self.enqueue_size.load(Ordering::Acquire) > 0 {
            self.pool.join();
        }
    }
}

impl ModelGenerator {
    fn lerp_f64(s: f64, e: f64, t: f64) -> f64 {
        s + t * (e - s)
    }

    fn new(
        pool: std::rc::Rc<threadpool::ThreadPool>,
        max_queue_size: u32,
        beat_interval: u32,
    ) -> Self {
        Self {
            pool: pool,
            max_queue_size: max_queue_size,
            beat_interval: beat_interval,
            enqueue_size: Arc::new(AtomicU32::new(0)),
            curr_model_sync: Arc::new(Mutex::new((0, None))),
        }
    }

    fn max_queue_size(&self) -> u32 {
        self.max_queue_size
    }

    fn queue_size(&self) -> u32 {
        self.enqueue_size.load(Ordering::Relaxed) as u32
    }

    fn dequeue(&mut self, ts: u32) -> Option<GeneratedModel> {
        if self.enqueue_size.load(Ordering::Acquire) < self.max_queue_size {
            self.enqueue_size.fetch_add(1, Ordering::Relaxed);

            let beat_interval = self.beat_interval;
            let gen_ts = ts;
            let curr_model_sync = self.curr_model_sync.clone();
            let enq_size = self.enqueue_size.clone();

            self.pool.execute(move || {
                let interval = gen_ts / beat_interval;
                let mut width: (f64, f64) = (0.5, 0.55);
                let mut thickness: (f64, f64) = (0.25, 0.55);
                let mut height: (f64, f64) = (0.75, 1.0);

                if interval & 1 == 0 {
                    std::mem::swap(&mut width.0, &mut width.1);
                    std::mem::swap(&mut thickness.0, &mut thickness.1);
                    std::mem::swap(&mut height.0, &mut height.1);
                }

                let delta = (gen_ts % beat_interval) as f64 / beat_interval as f64;
                let mesh = occt::make_flask(
                    Self::lerp_f64(width.0, width.1, delta),
                    Self::lerp_f64(thickness.0, thickness.1, delta),
                    Self::lerp_f64(height.0, height.1, delta),
                );

                let mut curr_model = curr_model_sync.lock().unwrap();

                if curr_model.0 <= ts {
                    curr_model.0 = ts;
                    curr_model.1 = Some(GeneratedModel { ts: ts, mesh: mesh });
                }

                enq_size.fetch_sub(1, Ordering::Release);
            });
        }

        return self.curr_model_sync.lock().unwrap().1.take();
    }
}
