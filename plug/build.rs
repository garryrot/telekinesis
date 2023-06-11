use std::env;

extern crate cbindgen;

fn main() {
    cbindgen::generate(env::var("CARGO_MANIFEST_DIR").unwrap())
                .expect("Unable to generate bindings")
                .write_to_file("include/telekinesis_plug.h");
}
