const OCCT_DIR: &str = "build/occt";
const OCCT_LIBS: &[&str] = &[
    "TKMath",
    "TKernel",
    "TKFeat",
    "TKGeomBase",
    "TKG2d",
    "TKG3d",
    "TKTopAlgo",
    "TKGeomAlgo",
    "TKBRep",
    "TKPrim",
    "TKMesh",
    "TKShHealing",
    "TKFillet",
    "TKBool",
    "TKBO",
    "TKOffset"
];

use std::env;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let occt_dir = format!("{dir}/{OCCT_DIR}");

    println!("cargo:rustc-link-search=native={occt_dir}/lib");
        
    for lib in OCCT_LIBS {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    cxx_build::bridge("src/occt.rs")
        .cpp(true)
        .file("src/occt.cpp")
        .std("c++17")
        .include(format!("{occt_dir}/include/opencascade"))
        .compile("occt");

    println!("cargo:rerun-if-changed=src/occt.rs");
    println!("cargo:rerun-if-changed=src/occt.cpp");
    println!("cargo:rerun-if-changed=src/occt.h");
}