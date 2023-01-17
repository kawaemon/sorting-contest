fn main() {
    println!("cargo:rerun-if-changed=mysort.c");

    cc::Build::new()
        .file("mysort.c")
        .opt_level(2)
        .compile("mysort");
}
