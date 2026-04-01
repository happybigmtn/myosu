use std::path::{Path, PathBuf};

fn main() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("runtime crate should live under the workspace root");
    // SAFETY: build scripts are single-process setup code here; we set the hint
    // before spawning the wasm builder so it can find the workspace Cargo.lock.
    unsafe {
        std::env::set_var("WASM_BUILD_WORKSPACE_HINT", workspace_root);
    }

    if std::env::var_os("SKIP_WASM_BUILD").is_some()
        && let Some(cached_wasm) = find_cached_runtime_wasm(workspace_root)
    {
        write_cached_wasm_binary_module(&cached_wasm);
        println!(
            "cargo:warning=using cached myosu-chain runtime wasm at {}",
            cached_wasm.display()
        );
        return;
    }

    substrate_wasm_builder::WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .build();
}

fn find_cached_runtime_wasm(workspace_root: &Path) -> Option<PathBuf> {
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let candidates = [
        target_root
            .join(&profile)
            .join("wbuild")
            .join("myosu-chain-runtime")
            .join("myosu_chain_runtime.wasm"),
        target_root
            .join("debug")
            .join("wbuild")
            .join("myosu-chain-runtime")
            .join("myosu_chain_runtime.wasm"),
        target_root
            .join("release")
            .join("wbuild")
            .join("myosu-chain-runtime")
            .join("myosu_chain_runtime.wasm"),
    ];

    candidates.into_iter().find(|candidate| candidate.is_file())
}

fn write_cached_wasm_binary_module(cached_wasm: &Path) {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR should be present"));
    let module_path = out_dir.join("wasm_binary.rs");
    let cached_wasm = cached_wasm
        .to_str()
        .expect("cached wasm path should be valid UTF-8");
    let module = format!(
        "pub const WASM_BINARY_PATH: Option<&str> = Some({path:?});\
pub const WASM_BINARY: Option<&[u8]> = Some(include_bytes!({path:?}));\
pub const WASM_BINARY_BLOATY: Option<&[u8]> = None;",
        path = cached_wasm,
    );
    std::fs::write(&module_path, module).expect("cached wasm module should write");
}
