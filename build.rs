fn main() {

    println!("cargo:rerun-if-changed=src/version.c");
    println!("cargo:rerun-if-changed=src/version.def");
    println!("cargo:rustc-link-arg=/DEF src/version.def");
    let dst = cmake::build("version");
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=shared=version");
}