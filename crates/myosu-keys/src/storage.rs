use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crypto_secretbox::{
    Nonce, XSalsa20Poly1305,
    aead::{Aead, AeadCore, KeyInit, OsRng, rand_core::RngCore},
};
use scrypt::{Params, scrypt};
use serde::{Deserialize, Serialize};
use sp_core::{Pair, sr25519};
use toml_edit::{DocumentMut, Item, Value};

use crate::{KeyError, config_dir_from_home, default_config_dir, pair_to_address};

const CONFIG_FILE_NAME: &str = "config.toml";
const KEYFILE_VERSION: u32 = 1;
const CONFIG_VERSION: i64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorConfig {
    pub active_account: String,
    pub key_file: String,
    pub network: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredOperatorAccount {
    pub address: String,
    pub config_path: PathBuf,
    pub key_path: PathBuf,
    pub network: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedOperatorAccount {
    pub address: String,
    pub key_path: PathBuf,
}

pub struct LoadedOperatorAccount {
    pub config: OperatorConfig,
    pub key_path: PathBuf,
    pub pair: sr25519::Pair,
}

#[derive(Debug, Deserialize, Serialize)]
struct StoredKeyfile {
    version: u32,
    address: String,
    cipher: String,
    kdf: String,
    scrypt_log_n: u8,
    scrypt_r: u32,
    scrypt_p: u32,
    salt_hex: String,
    nonce_hex: String,
    ciphertext_hex: String,
}

pub fn config_file_from_home(home: &Path) -> PathBuf {
    config_dir_from_home(home).join(CONFIG_FILE_NAME)
}

pub fn default_config_file() -> Result<PathBuf, KeyError> {
    Ok(default_config_dir()?.join(CONFIG_FILE_NAME))
}

pub fn key_file_path(keys_dir: &Path, address: &str) -> PathBuf {
    keys_dir.join(format!("{address}.json"))
}

pub fn save_mnemonic(
    config_dir: &Path,
    mnemonic: &str,
    password: &str,
    network: &str,
) -> Result<StoredOperatorAccount, KeyError> {
    let (pair, seed) = sr25519::Pair::from_phrase(mnemonic, None)
        .map_err(|error| KeyError::InvalidMnemonic(format!("{error:?}")))?;
    save_key_material(config_dir, &pair, seed.as_ref(), password, network)
}

pub fn save_pair(
    config_dir: &Path,
    pair: &sr25519::Pair,
    password: &str,
    network: &str,
) -> Result<StoredOperatorAccount, KeyError> {
    save_key_material(config_dir, pair, &pair.to_raw_vec(), password, network)
}

pub fn load_active_secret_uri(config_dir: &Path, password: &str) -> Result<String, KeyError> {
    let config = load_operator_config(config_dir)?;
    let key_path = key_file_path(
        &keys_dir_from_config_dir(config_dir),
        &config.active_account,
    );
    let key_bytes = load_key_material_from_keyfile(&key_path, password)?;
    secret_uri_from_key_material(&key_path, &key_bytes)
}

pub fn load_active_secret_uri_from_env(
    config_dir: &Path,
    password_env: &str,
) -> Result<String, KeyError> {
    let password = std::env::var(password_env).map_err(|_| KeyError::MissingPasswordEnv {
        env: password_env.to_owned(),
    })?;
    load_active_secret_uri(config_dir, &password)
}

fn save_key_material(
    config_dir: &Path,
    pair: &sr25519::Pair,
    key_material: &[u8],
    password: &str,
    network: &str,
) -> Result<StoredOperatorAccount, KeyError> {
    let address = pair_to_address(pair);
    let keys_dir = keys_dir_from_config_dir(config_dir);
    create_directory(config_dir)?;
    create_directory(&keys_dir)?;

    let key_path = key_file_path(&keys_dir, &address);
    let keyfile = encrypt_keyfile(key_material, password, &address)?;
    write_keyfile(&key_path, &keyfile)?;

    let config = OperatorConfig {
        active_account: address.clone(),
        key_file: key_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| KeyError::InvalidPath(key_path.clone()))?,
        network: network.to_owned(),
    };
    let config_path = config_dir.join(CONFIG_FILE_NAME);
    write_operator_config(&config_path, &config)?;

    Ok(StoredOperatorAccount {
        address,
        config_path,
        key_path,
        network: network.to_owned(),
    })
}

pub fn load_operator_config(config_dir: &Path) -> Result<OperatorConfig, KeyError> {
    let config_path = config_dir.join(CONFIG_FILE_NAME);
    let config_text = fs::read_to_string(&config_path).map_err(|source| KeyError::ReadFile {
        path: config_path.clone(),
        source,
    })?;
    parse_operator_config(&config_path, &config_text)
}

pub fn list_stored_accounts(config_dir: &Path) -> Result<Vec<ListedOperatorAccount>, KeyError> {
    let keys_dir = keys_dir_from_config_dir(config_dir);
    let entries = match fs::read_dir(&keys_dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(KeyError::ReadFile {
                path: keys_dir,
                source,
            });
        }
    };

    let mut accounts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| KeyError::ReadFile {
            path: keys_dir_from_config_dir(config_dir),
            source,
        })?;
        let key_path = entry.path();
        if !key_path.is_file() {
            continue;
        }
        let Some(file_stem) = key_path.file_stem() else {
            continue;
        };
        accounts.push(ListedOperatorAccount {
            address: file_stem.to_string_lossy().into_owned(),
            key_path,
        });
    }
    accounts.sort_by(|left, right| left.address.cmp(&right.address));
    Ok(accounts)
}

