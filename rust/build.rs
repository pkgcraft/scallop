use std::env;
use std::path::PathBuf;

fn main() {
    // generate scallop-specific bindings
    // link to the scallop library
    println!("cargo:rustc-link-lib=scallop");
    //println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        // header to generate bindings for
        .header("../src/scallop.h")
        // invalidate built crate whenever any included header file changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("scallop-bindings.rs"))
        .expect("Couldn't write bindings!");

    // generate bash-specific bindings
    println!("cargo:rerun-if-changed=bash-wrapper.h");
    let bindings = bindgen::Builder::default()
        // header to generate bindings for
        .header("bash-wrapper.h")
        // invalidate built crate whenever any included header file changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bash-bindings.rs"))
        .expect("Couldn't write bindings!");
}
