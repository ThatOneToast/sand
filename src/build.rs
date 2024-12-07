use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let parser_src_dir = Path::new(&manifest_dir).join("tree-sitter-sand/src");
    if !parser_src_dir.exists() {
        panic!("Tree-sitter parser source directory not found!");
    }

    // Compile parser.c
    let parser_output = Command::new("clang")
        .args(&[
            "-c",
            &parser_src_dir.join("parser.c").to_string_lossy(),
            "-o", "parser.o",
            "-I", &parser_src_dir.to_string_lossy(),
            "-fPIC",
            "-arch", "arm64"
        ])
        .status()
        .expect("Failed to compile parser.c");

    if !parser_output.success() {
        panic!("Failed to compile parser.c");
    }

    // Check for optional scanner.c
    let scanner_path = parser_src_dir.join("scanner.c");
    if scanner_path.exists() {
        let scanner_output = Command::new("clang")
            .args(&[
                "-c",
                &scanner_path.to_string_lossy(),
                "-o", "scanner.o",
                "-I", &parser_src_dir.to_string_lossy(),
                "-fPIC",
                "-arch", "arm64"
            ])
            .status()
            .expect("Failed to compile scanner.c");

        if !scanner_output.success() {
            panic!("Failed to compile scanner.c");
        }
    }

    // Create static library
    let ar_output = Command::new("ar")
        .args(&[
            "crs",
            "libsand_parser.a",
            "parser.o"
        ])
        .status()
        .expect("Failed to create static library");

    if !ar_output.success() {
        panic!("Failed to create static library");
    }

    if scanner_path.exists() {
        let ar_add_output = Command::new("ar")
            .args(&[
                "rs",
                "libsand_parser.a",
                "scanner.o"
            ])
            .status()
            .expect("Failed to add scanner to static library");

        if !ar_add_output.success() {
            panic!("Failed to add scanner to static library");
        }
    }

    println!("cargo:rustc-link-search=.");
    println!("cargo:rustc-link-lib=static=sand_parser");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tree-sitter-sand/src/parser.c");

    if scanner_path.exists() {
        println!("cargo:rerun-if-changed=tree-sitter-sand/src/scanner.c");
    }
}