pub fn set_active_account(config_dir: &Path, address: &str) -> Result<OperatorConfig, KeyError> {
    let mut config = load_operator_config(config_dir)?;
    let key_path = key_file_path(&keys_dir_from_config_dir(config_dir), address);
    if !key_path.exists() {
        return Err(KeyError::MissingKeyfile { path: key_path });
    }

    config.active_account = address.to_owned();
    config.key_file = key_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| KeyError::InvalidPath(key_path.clone()))?;

    let config_path = config_dir.join(CONFIG_FILE_NAME);
    write_operator_config(&config_path, &config)?;
    Ok(config)
}

pub fn import_keyfile(
    config_dir: &Path,
    source_path: &Path,
    network: &str,
) -> Result<StoredOperatorAccount, KeyError> {
    let key_text = fs::read_to_string(source_path).map_err(|source| KeyError::ReadFile {
        path: source_path.to_path_buf(),
        source,
    })?;
    let keyfile = serde_json::from_str::<StoredKeyfile>(&key_text).map_err(|source| {
        KeyError::DeserializeKeyfile {
            path: source_path.to_path_buf(),
            source,
        }
    })?;
    validate_keyfile(source_path, &keyfile)?;

    let config = load_operator_config(config_dir);
    let network = match config {
        Ok(config) => config.network,
        Err(KeyError::ReadFile { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            network.to_owned()
        }
        Err(error) => return Err(error),
    };

    let keys_dir = keys_dir_from_config_dir(config_dir);
    create_directory(config_dir)?;
    create_directory(&keys_dir)?;

    let key_path = key_file_path(&keys_dir, &keyfile.address);
    write_private_file(&key_path, key_text.as_bytes())?;

    let config = OperatorConfig {
        active_account: keyfile.address.clone(),
        key_file: key_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| KeyError::InvalidPath(key_path.clone()))?,
        network: network.clone(),
    };
    let config_path = config_dir.join(CONFIG_FILE_NAME);
    write_operator_config(&config_path, &config)?;

    Ok(StoredOperatorAccount {
        address: keyfile.address,
        config_path,
        key_path,
        network,
    })
}

pub fn export_active_keyfile(config_dir: &Path, output_path: &Path) -> Result<PathBuf, KeyError> {
    let config = load_operator_config(config_dir)?;
    let key_path = key_file_path(
        &keys_dir_from_config_dir(config_dir),
        &config.active_account,
    );
    if !key_path.exists() {
        return Err(KeyError::MissingKeyfile { path: key_path });
    }

    let key_text = fs::read_to_string(&key_path).map_err(|source| KeyError::ReadFile {
        path: key_path.clone(),
        source,
    })?;
    write_private_file(output_path, key_text.as_bytes())?;
    Ok(output_path.to_path_buf())
}

