use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut config = cmake::Config::new(&src_dir);

    let out_dir = src_dir.join("cmake-rust-build");

    config
        .define("STATIC_BUNDLE", "ON")
        .define("BUILD_STATIC_DEPS", "ON")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("STATIC_LIBSTD", "ON")
        .define("OXEN_LOGGING_FMT_HEADER_ONLY", "ON")
        .env("CMAKE_BUILD_PARALLEL_LEVEL", "8")
        .always_configure(false)
        .generator("Ninja")
        .out_dir(&out_dir);

    for target in ["session-json", "session-util"] {
        config.build_target(target).build();
    }

    for search_path in ["libsession-json", "libsession-util"] {
        println!(
            "cargo:rustc-link-search=native={}",
            out_dir.join("build").join(search_path).display()
        );
    }

    for lib in ["session-json", "session-util"] {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    let bindings = bindgen::Builder::default()
        .detect_include_paths(true)
        .clang_arg("-Ilibsession-util/include")
        // The input header we would like to generate
        // bindings for.
        .header("session.h")
        .derive_eq(true)
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

    let descriptor_path = out_path.join("proto_descriptor.bin");

    let proto_files = [
        "libsession-util/proto/SessionProtos.proto",
        "libsession-util/proto/WebSocketResources.proto",
        "protos/app.proto",
    ]
    .map(|s| src_dir.join(s).to_path_buf());

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types()
        .compile_protos(
            proto_files.as_slice(),
            proto_files
                .iter()
                .map(|f| f.parent().unwrap().to_path_buf())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .unwrap();

    let descriptor_set = std::fs::read(descriptor_path).expect("To read descriptor file");

    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)
        .expect("To register descriptors")
        .build(&[".SessionProtos", ".WebSocketProtos", ".SessionCliApp"])
        .expect("To build pbjson");

    // Tell cargo to track the proto files
    for proto_file in proto_files {
        println!("cargo:rerun-if-changed={}", proto_file.display());
    }
}
