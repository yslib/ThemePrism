use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let source = Path::new("src/platform/gui/macos/theme_gui_host.m");
    println!("cargo:rerun-if-changed={}", source.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    let object = out_dir.join("theme_gui_host.o");
    let library = out_dir.join("libtheme_gui_host.a");

    run(Command::new("clang").args([
        "-fobjc-arc",
        "-c",
        source
            .to_str()
            .expect("Objective-C source path should be valid UTF-8"),
        "-o",
        object
            .to_str()
            .expect("Objective-C object path should be valid UTF-8"),
    ]));

    run(Command::new("ar").args([
        "-crus",
        library
            .to_str()
            .expect("static library path should be valid UTF-8"),
        object.to_str().expect("object path should be valid UTF-8"),
    ]));

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=theme_gui_host");
    println!("cargo:rustc-link-lib=framework=Cocoa");
}

fn run(command: &mut Command) {
    let status = command.status().expect("failed to spawn build tool");
    assert!(status.success(), "build tool failed with status: {status}");
}