pub fn load_active_pair(
    config_dir: &Path,
    password: &str,
) -> Result<LoadedOperatorAccount, KeyError> {
    let config = load_operator_config(config_dir)?;
    let key_path = key_file_path(
        &keys_dir_from_config_dir(config_dir),
        &config.active_account,
    );
    let pair = load_pair_from_keyfile(&key_path, password)?;

    let expected_name = key_file_path(Path::new("keys"), &config.active_account)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| KeyError::InvalidPath(key_path.clone()))?;
    if config.key_file != expected_name {
        return Err(KeyError::InvalidConfig {
            path: config_dir.join(CONFIG_FILE_NAME),
            reason: format!(
                "active_account {} expects key_file {expected_name}, found {}",
                config.active_account, config.key_file
            ),
        });
    }

    Ok(LoadedOperatorAccount {
        config,
        key_path,
        pair,
    })
}

pub fn load_pair_from_keyfile(key_path: &Path, password: &str) -> Result<sr25519::Pair, KeyError> {
    let key_bytes = load_key_material_from_keyfile(key_path, password)?;
    sr25519::Pair::from_seed_slice(&key_bytes).map_err(|source| KeyError::InvalidSeedMaterial {
        path: key_path.to_path_buf(),
        reason: format!("{source:?}"),
    })
}

fn load_key_material_from_keyfile(key_path: &Path, password: &str) -> Result<Vec<u8>, KeyError> {
    let key_text = fs::read_to_string(key_path).map_err(|source| KeyError::ReadFile {
        path: key_path.to_path_buf(),
        source,
    })?;
    let keyfile = serde_json::from_str::<StoredKeyfile>(&key_text).map_err(|source| {
        KeyError::DeserializeKeyfile {
            path: key_path.to_path_buf(),
            source,
        }
    })?;
    decrypt_keyfile(&keyfile, key_path, password)
}

fn keys_dir_from_config_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("keys")
}

fn create_directory(path: &Path) -> Result<(), KeyError> {
    fs::create_dir_all(path).map_err(|source| KeyError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn encrypt_keyfile(
    key_material: &[u8],
    password: &str,
    address: &str,
) -> Result<StoredKeyfile, KeyError> {
    let params = Params::recommended();
    let mut salt = [0_u8; 32];
    OsRng.fill_bytes(&mut salt);

    let key = derive_encryption_key(password, &salt, params)?;
    let cipher = XSalsa20Poly1305::new((&key).into());
    let nonce = XSalsa20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, key_material)
        .map_err(|_| KeyError::EncryptSeed)?;

    Ok(StoredKeyfile {
        version: KEYFILE_VERSION,
        address: address.to_owned(),
        cipher: "xsalsa20poly1305".to_owned(),
        kdf: "scrypt".to_owned(),
        scrypt_log_n: params.log_n(),
        scrypt_r: params.r(),
        scrypt_p: params.p(),
        salt_hex: hex::encode(salt),
        nonce_hex: hex::encode(nonce),
        ciphertext_hex: hex::encode(ciphertext),
    })
}

fn derive_encryption_key(
    password: &str,
    salt: &[u8],
    params: Params,
) -> Result<[u8; 32], KeyError> {
    let mut key = [0_u8; 32];
    scrypt(password.as_bytes(), salt, &params, &mut key).map_err(|_| KeyError::KeyDerivation)?;
    Ok(key)
}

fn write_keyfile(path: &Path, keyfile: &StoredKeyfile) -> Result<(), KeyError> {
    let key_text =
        serde_json::to_string_pretty(keyfile).map_err(|source| KeyError::SerializeKeyfile {
            path: path.to_path_buf(),
            source,
        })?;
    write_private_file(path, key_text.as_bytes())
}

fn write_operator_config(path: &Path, config: &OperatorConfig) -> Result<(), KeyError> {
    let mut doc = DocumentMut::new();
    let table = doc.as_table_mut();
    table.insert("version", Item::Value(Value::from(CONFIG_VERSION)));
    table.insert("network", Item::Value(Value::from(config.network.as_str())));
    table.insert(
        "active_account",
        Item::Value(Value::from(config.active_account.as_str())),
    );
    table.insert(
        "key_file",
        Item::Value(Value::from(config.key_file.as_str())),
    );
    write_private_file(path, doc.to_string().as_bytes())
}

fn parse_operator_config(path: &Path, text: &str) -> Result<OperatorConfig, KeyError> {
    let doc = DocumentMut::from_str(text).map_err(|source| KeyError::InvalidConfig {
        path: path.to_path_buf(),
        reason: source.to_string(),
    })?;

    let version = read_config_integer(path, &doc, "version")?;
    if version != CONFIG_VERSION {
        return Err(KeyError::InvalidConfig {
            path: path.to_path_buf(),
            reason: format!("unsupported config version {version}; expected {CONFIG_VERSION}"),
        });
    }

    Ok(OperatorConfig {
        active_account: read_config_string(path, &doc, "active_account")?,
        key_file: read_config_string(path, &doc, "key_file")?,
        network: read_config_string(path, &doc, "network")?,
    })
}

fn read_config_integer(path: &Path, doc: &DocumentMut, field: &str) -> Result<i64, KeyError> {
    doc[field]
        .as_integer()
        .ok_or_else(|| KeyError::InvalidConfig {
            path: path.to_path_buf(),
            reason: format!("missing integer field {field}"),
        })
}

fn read_config_string(path: &Path, doc: &DocumentMut, field: &str) -> Result<String, KeyError> {
    doc[field]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| KeyError::InvalidConfig {
            path: path.to_path_buf(),
            reason: format!("missing string field {field}"),
        })
}

