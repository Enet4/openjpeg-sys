use bindgen::callbacks::ParseCallbacks;
use bindgen::RustTarget;
use std::env;

fn main() {
    generate_bindings();
}

fn generate_bindings() {
    let rust_target: RustTarget = env::var("CARGO_PKG_RUST_VERSION").unwrap().parse().unwrap();

    let bindings = bindgen::Builder::default()
        .header("../vendor/src/lib/openjp2/openjpeg.h")
        .clang_args(["-I../vendor/src/lib/openjp2", "-I../config"])
        // replace FILE
        .blocklist_type("FILE")
        .raw_line("use libc::FILE;")
        // replace OPJ_BYTE
        .blocklist_type("OPJ_BYTE")
        .raw_line("pub type OPJ_BYTE = u8;")
        // other allowlists and blocklists
        .blocklist_type("^_.*")
        .allowlist_function("^opj.*")
        .allowlist_type("^opj.*")
        .allowlist_var("^OPJ.*")
        .rust_edition(bindgen::RustEdition::Edition2021)
        .rust_target(rust_target)
        .rustified_enum(".*")
        .trust_clang_mangling(false)
        .layout_tests(false)
        // convert doxygen to rustdoc
        .parse_callbacks(Box::new(ProcessComments))
        // generate the bindings
        .generate()
        .expect("Unable to generate bindings");

    // write the bindings to the main project as ffi.rs
    bindings
        .write_to_file("../src/ffi.rs")
        .expect("Couldn't write bindings!");
}

/// Use `doxygen_bindgen` to process doc comments
#[derive(Debug)]
struct ProcessComments;

impl ParseCallbacks for ProcessComments {
    fn process_comment(&self, comment: &str) -> Option<String> {
        match doxygen_bindgen::transform(comment) {
            Ok(res) => Some(res),
            Err(err) => {
                eprintln!("Failed to process doxygen comment: {comment}\n{err}");
                None
            }
        }
    }
}
