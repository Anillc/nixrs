use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=nixexprc");
    println!("cargo:rustc-link-lib=nixstorec");
    println!("cargo:rustc-link-lib=nixutilc");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("failed to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
}
