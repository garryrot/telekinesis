
fn main() {
    let _res = cxx_build::bridges(vec!["src/lib.rs", "src/logging.rs"]);
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/logging.rs");
}