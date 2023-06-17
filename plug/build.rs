
fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("src/api.cc")
        .compile("telekinesis_plug");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/api.cc");
}
