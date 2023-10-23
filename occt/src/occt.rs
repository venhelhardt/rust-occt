#[cxx::bridge(namespace = "occt")]
pub(crate) mod ffi {
    struct Tuple3f
    {
        x: f32,
        y: f32,
        z: f32
    }

    struct MeshVerts
    {
        count: u32,
        ptr: * const f32
    }

    struct MeshNorms
    {
        count: u32,
        ptr: * const f32
    }

    struct MeshTris
    {
        count: u32,
        ptr: * const u32
    }

    struct MeshBbox
    {
        min: Tuple3f,
        max: Tuple3f
    }

    unsafe extern "C++" {
        include!("occt/src/occt.h");

        type MeshBlob;

        fn bbox(&self) -> MeshBbox;
        fn verts(&self) -> MeshVerts;
        fn norms(&self) -> MeshNorms;
        fn tris(&self) -> MeshTris;

        fn make_flask(width: f64, thickness: f64, height: f64) -> UniquePtr<MeshBlob>;
    }
}

unsafe impl Sync for ffi::MeshBlob {}
unsafe impl Send for ffi::MeshBlob {}