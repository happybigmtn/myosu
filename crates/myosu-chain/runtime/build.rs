#[cfg(feature = "std")]
fn main() {
    use std::{env, path::PathBuf};

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .ancestors()
        .nth(3)
        .expect("runtime crate lives under crates/myosu-chain/runtime");

    // The wasm builder walks upward from OUT_DIR to find the workspace lockfile. In this repo
    // the target dir can live outside the workspace, so we give it an explicit workspace root.
    unsafe {
        env::set_var("WASM_BUILD_WORKSPACE_HINT", workspace_root);
    }

    substrate_wasm_builder::WasmBuilder::build_using_defaults();
}

#[cfg(not(feature = "std"))]
fn main() {}
