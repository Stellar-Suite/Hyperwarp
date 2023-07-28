// adapted from example
fn main() {
    println!("cargo:rerun-if-changed=glue.c");
    println!("cargo:rerun-if-changed=preglue.c");
    // hacky workaround for binary
    // TODO: workaround by dlsyming
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    // println!("cargo:rustc-flags=-Wl,--allow-multiple-definition");
    // Use the `cc` crate to build a C file and statically link it.
    /*cc::Build::new()
        .file("glue.c")
        .compile("libhyperglue");*/
    cc::Build::new()
        .file("glue_header.c")
        .compile("libhyperglueheader");
    
}   