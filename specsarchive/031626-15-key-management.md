# Specification: Key Management — Account Lifecycle for Players

Source: DESIGN.md 8.0a-8.0d onboarding and wallet flows
Status: Draft
Date: 2026-03-17
Depends-on: CF-01..11 (chain scaffold — sr25519 account model)
Blocks: TU-05 Onboarding screen, TU-05 Wallet screen

## Purpose

Define how players create, store, import, export, and switch accounts.
DESIGN.md specifies the full onboarding flow (seed phrase → verification →
network selection) and wallet operations (/export, /switch, /fund). This
spec covers the cryptographic and storage mechanics behind those screens.

The gameplay CLI (`myosu-play`) needs a keypair to:
1. Sign transactions (staking, subnet registration if player becomes miner)
2. Identify the player on-chain (address displayed in wallet)
3. Authenticate to miners (future: signed strategy queries)

For Phase 0, key management is local-only. No hardware wallets, no browser
extension integration, no remote key storage.

## Scope

In scope:
- BIP-39 mnemonic generation (12/24 words) for new accounts
- Mnemonic → sr25519 keypair derivation (Substrate-compatible)
- Key file storage at `~/.myosu/keys/`
- Key import from mnemonic, raw seed, or JSON keystore
- Key export to mnemonic display or JSON keystore file
- Active key selection and switching
- Config file for network selection and active account

Out of scope:
- Hardware wallet support (Ledger, etc.)
- Browser extension integration (polkadot.js, etc.)
- Multi-signature accounts
- Key rotation or migration
- Encrypted transport of keys between devices

---

### AC-KM-01: Mnemonic Generation and Key Derivation

- Where: `crates/myosu-keys/src/lib.rs (new crate)`

  **Crate placement**: key management is a shared library, not part of
  `myosu-play`. Both `myosu-play` (gameplay binary) and `myosu-tui`
  (onboarding screen) need key functions. Placing in `myosu-play` would
  create a circular dependency. `myosu-keys` is a pure library crate
  with no binary or TUI dependencies.
- How: Use `sp-core` and `bip39` crates (already in Substrate dependency tree):

  ```rust
  use sp_core::{sr25519, Pair, crypto::Ss58Codec};
  use bip39::{Mnemonic, Language};

  pub fn generate_mnemonic() -> Mnemonic {
      Mnemonic::generate_in(Language::English, 12)
  }

  pub fn mnemonic_to_keypair(mnemonic: &Mnemonic) -> Result<sr25519::Pair> {
      let (pair, _seed) = sr25519::Pair::from_phrase(mnemonic.phrase(), None)
          .map_err(|e| anyhow!("invalid mnemonic: {e}"))?;
      Ok(pair)
  }

  pub fn keypair_to_address(pair: &sr25519::Pair) -> String {
      pair.public().to_ss58check()
  }
  ```

  12-word mnemonic by default (128-bit entropy). 24-word optional for
  users who request it via `create new account (24-word)`.

  Derivation uses Substrate's standard sr25519 from BIP-39 with no
  password. This is compatible with polkadot.js, Subkey, and all
  Substrate wallets.

- Required tests:
  - `keys::tests::generate_mnemonic_is_12_words`
  - `keys::tests::mnemonic_to_keypair_deterministic`
  - `keys::tests::address_is_valid_ss58`
  - `keys::tests::same_mnemonic_same_keypair`
- Pass/fail:
  - Generated mnemonic has 12 space-separated words from BIP-39 English wordlist
  - Same mnemonic always produces same sr25519 keypair
  - Address starts with valid ss58 prefix (5 for generic Substrate)
  - Different mnemonics produce different keypairs

### AC-KM-02: Key Storage

- Where: `crates/myosu-keys/src/keystore.rs (new)`
- How: Store keys in `~/.myosu/keys/` directory:

  ```
  ~/.myosu/
  ├── config.toml          # active account, network selection
  └── keys/
      ├── 5GrwvaEF.json    # JSON keystore (address as filename)
      └── 5FHneW46.json
  ```

  JSON keystore format (Substrate-compatible):
  ```json
  {
    "address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "encoded": "<scrypt-encrypted-seed>",
    "encoding": { "content": ["pkcs8", "sr25519"], "type": ["scrypt", "xsalsa20-poly1305"] },
    "meta": { "name": "Alice", "created": "2026-03-17T00:00:00Z" }
  }
  ```

  Keys are encrypted at rest with a user-provided password. The password
  is requested on first use per session and cached in memory (never on disk).

  Config file (`~/.myosu/config.toml`):
  ```toml
  [account]
  active = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
  name = "Alice"

  [network]
  endpoint = "ws://localhost:9944"
  chain = "devnet"
  ```

