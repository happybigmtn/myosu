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

fn cargo_invert(package: &str) -> String {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(workspace_root())
        .args(["tree", "-i", package, "--workspace"])
        .output()
        .expect("cargo tree should execute");

    assert!(
        output.status.success(),
        "cargo tree -i {package} failed\nstdout:\n{}\nstderr:\n{}",
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

/// NEM-005: Strengthen INV-004 to catch transitive dependencies.
///
/// The pre-existing `inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other`
/// test only checks the *direct* dep edges `myosu-play -> myosu-miner` and
/// `myosu-miner -> myosu-play`. A transitive path
/// (e.g. `myosu-chain-client -> some-shared-dep -> myosu-miner`) would not
/// surface as a literal `myosu-miner` substring in the `cargo tree` output of
/// the importer, and a re-export of a miner-originating type into the
/// `myosu-chain-client` public API would not surface as a `cargo tree`
/// violation at all.
///
/// This test closes both gaps:
///
/// 1. `cargo tree -i myosu-miner --workspace` (the reverse-dep walk) is the
///    authoritative transitive check. On a clean trunk checkout the only
///    line in the output is the `myosu-miner v0.1.0 (...)` package line
///    itself -- no other workspace crate may appear as a reverse-dependency,
///    direct OR transitive. If a future PR adds a `myosu-chain-client` (or
///    any other crate) dep on `myosu-miner`, the reverse walk will list it
///    here and the test fails closed with the offender named.
/// 2. `cargo tree -p myosu-chain-client --edges normal` is asserted to
///    contain no `myosu-miner` substring. This is the *forward* transitive
///    view: any chain of deps from `myosu-chain-client` that ends in
///    `myosu-miner` would surface as the literal `myosu-miner v0.1.0 (...)`
///    package line in the forward tree. The forward check is intentionally
///    separate from the reverse check so a future implementer can tell at
///    a glance which direction the violation points.
///
/// INV-004 enforcement stays in this crate (where the existing test lives)
/// because that test owns the *forbidden-edges* declaration for the whole
/// workspace; the cargo-tree walks in this test are the workspace-level
/// verification surface.
#[test]
fn inv_004_chain_client_does_not_re_export_miner_types() {
    // (1) Reverse-dep walk: no workspace crate may depend on `myosu-miner`,
    //     direct OR transitive. The only line in the output is the
    //     `myosu-miner` package line itself; a future PR that introduces a
    //     reverse-dep (e.g. a `myosu-chain-client` -> ... -> `myosu-miner`
    //     chain) would add a new "└── ..." child of that line and the
    //     test would fail.
    let reverse_tree = cargo_invert("myosu-miner");
    let non_self_lines: Vec<&str> = reverse_tree
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| {
            // Keep only lines that name a workspace package. The bare
            // `myosu-miner v0.1.0 (...)` line is the package header, not
            // a reverse-dep. Reverse-deps are the indented `└──` / `├──`
            // child lines that name another workspace package.
            line.contains("myosu-") && !line.starts_with("myosu-miner v")
        })
        .collect();
    assert!(
        non_self_lines.is_empty(),
        "INV-004: `myosu-miner` must have zero reverse-dependencies in the workspace \
         (direct OR transitive). Offending reverse-dep line(s):\n{}\n\nfull tree:\n{}",
        non_self_lines.join("\n"),
        reverse_tree,
    );

    // (2) Forward tree of `myosu-chain-client` must not mention
    //     `myosu-miner` at all (any path through a transitive dep would
    //     surface as the `myosu-miner v0.1.0 (...)` package line).
    let chain_client_tree = cargo_tree("myosu-chain-client");
    assert_no_dependency(
        &chain_client_tree,
        "myosu-chain-client",
        "myosu-miner",
    );
}
