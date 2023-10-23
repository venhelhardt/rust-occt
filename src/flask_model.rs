use crate::gfx;

use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;

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
    enq: Arc<mpsc::Sender<GeneratedModel>>,
    deq: mpsc::Receiver<GeneratedModel>,
    max_queue_size: u32,
    beat_interval: u32,
    // Requests queue, (timestamp, model)
    // Model is None for pending requests or consumed by dequeue
    queue: VecDeque<(u32, Option<GeneratedModel>)>,
    // Number of enqueued mesh generations
    enqueue_size: u32,
}

impl Drop for ModelGenerator {
    fn drop(&mut self) {
        if !self.queue.is_empty() {
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
        let (enq, deq): (mpsc::Sender<GeneratedModel>, mpsc::Receiver<GeneratedModel>) =
            std::sync::mpsc::channel();

        Self {
            pool: pool,
            enq: Arc::new(enq),
            deq: deq,
            max_queue_size: max_queue_size,
            beat_interval: beat_interval,
            queue: VecDeque::new(),
            enqueue_size: 0,
        }
    }

    fn max_queue_size(&self) -> u32 {
        self.max_queue_size
    }

    fn queue_size(&self) -> u32 {
        self.enqueue_size as u32
    }

    fn dequeue(&mut self, ts: u32) -> Option<GeneratedModel> {
        // Drain queue
        loop {
            let deq: Result<GeneratedModel, mpsc::TryRecvError> = self.deq.try_recv();

            if deq.is_err() {
                break;
            }

            self.enqueue_size -= 1;

            let recv_model = deq.unwrap();

            // Update requested mesh, otherwise discard it (since request was dropped)
            if let Some(first_model) = self.queue.front() {
                if first_model.0 <= recv_model.ts {
                    let idx = (recv_model.ts - first_model.0) as usize;

                    if idx < self.queue.len() && self.queue[idx].0 == recv_model.ts {
                        self.queue[idx].1 = Some(recv_model);
                    }
                }
            }
        }

        // Drop all staled models except one
        while self.queue.len() > 1 && self.queue[1].0 <= ts {
            self.queue.pop_front();
        }

        // Drop everything outside [ts, ts + self.max_queue_size]
        while !self.queue.is_empty() && self.queue[0].0 + self.max_queue_size < ts {
            self.queue.pop_front();
        }

        while !self.queue.is_empty() && self.queue.back().unwrap().0 > ts + self.max_queue_size {
            self.queue.pop_back();
        }

        // Schedule generations
        let beat_interval = self.beat_interval;

        while self.max_queue_size > self.queue.len() as u32 {
            let gen_ts = if self.queue.is_empty() {
                ts
            } else {
                self.queue.back().unwrap().0 + 1
            };

            self.queue.push_back((gen_ts, None));
            self.enqueue_size += 1;

            let enq = self.enq.clone();

            self.pool.execute(move || {
                let interval = gen_ts / beat_interval;
                let mut width: (f64, f64) = (0.5, 0.55);
                let mut thickness: (f64, f64) = (0.25, 0.35);
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

                enq.send(GeneratedModel {
                    ts: gen_ts,
                    mesh: mesh,
                })
                .unwrap();
            });
        }

        // Consume 'best' model
        return self.queue[0].1.take();
    }
}
