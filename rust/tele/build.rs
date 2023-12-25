
fn main() {
    let bridges = vec!["src/lib.rs","src/logging.rs"];
    cxx_build::bridges(bridges)
        .compile("telekinesis_plug");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/logging.rs");
}
