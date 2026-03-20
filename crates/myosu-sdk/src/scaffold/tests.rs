//! Tests for the scaffold generator.

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use super::super::ScaffoldGenerator;

    #[test]
    fn generates_compilable_crate() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("myosu-games-test-game");

        let generator = ScaffoldGenerator::new("test-game").unwrap();
        generator.generate_into(&target_dir).unwrap();

        // Verify the generated Cargo.toml exists
        assert!(target_dir.join("Cargo.toml").exists());

        // Verify src/lib.rs exists
        assert!(target_dir.join("src/lib.rs").exists());

        // Verify src/game.rs exists with todo!()
        let game_rs = std::fs::read_to_string(target_dir.join("src/game.rs")).unwrap();
        assert!(game_rs.contains("todo!"));

        // Verify src/encoder.rs exists
        assert!(target_dir.join("src/encoder.rs").exists());

        // Verify src/renderer.rs exists
        assert!(target_dir.join("src/renderer.rs").exists());

        // Verify src/tests.rs exists
        assert!(target_dir.join("src/tests.rs").exists());

        // Verify README.md exists
        assert!(target_dir.join("README.md").exists());
    }

    #[test]
    fn generated_tests_fail_with_todo() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("myosu-games-todo-game");

        let generator = ScaffoldGenerator::new("todo-game").unwrap();
        generator.generate_into(&target_dir).unwrap();

        // Read the tests.rs and verify it has todo!
        let tests_rs = std::fs::read_to_string(target_dir.join("src/tests.rs")).unwrap();
        assert!(tests_rs.contains("todo!"));
    }

    #[test]
    fn refuses_to_overwrite_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("existing-game");

        // Create the directory first
        std::fs::create_dir(&target_dir).unwrap();

        let generator = ScaffoldGenerator::new("existing-game").unwrap();
        let result = generator.generate_into(&target_dir);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already exists"));
    }

    #[test]
    fn validates_game_name() {
        // Invalid names should be rejected
        let result = ScaffoldGenerator::new("invalid name with spaces");
        assert!(result.is_err());

        let result = ScaffoldGenerator::new("");
        assert!(result.is_err());
    }

    #[test]
    fn accepts_valid_game_names() {
        // Valid names should be accepted
        assert!(ScaffoldGenerator::new("kuhn-poker").is_ok());
        assert!(ScaffoldGenerator::new("liars_dice").is_ok());
        assert!(ScaffoldGenerator::new("rockpaperscissors").is_ok());
        assert!(ScaffoldGenerator::new("game123").is_ok());
    }
}