- Required tests:
  - `keystore::tests::save_and_load_key`
  - `keystore::tests::encrypted_key_requires_password`
  - `keystore::tests::wrong_password_fails`
  - `keystore::tests::list_stored_keys`
- Pass/fail:
  - Key saved to `~/.myosu/keys/<address>.json`
  - Loading with correct password returns keypair
  - Loading with wrong password returns error (not panic, not garbage)
  - `list_keys()` returns all stored addresses with names

### AC-KM-03: Key Import

- Where: `crates/myosu-keys/src/keystore.rs (extend)`
- How: Three import methods matching DESIGN.md 8.0a options:

  1. **Import from mnemonic** (option 3: "recover from seed phrase"):
     User types 12 or 24 words. Derive keypair. Save to keystore.

  2. **Import from raw seed** (advanced):
     User provides hex-encoded 32-byte seed. Derive sr25519 keypair.

  3. **Import from JSON keystore** (option 2: "import existing key"):
     User provides path to a Substrate-compatible JSON keystore file.
     Copy to `~/.myosu/keys/`.

  Validation: after import, display the derived address and ask for
  confirmation before saving.

- Required tests:
  - `keystore::tests::import_from_mnemonic`
  - `keystore::tests::import_from_raw_seed`
  - `keystore::tests::import_from_json_file`
  - `keystore::tests::import_invalid_mnemonic_fails`
- Pass/fail:
  - Valid 12-word mnemonic imports and produces correct address
  - Invalid mnemonic (wrong words, wrong count) returns clear error
  - JSON keystore import preserves address and name metadata
  - After import, key is usable for signing

### AC-KM-04: Key Export and Account Switching

- Where: `crates/myosu-keys/src/keystore.rs (extend)`
- How: Two export methods matching DESIGN.md 8.0d wallet:

  `/export` command: export active key to JSON keystore file at a
  user-specified path. Requires password re-entry for confirmation.

  `/switch` command: list all stored keys, let user select by number
  or address prefix. Update `config.toml` active account.

  ```
  MYOSU / WALLET / SWITCH

  SELECT ACCOUNT

    #  address          name      balance
    1  5GrwvaE...       Alice     1000 MYOSU
    2  5FHneW4...       Bob       0 MYOSU

  > 2
  ```

  No mnemonic display after initial creation — mnemonics are never stored,
  only the encrypted seed. If the user needs their mnemonic again, they
  must have written it down during onboarding.

- Required tests:
  - `keystore::tests::export_to_json_file`
  - `keystore::tests::switch_active_account`
  - `keystore::tests::export_requires_password`
  - `keystore::tests::switch_nonexistent_account_errors`
- Pass/fail:
  - Exported JSON file is importable by polkadot.js and Subkey
  - `/switch 2` updates config.toml active account
  - Export without correct password is refused

## Onboarding flow mapping

| DESIGN.md screen | KM AC | What happens |
|-----------------|-------|--------------|
| 8.0a Welcome: "create new account" | KM-01 | Generate mnemonic, derive keypair |
| 8.0a Welcome: "import existing key" | KM-03 | Import from JSON keystore |
| 8.0a Welcome: "recover from seed phrase" | KM-03 | Import from mnemonic |
| 8.0b Seed Backup | (display only) | Show mnemonic, user writes it down |
| 8.0c Verify | (display only) | Prompt 3 random words, verify match |
| 8.0c Network | KM-02 | Save key + config with selected network |
| 8.0d Wallet: /export | KM-04 | Export to file |
| 8.0d Wallet: /switch | KM-04 | Switch active account |

## Security considerations

- Mnemonics are generated in memory, displayed once, never written to disk
- Seeds are encrypted at rest with scrypt + xsalsa20-poly1305
- Password is held in memory only for the duration of the session
- On `--pipe` mode, password prompt goes to stderr (not mixed with game output)
- Key files have 0600 permissions (owner-read-only)
- No key material in logs, crash dumps, or error messages
