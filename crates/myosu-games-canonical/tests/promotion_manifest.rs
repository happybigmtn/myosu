use std::{path::PathBuf, process::Command};

#[test]
fn promotion_manifest_example_prints_one_row_per_research_game() {
    let output = Command::new(env!("CARGO"))
        .current_dir(repo_root())
        .env("SKIP_WASM_BUILD", "1")
        .args([
            "run",
            "-p",
            "myosu-games-canonical",
            "--example",
            "promotion_manifest",
            "--quiet",
        ])
        .output()
        .unwrap_or_else(|error| panic!("promotion manifest example should run: {error}"));

    assert!(
        output.status.success(),
        "promotion manifest failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("promotion manifest stdout should be UTF-8: {error}"));
    let row_count = stdout
        .lines()
        .filter(|line| line.starts_with("SOLVER_PROMOTION_GAME "))
        .count();

    assert_eq!(row_count, 22);
    assert!(stdout.contains("slug=nlhe-heads-up"));
    assert!(stdout.contains("tier=benchmarked"));
    assert!(stdout.contains("code_bundle_support=benchmarked"));
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
