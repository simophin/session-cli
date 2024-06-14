use std::env;
use std::path::PathBuf;

fn main() {

    let dst = cmake::Config::new("libsession-util")
        .define("STATIC_BUNDLE", "ON")
        .define("BUILD_STATIC_DEPS", "ON")
        .define("STATIC_LIBSTD", "ON")
        .env("CMAKE_BUILD_PARALLEL_LEVEL", "8")
        .build_target("session-util")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=session-util");

    let bindings = bindgen::Builder::default()
        .detect_include_paths(true)
        // The input header we would like to generate
        // bindings for.
        .header("session.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}