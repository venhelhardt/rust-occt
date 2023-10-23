use super::{Attrib, Program, Shader, Uniform};

pub struct PhongFx {
    prog: Program,
    mvp_loc: Uniform,
    model_loc: Uniform,
    model_norm_loc: Uniform,
    cam_pos_loc: Uniform,
    light_pos_loc: Uniform,
    pos_loc: Attrib,
    norm_loc: Attrib,
}

impl PhongFx {
    pub fn new() -> Result<PhongFx, String> {
        let vertex_shader = Shader::vertex_source(
            r#"
            #version 330 core
            uniform mat4 mvp;
            uniform mat4 model;
            uniform mat3 model_norm;
            in vec3 pos;
            in vec3 norm;
            out vec3 pos_frag;
            out vec3 norm_frag;
            void main() {
                norm_frag = model_norm * norm;
                pos_frag = vec3(model * vec4(pos, 1.0));
                gl_Position = mvp * vec4(pos, 1.0);
            }
        "#,
        )?;

        let frag_shader = Shader::fragment_source(
            r#"
            #version 330 core
            uniform vec3 camera_pos;
            uniform vec3 light_pos;
            in vec3 pos_frag;
            in vec3 norm_frag;
            out vec4 FragColor;
            void main() {
                vec3 light_color = vec3(1.0, 1.0, 1.0);
                vec3 obj_color = vec3(0.2, 0.5, 0.2);
                vec3 ambient_color = obj_color * 0.2;
                vec3 norm = normalize(norm_frag);
                vec3 light_dir = normalize(light_pos - pos_frag);
                vec3 cam_dir = normalize(camera_pos - pos_frag);
                vec3 reflect_dir = reflect(-light_dir, norm);  
                vec3 diffuse_color = max(dot(norm, light_dir), 0.0) * light_color;
                vec3 specular_color = 2.0 * pow(max(dot(cam_dir, reflect_dir), 0.0), 4.0) * light_color;
                vec3 frag_color = (ambient_color + diffuse_color + specular_color) * obj_color;

                FragColor = vec4(frag_color, 1.0);
            }
        "#,
        )?;

        let prog = Program::link(&[&vertex_shader, &frag_shader])?;
        let mvp_loc = prog.get_uniform("mvp").ok_or("Failed to get mvp uniform")?;
        let model_loc = prog
            .get_uniform("model")
            .ok_or("Failed to get model uniform")?;
        let model_norm_loc = prog
            .get_uniform("model_norm")
            .ok_or("Failed to get model_norm uniform")?;
        let cam_pos_loc = prog
            .get_uniform("camera_pos")
            .ok_or("Failed to get cam_pos uniform")?;
        let light_pos_loc = prog
            .get_uniform("light_pos")
            .ok_or("Failed to get light_pos uniform")?;
        let vert_loc = prog
            .get_attrib("pos")
            .ok_or("Failed to get pos attribute")?;
        let norm_loc = prog
            .get_attrib("norm")
            .ok_or("Failed to get norm attribute")?;

        Ok(PhongFx {
            prog: prog,
            mvp_loc: mvp_loc,
            model_loc: model_loc,
            model_norm_loc: model_norm_loc,
            cam_pos_loc: cam_pos_loc,
            light_pos_loc: light_pos_loc,
            pos_loc: vert_loc,
            norm_loc: norm_loc,
        })
    }

    pub fn pos_loc_id(&self) -> u32 {
        return self.pos_loc.id as u32;
    }

    pub fn norm_loc_id(&self) -> u32 {
        return self.norm_loc.id as u32;
    }

    pub fn activate<F>(
        &self,
        model: &glam::Mat4,
        view: &glam::Mat4,
        proj: &glam::Mat4,
        light_pos: &glam::Vec3,
        cb: F,
    ) where
        F: FnOnce(),
    {
        self.prog.activate(|| -> () {
            let mvp = proj.mul_mat4(&view.mul_mat4(&model));
            let view_inv = view.inverse();

            self.mvp_loc.set_mat4(&mvp);
            self.model_loc.set_mat4(&model);
            self.model_norm_loc.set_mat3(&glam::Mat3::from_quat(
                model.to_scale_rotation_translation().1,
            ));
            self.cam_pos_loc
                .set_vec3(&view_inv.to_scale_rotation_translation().2);
            self.light_pos_loc.set_vec3(&light_pos);

            cb();
        });
    }
}
