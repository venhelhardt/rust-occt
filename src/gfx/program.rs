use std::ffi::CString;

use gl::types::{GLint, GLuint};
use glam::{Mat3, Mat4, Vec3};

use super::Shader;

pub struct Program {
    pub(super) id: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) };
    }
}

impl Program {
    pub fn link(shaders: &[&Shader]) -> Result<Program, String> {
        let program = Program {
            id: unsafe { gl::CreateProgram() },
        };

        let successful: bool;

        unsafe {
            for shader in shaders.iter() {
                gl::AttachShader(program.id, shader.id);
            }

            gl::LinkProgram(program.id);

            successful = {
                let mut result: GLint = 0;
                gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut result);
                result != 0
            };
        }

        if successful {
            Ok(program)
        } else {
            Err("Can't link program".to_string())
        }
    }

    pub fn activate<F>(&self, cb: F)
    where
        F: FnOnce(),
    {
        let mut prev_prog_id: GLint = 0;

        unsafe {
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut prev_prog_id);
            gl::UseProgram(self.id);
        };

        cb();

        unsafe { gl::UseProgram(prev_prog_id as GLuint) };
    }

    pub fn get_attrib(&self, name: &str) -> Option<Attrib> {
        let c_name = CString::new(name).unwrap();
        match unsafe { gl::GetAttribLocation(self.id, c_name.as_ptr()) } {
            -1 => None,
            id => Some(Attrib { id: id }),
        }
    }

    pub fn get_uniform(&self, name: &str) -> Option<Uniform> {
        let c_name = CString::new(name).unwrap();
        match unsafe { gl::GetUniformLocation(self.id, c_name.as_ptr()) } {
            -1 => None,
            id => Some(Uniform { id: id }),
        }
    }
}

pub struct Attrib {
    pub(super) id: GLint,
}

impl Attrib {}

pub struct Uniform {
    pub(super) id: GLint,
}

impl Uniform {
    pub fn set_vec3(&self, vec: &Vec3) {
        unsafe {
            gl::Uniform3f(self.id, vec.x, vec.y, vec.z);
        };
    }

    pub fn set_mat3(&self, mat: &Mat3) {
        unsafe {
            gl::UniformMatrix3fv(self.id, 1, gl::FALSE, &mat.to_cols_array()[0]);
        };
    }

    pub fn set_mat4(&self, mat: &Mat4) {
        unsafe {
            gl::UniformMatrix4fv(self.id, 1, gl::FALSE, &mat.to_cols_array()[0]);
        };
    }
}
