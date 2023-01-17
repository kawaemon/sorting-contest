fn main() {
    cc::Build::new()
        .file("mysort.c")
        .opt_level(2)
        .compile("mysort");
}
