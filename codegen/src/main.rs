use bindgen::callbacks::ParseCallbacks;
use bindgen::RustTarget;
use std::env;
use std::io::BufRead;

fn main() {
    generate_bindings();
}

fn rust_target_version() -> RustTarget {
    // try to find it in ../Cargo.toml
    match std::fs::File::open("../Cargo.toml") {
        Ok(cargo_toml_file) => {
            let cargo_toml_reader = std::io::BufReader::new(cargo_toml_file);

            for line in cargo_toml_reader.lines() {
                let line = line.unwrap();
                if line.starts_with("rust-version") {
                    // get version, remove quotes
                    return line.split('"').nth(1).unwrap().parse().unwrap();
                }
            }
        }
        Err(_) => {
            // ignore
        }
    }
    // if not found, use the environment variable from Cargo build
    env::var("CARGO_PKG_RUST_VERSION").unwrap().parse().unwrap()
}

fn generate_bindings() {
    let rust_target: RustTarget = rust_target_version();

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
        // remove trailing "<"
        // (it can happen in the syntax /**< ... */)
        let comment = comment.trim_start_matches("< ");
        match doxygen_bindgen::transform(comment) {
            Ok(res) => Some(res),
            Err(err) => {
                eprintln!("Failed to process doxygen comment: {comment}\n{err}");
                None
            }
        }
    }
}
