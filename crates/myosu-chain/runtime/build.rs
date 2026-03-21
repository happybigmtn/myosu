use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build.rs"));
    let wasm_binary = out_dir.join("wasm_binary.rs");

    fs::write(
        wasm_binary,
        "pub const WASM_BINARY_PATH: Option<&str> = None;\n\
         pub const WASM_BINARY: Option<&[u8]> = None;\n\
         pub const WASM_BINARY_BLOATY: Option<&[u8]> = None;\n",
    )
    .expect("write dummy wasm binary include");
}
