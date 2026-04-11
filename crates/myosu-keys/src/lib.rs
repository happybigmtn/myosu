use std::{
    env, io,
    path::{Path, PathBuf},
};

use hex::FromHexError;
use serde_json::Error as SerdeJsonError;
use sp_core::{Pair, crypto::Ss58Codec, sr25519};
use thiserror::Error;

mod storage;

pub use storage::{
    ListedOperatorAccount, LoadedOperatorAccount, OperatorConfig, StoredOperatorAccount,
    config_file_from_home, default_config_file, export_active_keyfile, import_keyfile,
    key_file_path, list_stored_accounts, load_active_pair, load_active_secret_uri,
    load_active_secret_uri_from_env, load_operator_config, load_pair_from_keyfile, save_mnemonic,
    save_pair, set_active_account,
};

/// Errors returned by the shared key helpers.
#[derive(Debug, Error)]
pub enum KeyError {
    /// The mnemonic could not be parsed into an sr25519 pair.
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    /// HOME is required to resolve the default operator directories.
    #[error("HOME is not set; cannot resolve the myosu key directory")]
    MissingHomeDir,
    /// The requested password environment variable was not present.
    #[error("environment variable {env} is not set; cannot unlock the myosu key")]
    MissingPasswordEnv { env: String },
    /// No operator key source was provided by a caller that bypassed CLI validation.
    #[error("missing operator key source; pass --key or --key-config-dir")]
    MissingKeySource,
    /// Creating a config or key directory failed.
    #[error("failed to create directory {path}: {source}")]
    CreateDirectory { path: PathBuf, source: io::Error },
    /// A requested path shape was invalid for the current operation.
    #[error("path {0} is not a valid operator file path")]
    InvalidPath(PathBuf),
    /// Reading a config or key file failed.
    #[error("failed to read {path}: {source}")]
    ReadFile { path: PathBuf, source: io::Error },
    /// Writing a config or key file failed.
    #[error("failed to write {path}: {source}")]
    WriteFile { path: PathBuf, source: io::Error },
    /// The config file contents were malformed.
    #[error("invalid operator config at {path}: {reason}")]
    InvalidConfig { path: PathBuf, reason: String },
    /// Serializing a key file failed.
    #[error("failed to serialize key file {path}: {source}")]
    SerializeKeyfile {
        path: PathBuf,
        source: SerdeJsonError,
    },
    /// Parsing a key file failed.
    #[error("failed to parse key file {path}: {source}")]
    DeserializeKeyfile {
        path: PathBuf,
        source: SerdeJsonError,
    },
    /// The key file metadata is malformed or unsupported.
    #[error("invalid key file at {path}: {reason}")]
    InvalidKeyfile { path: PathBuf, reason: String },
    /// The requested operator key file was not present on disk.
    #[error("expected operator key file {path} does not exist")]
    MissingKeyfile { path: PathBuf },
    /// One of the encoded keyfile byte fields was malformed.
    #[error("invalid hex field {field} in {path}: {source}")]
    InvalidKeyfileHex {
        path: PathBuf,
        field: String,
        source: FromHexError,
    },
    /// Deriving the symmetric encryption key failed.
    #[error("failed to derive the encryption key from the supplied password")]
    KeyDerivation,
    /// Encrypting seed bytes failed.
    #[error("failed to encrypt the operator seed")]
    EncryptSeed,
    /// Decrypting seed bytes failed.
    #[error("failed to decrypt the operator seed; verify the password and key file")]
    DecryptSeed,
    /// The decrypted seed bytes could not be rehydrated into an sr25519 pair.
    #[error("invalid decrypted seed material in {path}: {reason}")]
    InvalidSeedMaterial { path: PathBuf, reason: String },
    /// The decrypted key material cannot be expressed as a standard secret URI.
    #[error("key file {path} cannot be represented as a standard secret URI: {reason}")]
    SecretUriUnsupported { path: PathBuf, reason: String },
}

