extern crate bindgen;
extern crate walkdir;

use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let idf_components_path = PathBuf::from(env::var("IDF_PATH").unwrap()).join("components");
    let xtensa_toolchain_path = PathBuf::from(env::var("XTENSA_TOOLCHAIN_ROOT").unwrap());
    let mut include_paths = Vec::new();
    for entry in WalkDir::new(idf_components_path.to_str().unwrap()).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "include" {
            include_paths.push("-I".to_owned() + entry.path().to_str().unwrap());
        }
    }
    
    let bindings = bindgen::Builder::default()
        .clang_arg("-nostdinc")
        .clang_args(include_paths)
        .clang_arg("-I".to_owned() + xtensa_toolchain_path.clone().join("xtensa-esp32-elf").join("include").to_str().unwrap())
        .clang_arg("-I".to_owned() + xtensa_toolchain_path.clone().join("lib").join("gcc").join("xtensa-esp32-elf").join("5.2.0").join("include").to_str().unwrap())
        .clang_arg("-I".to_owned() + xtensa_toolchain_path.clone().join("lib").join("gcc").join("xtensa-esp32-elf").join("5.2.0").join("include-fixed").to_str().unwrap())
        .clang_arg("-I".to_owned() + idf_components_path.clone().join("newlib").join("platform_include").to_str().unwrap())
        .clang_arg("-I".to_owned() + idf_components_path.clone().join("lwip").join("include").join("apps").to_str().unwrap())
        .clang_arg("-I../../../build/include")
        .use_core()
        .disable_untagged_union()
        .generate_comments(false)
        .rustfmt_bindings(true)
        .layout_tests(false)
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .whitelist_function(r"(esp|ESP)_.+")
        .whitelist_function(r"(spi_|spicommon_).+")
        .whitelist_function(r"(i2c_|I2C_).+")
        .whitelist_function(r"(gpio|GPIO)_.+")
        .whitelist_function(r"nvs_flash_.+")
        .whitelist_function(r"tcpip_.+")
        .whitelist_function(r"ip(4|6)addr_.+")
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
