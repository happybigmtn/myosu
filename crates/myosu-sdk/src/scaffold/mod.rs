//! Scaffold tool for generating new game engine crates.
//!
//! The `ScaffoldGenerator` creates a new `myosu-games-<name>` crate
//! from a template, providing compileable stubs for `CfrGame`,
//! `Encoder`, and the optional `GameRenderer`.
//!
//! # Example
//!
//! ```rust,ignore
//! use myosu_sdk::scaffold::ScaffoldGenerator;
//!
//! let generator = ScaffoldGenerator::new("kuhn-poker")?;
//! let path = generator.generate()?;
//! println!("generated {}", path.display());
//! ```

mod templates;

use std::path::{Component, Path, PathBuf};

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
        if name.is_empty() {
            return Err(ScaffoldError::InvalidName("empty".to_string()));
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(ScaffoldError::InvalidName(name.to_string()));
        }

        Ok(Self {
            name: name.to_string(),
        })
    }

    /// Get the validated game name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the generated crate name.
    pub fn crate_name(&self) -> String {
        format!("myosu-games-{}", self.name)
    }

    /// Generate a crate into the conventional `myosu-games-<name>` directory.
    pub fn generate(&self) -> Result<PathBuf, ScaffoldError> {
        let path = PathBuf::from(self.crate_name());
        self.generate_into(&path)?;
        Ok(path)
    }

    /// Generate the crate at the given path.
    pub fn generate_into(&self, path: &Path) -> Result<(), ScaffoldError> {
        if path.exists() {
            return Err(ScaffoldError::DirectoryExists(path.to_path_buf()));
        }

        std::fs::create_dir_all(path)?;

        let sdk_dependency_path = sdk_dependency_path(path)?;

        std::fs::write(
            path.join("Cargo.toml"),
            templates::cargo_toml(&self.crate_name(), &sdk_dependency_path),
        )?;

        std::fs::create_dir_all(path.join("src"))?;
        std::fs::write(path.join("src/lib.rs"), templates::lib_rs(&self.name))?;
        std::fs::write(path.join("src/game.rs"), templates::game_rs(&self.name))?;
        std::fs::write(
            path.join("src/encoder.rs"),
            templates::encoder_rs(&self.name),
        )?;
        std::fs::write(
            path.join("src/renderer.rs"),
            templates::renderer_rs(&self.name),
        )?;
        std::fs::write(path.join("src/tests.rs"), templates::tests_rs(&self.name))?;
        std::fs::write(path.join("README.md"), templates::readme_md(&self.name))?;

        Ok(())
    }
}

fn sdk_dependency_path(target_dir: &Path) -> Result<String, ScaffoldError> {
    let sdk_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
    let target_dir = target_dir.canonicalize()?;
    Ok(relative_path(&target_dir, &sdk_dir)
        .to_string_lossy()
        .into_owned())
}

fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let shared_prefix = from_components
        .iter()
        .zip(&to_components)
        .take_while(|(left, right)| left == right)
        .count();

    let mut relative = PathBuf::new();

    for component in &from_components[shared_prefix..] {
        if matches!(component, Component::Normal(_) | Component::ParentDir) {
            relative.push("..");
        }
    }

    for component in &to_components[shared_prefix..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

#[cfg(test)]
mod tests;
