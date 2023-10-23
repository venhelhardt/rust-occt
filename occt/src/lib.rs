extern crate cxx;

mod occt;

pub type MeshBlob = cxx::UniquePtr<occt::ffi::MeshBlob>;

pub fn make_flask(width: f64, thickness: f64, height: f64) -> MeshBlob {
    occt::ffi::make_flask(width, thickness, height)
}