fn write_private_file(path: &Path, contents: &[u8]) -> Result<(), KeyError> {
    fs::write(path, contents).map_err(|source| KeyError::WriteFile {
        path: path.to_path_buf(),
        source,
    })?;
    set_private_permissions(path)
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), KeyError> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions).map_err(|source| KeyError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<(), KeyError> {
    Ok(())
}

fn decrypt_keyfile(
    keyfile: &StoredKeyfile,
    path: &Path,
    password: &str,
) -> Result<Vec<u8>, KeyError> {
    validate_keyfile(path, keyfile)?;

    let params = Params::new(keyfile.scrypt_log_n, keyfile.scrypt_r, keyfile.scrypt_p, 32)
        .map_err(|_| KeyError::InvalidKeyfile {
            path: path.to_path_buf(),
            reason: "invalid scrypt params".to_owned(),
        })?;
    let salt = decode_key_bytes(path, "salt_hex", &keyfile.salt_hex)?;
    let nonce_bytes = decode_nonce(path, &keyfile.nonce_hex)?;
    let ciphertext = decode_key_bytes(path, "ciphertext_hex", &keyfile.ciphertext_hex)?;
    let nonce = Nonce::from(nonce_bytes);
    let key = derive_encryption_key(password, &salt, params)?;
    let cipher = XSalsa20Poly1305::new((&key).into());
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_| KeyError::DecryptSeed)?;
    let pair = sr25519::Pair::from_seed_slice(&plaintext).map_err(|source| {
        KeyError::InvalidSeedMaterial {
            path: path.to_path_buf(),
            reason: format!("{source:?}"),
        }
    })?;

    let loaded_address = pair_to_address(&pair);
    if loaded_address != keyfile.address {
        return Err(KeyError::InvalidKeyfile {
            path: path.to_path_buf(),
            reason: format!(
                "stored address {} does not match decrypted key {loaded_address}",
                keyfile.address
            ),
        });
    }

    Ok(plaintext)
}

