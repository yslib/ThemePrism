use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let source = "src/platform/gui/macos/theme_gui_host.m";
    println!("cargo:rerun-if-changed={source}");

    cc::Build::new()
        .file(source)
        .flag("-fobjc-arc")
        .compile("theme_gui_host");

    println!("cargo:rustc-link-lib=framework=Cocoa");
}
