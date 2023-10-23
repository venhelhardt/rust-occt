use gl::types::{GLint, GLuint};

pub struct Shader {
    pub(super) id: GLuint,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) };
    }
}

impl Shader {
    pub fn vertex_source(src: &str) -> Result<Shader, String> {
        Self::make_from_source(src, gl::VERTEX_SHADER)
    }

    pub fn fragment_source(src: &str) -> Result<Shader, String> {
        Self::make_from_source(src, gl::FRAGMENT_SHADER)
    }

    fn make_from_source(src: &str, shader_type: GLuint) -> Result<Shader, String> {
        let shader = Shader {
            id: unsafe { gl::CreateShader(shader_type) },
        };

        unsafe {
            let ptr: *const u8 = src.as_bytes().as_ptr();
            let ptr_i8: *const i8 = std::mem::transmute(ptr);
            let len = src.len() as GLint;

            gl::ShaderSource(shader.id, 1, &ptr_i8, &len);
        }

        let successful = unsafe {
            gl::CompileShader(shader.id);

            let mut result: GLint = 0;

            gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut result);

            result != 0
        };

        if successful {
            Ok(shader)
        } else {
            let mut len = 0;

            unsafe { gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut len) };

            assert!(len > 0);

            let mut buf = Vec::with_capacity(len as usize);
            let buf_ptr = buf.as_mut_ptr() as *mut gl::types::GLchar;

            unsafe {
                gl::GetShaderInfoLog(shader.id, len, std::ptr::null_mut(), buf_ptr);
                buf.set_len(len as usize);
            };

            match String::from_utf8(buf) {
                Ok(log) => Err(log),
                Err(vec) => Err(format!(
                    "Could not convert compilation log from buffer: {}",
                    vec
                )),
            }
        }
    }
}