fn validate_keyfile(path: &Path, keyfile: &StoredKeyfile) -> Result<(), KeyError> {
    if keyfile.version != KEYFILE_VERSION {
        return Err(KeyError::InvalidKeyfile {
            path: path.to_path_buf(),
            reason: format!(
                "unsupported keyfile version {}; expected {}",
                keyfile.version, KEYFILE_VERSION
            ),
        });
    }
    if keyfile.cipher != "xsalsa20poly1305" {
        return Err(KeyError::InvalidKeyfile {
            path: path.to_path_buf(),
            reason: format!("unsupported cipher {}", keyfile.cipher),
        });
    }
    if keyfile.kdf != "scrypt" {
        return Err(KeyError::InvalidKeyfile {
            path: path.to_path_buf(),
            reason: format!("unsupported kdf {}", keyfile.kdf),
        });
    }
    Ok(())
}

fn decode_key_bytes(path: &Path, field: &str, value: &str) -> Result<Vec<u8>, KeyError> {
    hex::decode(value).map_err(|source| KeyError::InvalidKeyfileHex {
        path: path.to_path_buf(),
        field: field.to_owned(),
        source,
    })
}

fn decode_nonce(path: &Path, value: &str) -> Result<[u8; 24], KeyError> {
    let nonce = decode_key_bytes(path, "nonce_hex", value)?;
    nonce.try_into().map_err(|_| KeyError::InvalidKeyfile {
        path: path.to_path_buf(),
        reason: "nonce_hex must decode to 24 bytes".to_owned(),
    })
}

