fn main() {
    println!("cargo:rerun-if-changed=mysort.c");

    cc::Build::new()
        .compiler("/usr/bin/clang")
        .file("mysort.c")
        .opt_level(2)
        .debug(true)
        .compile("mysort");
}
