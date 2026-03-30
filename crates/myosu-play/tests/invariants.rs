use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("myosu-play should live under crates/ within the workspace root")
        .to_path_buf()
}

fn cargo_tree(package: &str) -> String {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(workspace_root())
        .args(["tree", "-p", package, "--edges", "normal"])
        .output()
        .expect("cargo tree should execute");

    assert!(
        output.status.success(),
        "cargo tree failed for {package}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    String::from_utf8(output.stdout).expect("cargo tree output should be utf-8")
}

fn assert_no_dependency(tree: &str, source_package: &str, forbidden_package: &str) {
    assert!(
        !tree.contains(forbidden_package),
        "{source_package} must not depend on {forbidden_package} (INV-004)\n\n{tree}",
    );
}

#[test]
fn inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other() {
    let play_tree = cargo_tree("myosu-play");
    assert_no_dependency(&play_tree, "myosu-play", "myosu-miner");

    let miner_tree = cargo_tree("myosu-miner");
    assert_no_dependency(&miner_tree, "myosu-miner", "myosu-play");
}