fn secret_uri_from_key_material(path: &Path, key_material: &[u8]) -> Result<String, KeyError> {
    match key_material.len() {
        32 => Ok(format!("0x{}", hex::encode(key_material))),
        length => Err(KeyError::SecretUriUnsupported {
            path: path.to_path_buf(),
            reason: format!("expected 32-byte seed material, found {length} bytes"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        export_active_keyfile, import_keyfile, key_file_path, list_stored_accounts,
        load_active_pair, load_operator_config, save_mnemonic, set_active_account,
    };
    use crate::{generate_mnemonic, mnemonic_to_pair, pair_to_address};
    use sp_core::{Pair, sr25519};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-keys-{label}-{nanos}"))
    }

    #[test]
    fn save_and_load_round_trips_active_pair() {
        let root = temp_root("roundtrip");
        let mnemonic = generate_mnemonic();
        let original = mnemonic_to_pair(&mnemonic).expect("generated mnemonic is valid");

        let stored = save_mnemonic(&root, &mnemonic, "correct horse battery staple", "devnet")
            .expect("save should succeed");
        let loaded =
            load_active_pair(&root, "correct horse battery staple").expect("load should succeed");

        assert_eq!(stored.address, pair_to_address(&original));
        assert_eq!(loaded.config.network, "devnet");
        assert_eq!(loaded.config.active_account, stored.address);
        assert_eq!(loaded.pair.public(), original.public());
        assert_eq!(loaded.key_path, stored.key_path);

        fs::remove_dir_all(&root).expect("temp root should clean");
    }

    #[test]
    fn save_writes_expected_config_and_key_paths() {
        let root = temp_root("paths");
        let mnemonic = generate_mnemonic();
        let stored =
            save_mnemonic(&root, &mnemonic, "hunter2", "test_finney").expect("save should succeed");
        let config = load_operator_config(&root).expect("config should parse");

        assert_eq!(stored.config_path, root.join("config.toml"));
        assert_eq!(config.network, "test_finney");
        assert_eq!(config.active_account, stored.address);
        assert_eq!(
            stored.key_path,
            key_file_path(&root.join("keys"), &stored.address)
        );
        assert_eq!(config.key_file, format!("{}.json", stored.address));

        fs::remove_dir_all(&root).expect("temp root should clean");
    }

    #[test]
    fn wrong_password_is_rejected() {
        let root = temp_root("wrong-password");
        let mnemonic = generate_mnemonic();
        save_mnemonic(&root, &mnemonic, "right-password", "devnet").expect("save should succeed");

        let error = match load_active_pair(&root, "wrong-password") {
            Ok(_) => panic!("wrong password should fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("failed to decrypt"));

        fs::remove_dir_all(&root).expect("temp root should clean");
    }

    #[test]
    fn mnemonic_keystore_loads_as_standard_secret_uri() {
        let root = temp_root("secret-uri");
        let mnemonic = generate_mnemonic();
        let original = mnemonic_to_pair(&mnemonic).expect("generated mnemonic is valid");
        save_mnemonic(&root, &mnemonic, "hunter2", "devnet").expect("save should succeed");

        let uri = super::load_active_secret_uri(&root, "hunter2").expect("secret uri should load");
        let repaired =
            sr25519::Pair::from_string(&uri, None).expect("loaded secret uri should parse");

        assert_eq!(repaired.public(), original.public());

        fs::remove_dir_all(&root).expect("temp root should clean");
    }

    #[test]
    fn list_stored_accounts_returns_sorted_addresses() {
        let root = temp_root("list");
        let first = save_mnemonic(&root, &generate_mnemonic(), "hunter2", "devnet")
            .expect("first save should succeed");
        let second = save_mnemonic(&root, &generate_mnemonic(), "hunter2", "devnet")
            .expect("second save should succeed");

        let listed = list_stored_accounts(&root).expect("list should succeed");

        assert_eq!(listed.len(), 2);
        assert!(listed[0].address < listed[1].address);
        assert!(
            listed
                .iter()
                .any(|account| account.address == first.address)
        );
        assert!(
            listed
                .iter()
                .any(|account| account.address == second.address)
        );

        fs::remove_dir_all(&root).expect("temp root should clean");
    }

    #[test]
    fn set_active_account_updates_config() {
        let root = temp_root("switch");
        let first = save_mnemonic(&root, &generate_mnemonic(), "hunter2", "devnet")
            .expect("first save should succeed");
        let second = save_mnemonic(&root, &generate_mnemonic(), "hunter2", "devnet")
            .expect("second save should succeed");

        let updated =
            set_active_account(&root, &first.address).expect("switching active account works");
        let config = load_operator_config(&root).expect("config should parse");

        assert_eq!(updated.active_account, first.address);
        assert_eq!(config.active_account, first.address);
        assert_eq!(config.key_file, format!("{}.json", first.address));
        assert_ne!(config.active_account, second.address);

        fs::remove_dir_all(&root).expect("temp root should clean");
    }

    #[test]
    fn import_keyfile_copies_key_and_sets_active_account() {
        let source_root = temp_root("import-source");
        let imported_root = temp_root("import-dest");
        let source = save_mnemonic(&source_root, &generate_mnemonic(), "hunter2", "devnet")
            .expect("source key save should succeed");

        let imported =
            import_keyfile(&imported_root, &source.key_path, "test_finney").expect("import works");
        let config = load_operator_config(&imported_root).expect("config should parse");

        assert_eq!(imported.address, source.address);
        assert_eq!(config.active_account, source.address);
        assert_eq!(config.network, "test_finney");
        assert!(imported.key_path.exists());

        fs::remove_dir_all(&source_root).expect("source root should clean");
        fs::remove_dir_all(&imported_root).expect("imported root should clean");
    }

    #[test]
    fn export_active_keyfile_writes_copy() {
        let root = temp_root("export");
        let stored = save_mnemonic(&root, &generate_mnemonic(), "hunter2", "devnet")
            .expect("save should succeed");
        let export_path = root.join("exported.json");

        let exported = export_active_keyfile(&root, &export_path).expect("export works");
        let original = fs::read_to_string(&stored.key_path).expect("original key should exist");
        let copied = fs::read_to_string(&exported).expect("exported key should exist");

        assert_eq!(exported, export_path);
        assert_eq!(original, copied);

        fs::remove_dir_all(&root).expect("temp root should clean");
    }

    #[cfg(unix)]
    #[test]
    fn save_sets_private_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_root("permissions");
        let mnemonic = generate_mnemonic();
        let stored =
            save_mnemonic(&root, &mnemonic, "hunter2", "devnet").expect("save should succeed");

        let config_mode = fs::metadata(&stored.config_path)
            .expect("config metadata should exist")
            .permissions();
        let key_mode = fs::metadata(&stored.key_path)
            .expect("key metadata should exist")
            .permissions();

        assert_eq!(config_mode.mode() & 0o777, 0o600);
        assert_eq!(key_mode.mode() & 0o777, 0o600);

        fs::remove_dir_all(&root).expect("temp root should clean");
    }
}
