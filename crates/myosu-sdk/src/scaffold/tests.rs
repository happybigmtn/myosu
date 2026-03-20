use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

use super::ScaffoldGenerator;

#[test]
fn generates_compilable_crate() {
    let temp_dir = TempDir::new().expect("tempdir");
    let target_dir = temp_dir.path().join("myosu-games-test-game");

    let generator = ScaffoldGenerator::new("test-game").expect("valid name");
    generator
        .generate_into(&target_dir)
        .expect("scaffold should generate");

    assert!(target_dir.join("Cargo.toml").exists());
    assert!(target_dir.join("src/lib.rs").exists());
    assert!(target_dir.join("src/game.rs").exists());
    assert!(target_dir.join("src/encoder.rs").exists());
    assert!(target_dir.join("src/renderer.rs").exists());
    assert!(target_dir.join("src/tests.rs").exists());
    assert!(target_dir.join("README.md").exists());

    let cargo_toml = std::fs::read_to_string(target_dir.join("Cargo.toml")).expect("read Cargo");
    assert!(cargo_toml.contains("name = \"myosu-games-test-game\""));
    assert!(cargo_toml.contains("tui = [\"myosu-sdk/tui\"]"));

    let game_rs = std::fs::read_to_string(target_dir.join("src/game.rs")).expect("read game");
    assert!(game_rs.contains("history: [Option<GameAction>; MAX_HISTORY]"));
    assert!(game_rs.contains("impl CfrGame for Game"));

    let default_check = cargo(&target_dir, ["check", "--offline"]);
    assert!(
        default_check.status.success(),
        "generated crate should compile:\n{}",
        render_output(&default_check)
    );

    let tui_check = cargo(&target_dir, ["check", "--offline", "--features", "tui"]);
    assert!(
        tui_check.status.success(),
        "generated crate should compile with tui enabled:\n{}",
        render_output(&tui_check)
    );
}

#[test]
fn generated_tests_fail_with_todo() {
    let temp_dir = TempDir::new().expect("tempdir");
    let target_dir = temp_dir.path().join("myosu-games-todo-game");

    let generator = ScaffoldGenerator::new("todo-game").expect("valid name");
    generator
        .generate_into(&target_dir)
        .expect("scaffold should generate");

    let test_run = cargo(&target_dir, ["test", "--offline", "--color", "never"]);
    let combined = render_output(&test_run);

    assert!(!test_run.status.success(), "generated tests should fail");
    assert!(
        combined.contains("replace this todo with a real compliance assertion"),
        "expected scaffold todo failure, got:\n{combined}"
    );
}

#[test]
fn refuses_to_overwrite_existing_directory() {
    let temp_dir = TempDir::new().expect("tempdir");
    let target_dir = temp_dir.path().join("existing-game");

    std::fs::create_dir(&target_dir).expect("create existing directory");

    let generator = ScaffoldGenerator::new("existing-game").expect("valid name");
    let result = generator.generate_into(&target_dir);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn validates_game_name() {
    assert!(ScaffoldGenerator::new("invalid name with spaces").is_err());
    assert!(ScaffoldGenerator::new("").is_err());
    assert!(ScaffoldGenerator::new("UpperCase").is_err());
}

#[test]
fn accepts_valid_game_names() {
    assert!(ScaffoldGenerator::new("kuhn-poker").is_ok());
    assert!(ScaffoldGenerator::new("liars_dice").is_ok());
    assert!(ScaffoldGenerator::new("rockpaperscissors").is_ok());
    assert!(ScaffoldGenerator::new("game123").is_ok());
}

#[test]
fn generate_uses_default_directory_name() {
    let temp_dir = TempDir::new().expect("tempdir");
    let previous = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(temp_dir.path()).expect("chdir");
    let _restore = CurrentDirGuard(previous);

    let generator = ScaffoldGenerator::new("kuhn-poker").expect("valid name");
    let generated_path = generator.generate().expect("generate default path");

    assert_eq!(generated_path, PathBuf::from("myosu-games-kuhn-poker"));
    assert!(
        temp_dir
            .path()
            .join(&generated_path)
            .join("Cargo.toml")
            .exists()
    );
}

fn cargo<const N: usize>(manifest_dir: &Path, args: [&str; N]) -> Output {
    let target_dir = manifest_dir.join("target");
    let manifest_path = manifest_dir.join("Cargo.toml");

    Command::new("cargo")
        .args(args)
        .arg("--manifest-path")
        .arg(&manifest_path)
        .env("CARGO_TARGET_DIR", &target_dir)
        .current_dir(manifest_dir)
        .output()
        .expect("cargo command should launch")
}

fn render_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("stdout:\n{stdout}\n\nstderr:\n{stderr}")
}

struct CurrentDirGuard(PathBuf);

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.0).expect("restore cwd");
    }
}
