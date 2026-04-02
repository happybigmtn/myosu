# Specification: Key Management

Source: Reverse-engineered from crates/myosu-keys (main.rs, lib.rs, storage.rs)
Status: Draft
Depends-on: none

## Purpose

The key management CLI handles the lifecycle of encrypted operator keypairs used
by miners and validators to sign on-chain transactions. It provides key
generation, import from multiple sources, encrypted persistence, export, active
account switching, password rotation, and bootstrap command generation. Without
this component, operators would need to manage raw secret keys directly,
increasing the risk of key exposure.

The primary consumer is a miner or validator operator setting up their signing
identity before starting the operator binaries.

## Whole-System Goal

Current state: The key management CLI is fully implemented with 10 commands
covering creation, import (mnemonic, raw seed, keyfile), listing, display,
switching, export, password change, and bootstrap command generation.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: An operator can generate, import, manage, and securely store
sr25519 keypairs, then produce ready-to-run miner and validator commands that
reference the encrypted key configuration.

Still not solved here: On-chain registration, staking, and the actual miner or
validator operation are separate concerns.

## Scope

In scope:
- 12-word sr25519 mnemonic generation
- Key import from mnemonic, raw 32-byte seed, or existing encrypted keyfile
- XSalsa20Poly1305 AEAD encryption with scrypt KDF
- TOML-based operator configuration tracking active account and network
- JSON keyfile format with version, cipher, KDF, and encrypted seed
- 0o600 Unix file permissions on keyfiles and config
- Active account listing, display, switching, and export
- Password rotation (re-encryption with new password)
- Bootstrap command generation for miner and validator startup
- Password sourced from environment variables (never CLI arguments)

Out of scope:
- Multi-signature or threshold key schemes
- Hardware wallet integration
- On-chain key rotation or migration
- Network connectivity or chain interaction (except in bootstrap output)
- Key derivation paths beyond the base sr25519 pair

## Current State

The CLI exists at crates/myosu-keys with approximately 1,960 lines of code. It
uses crypto_secretbox (XSalsa20Poly1305) for AEAD encryption, scrypt for
password-based key derivation, and sp-core for sr25519 key operations.

The config directory defaults to ~/.myosu and contains a config.toml (version 1,
tracking active_account, key_file, and network) and a keys/ subdirectory with
JSON keyfiles named by SS58 address.

Each keyfile stores: version (must be 1), address (SS58), cipher
("xsalsa20poly1305"), kdf ("scrypt"), scrypt parameters (log_n, r, p), salt
(32-byte hex), nonce (24-byte hex), and ciphertext (encrypted seed hex). The
seed is the raw 32-byte sr25519 secret. Scrypt uses recommended parameters
(log_n=15, r=8, p=1).

Password is always sourced from an environment variable (default
MYOSU_KEY_PASSWORD), never from CLI arguments or stdin prompts. The variable
name is configurable via --password-env.

The bootstrap command (print-bootstrap) loads the active account, decrypts it to
verify password correctness, and prints cargo run commands for myosu-miner and
myosu-validator with all necessary flags (--key-config-dir, --key-password-env,
--chain, --subnet).

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Key generation | generate_mnemonic + sr25519 pair derivation | Reuse | Standard substrate key generation |
| Encryption | XSalsa20Poly1305 with scrypt KDF | Reuse | Industry-standard authenticated encryption |
| Config tracking | TOML config with active account and network | Reuse | Simple persistent state |
| Keyfile format | JSON with version, cipher, KDF, encrypted seed | Reuse | Self-describing format |
| File permissions | 0o600 on keyfiles and config | Reuse | Standard secret file protection |
| Bootstrap output | Printed cargo commands for miner/validator | Reuse | Operator onboarding flow |

## Non-goals

- Providing a key management daemon or server.
- Integrating with system keyrings (GNOME Keyring, macOS Keychain).
- Supporting key types other than sr25519.
- Implementing multi-party key generation or threshold signing.
- Managing on-chain identity or staking from the keys CLI.

## Behaviors

The create command generates a 12-word mnemonic, derives an sr25519 pair,
encrypts the seed with the password from the configured environment variable,
writes the keyfile to ~/.myosu/keys/{address}.json with 0o600 permissions, and
creates or updates config.toml to set the new account as active. The mnemonic
is printed to stdout exactly once; it is not stored.

Import-mnemonic reads a mnemonic from an environment variable, derives the pair,
and follows the same encryption and persistence flow. Import-raw-seed reads a
32-byte hex seed (with optional 0x prefix) from an environment variable.
Import-keyfile copies an existing encrypted keyfile, validates its structure
(version, cipher, kdf), and registers it in the config.

List enumerates all JSON files in the keys directory, extracting addresses from
filenames, and prints them sorted.

Show-active reads config.toml and prints the active address, network, config
path, and key path without decrypting anything.

Switch-active validates that the target address has a keyfile, updates
config.toml, and prints the new active account.

Change-password loads the active keyfile, decrypts with the old password,
re-encrypts with the new password (generating fresh salt and nonce), and
overwrites the keyfile.

Export-active-keyfile copies the active account's encrypted keyfile to a
specified output path.

Print-bootstrap loads and decrypts the active account (verifying password
correctness), then prints cargo run commands for myosu-miner and
myosu-validator with the config directory, password environment variable, chain
endpoint, and subnet as arguments.

Decryption validates the keyfile structure (version must be 1, cipher must be
xsalsa20poly1305, kdf must be scrypt), derives the encryption key via scrypt,
decrypts via XSalsa20Poly1305, verifies the decrypted seed produces the stored
address, and returns the sr25519 pair.

## Acceptance Criteria

- Key creation generates a valid 12-word mnemonic and persists an encrypted
  keyfile.
- The mnemonic is printed exactly once during creation and is not stored on
  disk.
- Keyfiles are written with 0o600 permissions on Unix systems.
- Decryption with the correct password produces the original sr25519 pair.
- Decryption with an incorrect password fails with a descriptive error.
- Import from mnemonic, raw seed, and keyfile all produce valid encrypted
  keyfiles that can be decrypted.
- The active account can be switched between stored accounts.
- Password change produces a new keyfile that decrypts with the new password
  and rejects the old password.
- Bootstrap output produces syntactically valid cargo run commands for both
  miner and validator.
- Passwords are never read from CLI arguments, only from environment variables.
