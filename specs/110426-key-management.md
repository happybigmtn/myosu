# Specification: Operator Key Management

## Objective

Define the current state and intended direction of the operator key lifecycle managed by `myosu-keys`: key generation, encrypted storage, loading, and the sr25519 derivation surface used by miner, validator, and play binaries.

## Evidence Status

### Verified facts (code-grounded)

- Crate: `myosu-keys` at `crates/myosu-keys/` — `Cargo.toml` workspace member
- `KeyError` enum has 15+ variants covering: InvalidMnemonic, MissingHomeDir, MissingPasswordEnv, MissingKeySource, CreateDirectory, InvalidPath, ReadFile, WriteFile, InvalidConfig, SerializeKeyfile, DeserializeKeyfile, InvalidKeyfile, MissingKeyfile, InvalidKeyfileHex, KeyDerivation, EncryptSeed, DecryptSeed, InvalidSeedMaterial, SecretUriUnsupported — `crates/myosu-keys/src/lib.rs:22-91`
- Public API: `generate_mnemonic()`, `mnemonic_to_pair()`, `load_active_pair()`, `save_pair()` — `crates/myosu-keys/src/lib.rs`
- Storage module exports: `ListedOperatorAccount`, `LoadedOperatorAccount`, `OperatorConfig`, `StoredOperatorAccount` — `crates/myosu-keys/src/storage.rs`
- Storage functions: `config_file_from_home()`, `default_config_file()`, `export_active_keyfile()`, `import_keyfile()`, `key_file_path()`, `list_stored_accounts()`, `load_active_pair()`, `load_active_secret_uri()`, `load_active_secret_uri_from_env()`, `load_operator_config()`, `load_pair_from_keyfile()`, `save_mnemonic()`, `save_pair()`, `set_active_account()` — `crates/myosu-keys/src/storage.rs`
- Encryption algorithm: crypto_secretbox (NaCl-style, XSalsa20-Poly1305) — `crates/myosu-keys/`
- Key derivation: scrypt (password -> encryption key), using `Params::recommended()` when creating a keyfile and persisting the chosen scrypt parameters in the keyfile JSON — `crates/myosu-keys/src/storage.rs:347-374`
- Keyfile format: JSON with hex-encoded encrypted fields — `crates/myosu-keys/`
- Storage locations: config at `~/.myosu/config.toml`, keys at `~/.myosu/keys/{address}.json` — `crates/myosu-keys/src/storage.rs:18-72`
- Substrate key type: sr25519 — `crates/myosu-keys/src/lib.rs`
- Password sourced from environment variable (default `MYOSU_KEY_PASSWORD`) — miner/validator CLI
- Miner and validator both accept `--key` (direct file) or `--key-config-dir` (config directory) — miner/validator CLI definitions
- `create` prints the generated mnemonic once and explicitly says it is not stored on disk — `crates/myosu-keys/src/main.rs:305-320`, `crates/myosu-keys/src/main.rs:512-518`
- `export_active_keyfile()` copies the active encrypted Myosu keyfile JSON to a destination path; it does not decrypt or export plaintext seed material — `crates/myosu-keys/src/storage.rs:264-279`

### Recommendations (intended system)

- Key rotation procedure should be documented for operators
- Mnemonic backup guidance should be documented because generated mnemonics are displayed once and are not persisted by the keystore

### Hypotheses / unresolved questions

- Whether hardware security module (HSM) support is planned
- Whether multi-key setups (different keys for different subnets) are supported by the config model
- Whether the `Params::recommended()` scrypt defaults should be pinned in policy, exposed in docs, or made configurable

## Acceptance Criteria

- `generate_mnemonic()` produces a valid BIP-39 mnemonic
- `mnemonic_to_pair()` derives a valid sr25519 keypair from a mnemonic
- `save_pair()` encrypts the seed with crypto_secretbox and writes JSON to `~/.myosu/keys/{name}.json`
- `load_active_pair()` decrypts and returns the sr25519 pair using password from environment
- `load_pair_from_keyfile()` loads and decrypts from an explicit path
- `set_active_account()` updates `~/.myosu/config.toml` to point at the active operator
- `list_stored_accounts()` enumerates all accounts in the keys directory
- `config_file_from_home()` and `default_config_file()` resolve to `~/.myosu/config.toml`
- `KeyError` variants provide actionable error messages (path, operation, suggested fix)
- `create` communicates that the displayed mnemonic must be backed up immediately because it is not stored on disk
- Miner/validator `--key` flag loads from explicit file path without needing config directory
- Miner/validator `--key-config-dir` flag loads via config resolution

## Verification

```bash
# Unit tests
SKIP_WASM_BUILD=1 cargo test -p myosu-keys --quiet

# Compile check
SKIP_WASM_BUILD=1 cargo check -p myosu-keys

# Clippy
SKIP_WASM_BUILD=1 cargo clippy -p myosu-keys -- -D warnings
```

## Open Questions

1. **Mnemonic backup flow:** Generated mnemonics are printed once and not stored on disk. Is the current terminal note enough, or should creation require an explicit backup confirmation or verification step?
2. **Scrypt parameters:** New keyfiles use `Params::recommended()` and persist N/r/p in JSON. Are those defaults sufficient for modern attack resistance, and should policy pin exact parameters?
3. **Key rotation:** There is no documented key rotation procedure. If an operator's key is compromised, what's the recovery path? Is re-registration required?
4. **Multi-subnet operation:** Can an operator use different keys for different subnets? The config model has `active_account` suggesting a single active key.
5. **`export_active_keyfile()` safety:** This function copies encrypted keyfiles. Should exported encrypted keyfiles still require an operator warning because they are sensitive if the password is weak or reused?
6. **Secret URI support:** `SecretUriUnsupported` error variant exists. Does this mean Substrate `//` derivation paths are intentionally unsupported?