/// Generates a new sr25519 mnemonic phrase using Substrate's built-in path.
pub fn generate_mnemonic() -> String {
    let (_, phrase, _) = sr25519::Pair::generate_with_phrase(None);
    phrase
}

/// Derives an sr25519 pair from a mnemonic phrase.
pub fn mnemonic_to_pair(mnemonic: &str) -> Result<sr25519::Pair, KeyError> {
    let (pair, _) = sr25519::Pair::from_phrase(mnemonic, None)
        .map_err(|error| KeyError::InvalidMnemonic(format!("{error:?}")))?;
    Ok(pair)
}

/// Converts an sr25519 pair into a default-SS58 address.
pub fn pair_to_address(pair: &sr25519::Pair) -> String {
    pair.public().to_ss58check()
}

/// Derives the default-SS58 address directly from a mnemonic phrase.
pub fn mnemonic_to_address(mnemonic: &str) -> Result<String, KeyError> {
    let pair = mnemonic_to_pair(mnemonic)?;
    Ok(pair_to_address(&pair))
}

/// Returns the default Myosu config directory for a given home path.
pub fn config_dir_from_home(home: &Path) -> PathBuf {
    home.join(".myosu")
}

/// Returns the default Myosu key directory for a given home path.
pub fn keys_dir_from_home(home: &Path) -> PathBuf {
    config_dir_from_home(home).join("keys")
}

/// Resolves the default Myosu config directory from the current HOME.
pub fn default_config_dir() -> Result<PathBuf, KeyError> {
    let Some(home) = env::var_os("HOME") else {
        return Err(KeyError::MissingHomeDir);
    };
    Ok(config_dir_from_home(Path::new(&home)))
}

/// Resolves the default Myosu key directory from the current HOME.
pub fn default_keys_dir() -> Result<PathBuf, KeyError> {
    let Some(home) = env::var_os("HOME") else {
        return Err(KeyError::MissingHomeDir);
    };
    Ok(keys_dir_from_home(Path::new(&home)))
}

#[cfg(test)]
mod tests {
    use super::{
        config_dir_from_home, default_keys_dir, generate_mnemonic, keys_dir_from_home,
        mnemonic_to_address, mnemonic_to_pair, pair_to_address,
    };
    use std::path::Path;

    use sp_core::{Pair, crypto::Ss58Codec, sr25519};

    #[test]
    fn generate_mnemonic_is_12_words() {
        let mnemonic = generate_mnemonic();
        assert_eq!(mnemonic.split_whitespace().count(), 12);
    }

    #[test]
    fn same_mnemonic_same_keypair() {
        let mnemonic = generate_mnemonic();
        let first = mnemonic_to_pair(&mnemonic).expect("generated mnemonic is valid");
        let second = mnemonic_to_pair(&mnemonic).expect("generated mnemonic is valid");
        assert_eq!(first.public(), second.public());
    }

    #[test]
    fn address_is_valid_ss58() {
        let mnemonic = generate_mnemonic();
        let address = mnemonic_to_address(&mnemonic).expect("generated mnemonic is valid");
        let public =
            sr25519::Public::from_ss58check(&address).expect("address should decode as ss58");
        let pair = mnemonic_to_pair(&mnemonic).expect("generated mnemonic is valid");
        assert_eq!(public, pair.public());
        assert_eq!(address, pair_to_address(&pair));
    }

    #[test]
    fn config_and_keys_dirs_use_myozu_home_layout() {
        let home = Path::new("/tmp/test-home");
        assert_eq!(config_dir_from_home(home), home.join(".myosu"));
        assert_eq!(keys_dir_from_home(home), home.join(".myosu").join("keys"));
    }

    #[test]
    fn default_keys_dir_uses_home_environment() {
        let Some(home) = std::env::var_os("HOME") else {
            panic!("HOME should be present in the test environment");
        };
        assert_eq!(
            default_keys_dir().expect("HOME should be available"),
            Path::new(&home).join(".myosu").join("keys"),
        );
    }
}
