//! Scaffold tool for generating new game engine crates.
//!
//! The `ScaffoldGenerator` creates a new `myosu-games-<name>` crate
//! from a template, providing stub implementations of `CfrGame`, `Encoder`,
//! and `GameRenderer` that the developer fills in.
//!
//! # Example
//!
//! ```rust,ignore
//! use myosu_sdk::scaffold::ScaffoldGenerator;
//!
//! let generator = ScaffoldGenerator::new("kuhn-poker")?;
//! generator.generate()?;
//! ```

pub mod templates;
pub mod tests;

use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScaffoldError {
    #[error("directory already exists: {0}")]
    DirectoryExists(PathBuf),

    #[error("invalid game name: {0}")]
    InvalidName(String),

    #[error("failed to write file: {0}")]
    WriteError(#[from] std::io::Error),
}

/// Generator for scaffolding a new game crate.
pub struct ScaffoldGenerator {
    name: String,
}

impl ScaffoldGenerator {
    /// Create a new generator for the given game name.
    pub fn new(name: &str) -> Result<Self, ScaffoldError> {
        // Validate name: must be a valid Rust identifier (no spaces, etc.)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ScaffoldError::InvalidName(name.to_string()));
        }
        if name.is_empty() {
            return Err(ScaffoldError::InvalidName("empty".to_string()));
        }

        Ok(Self {
            name: name.to_string(),
        })
    }

    /// Get the validated name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Generate the crate at the given path.
    pub fn generate_into(self, path: &Path) -> Result<(), ScaffoldError> {
        if path.exists() {
            return Err(ScaffoldError::DirectoryExists(path.to_path_buf()));
        }

        std::fs::create_dir_all(path)?;

        let crate_name = format!("myosu-games-{}", self.name.replace('-', "_"));

        // Write Cargo.toml
        let cargo_toml = templates::cargo_toml(&crate_name);
        std::fs::write(path.join("Cargo.toml"), cargo_toml)?;

        // Create src directory
        std::fs::create_dir_all(path.join("src"))?;

        // Write src/lib.rs
        std::fs::write(path.join("src/lib.rs"), templates::lib_rs(&crate_name))?;

        // Write src/game.rs
        std::fs::write(path.join("src/game.rs"), templates::game_rs(&self.name))?;

        // Write src/encoder.rs
        std::fs::write(path.join("src/encoder.rs"), templates::encoder_rs())?;

        // Write src/renderer.rs (with tui feature)
        std::fs::write(path.join("src/renderer.rs"), templates::renderer_rs())?;

        // Write src/tests.rs
        std::fs::write(path.join("src/tests.rs"), templates::tests_rs())?;

        // Write README.md
        std::fs::write(path.join("README.md"), templates::readme_md(&self.name))?;

        Ok(())
    }
}

use std::path::PathBuf;
