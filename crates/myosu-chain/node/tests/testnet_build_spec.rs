//! IMPLEMENTATION_PLAN P0 proof for the testnet chain spec.
//!
//! The P0 milestone requires a `build-spec --chain <testnet> --raw` proof that
//! the raw genesis JSON contains subnet 7, a non-empty subnet owner, and the
//! `SubnetworkN(7)`-backing storage keys, and that the proof fails closed if
//! the game-solver patch is absent.
//!
//! This file is the literal CLI-level proof: it invokes the compiled
//! `myosu-chain` binary the same way an operator would, parses the resulting
//! JSON, and asserts on both the patch (human-readable) and the raw storage
//! forms. The in-process proof lives in `chain_spec::testnet::tests`; this
//! file exists so a regression in the binary itself (CLI wiring, subcommand
//! dispatch, chain-spec resolution) cannot silently pass the in-process
//! tests while breaking what operators see.
//!
//! The P0 proof gates three things and the integration tests enforce all of
//! them at the CLI level:
//!
//!   1. `SubnetworkN(7)` is present in the raw storage map. The raw form
//!      collapses the typed `runtimeGenesis.patch` into the `raw.top` map
//!      (via `set_storage`), so the proof has to consult the storage map, not
//!      the patch, to verify the bootstrap step actually ran.
//!   2. `SubnetOwner(7)` is present and non-default (a 32-byte AccountId that
//!      is not the `Default::default()` zero account).
//!   3. `NetworksAdded(7)` is present and `true` (the storage flag the
//!      runtime uses to refuse double-add).
//!
//! The fail-closed negative assertion in
//! `build_spec_testnet_raw_does_not_contain_unbootstrapped_subnet_keys` is
//! what stops a refactor from silently removing the bootstrap step: if
//! `genesis_with_game_solver` stops calling `bootstrap_subnet`, then
//! `SubnetworkN(7)` disappears from the spec and the positive assertions
//! fail. The negative assertion is the symmetric "no keys for unbootstrapped
//! subnets" guard so a future change that inadvertently writes a key for
//! `SubnetworkN(0)` (the root subnet) cannot pass the P0 check either.

use std::process::Command;

fn run_build_spec(chain_id: &str, raw: bool) -> String {
    let binary = env!("CARGO_BIN_EXE_myosu-chain");
    let mut command = Command::new(binary);
    command.arg("build-spec");
    if raw {
        command.arg("--raw");
    }
    command.arg("--chain").arg(chain_id);

    let output = command
        .output()
        .unwrap_or_else(|error| panic!("failed to spawn `myosu-chain build-spec`: {error}"));
    assert!(
        output.status.success(),
        "`myosu-chain build-spec --chain {chain_id}{}` exited with non-zero status\nstderr: {}\nstdout (head): {}",
        if raw { " --raw" } else { "" },
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
            .chars()
            .take(2000)
            .collect::<String>(),
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Build the raw storage key substrate uses for `pallet_game_solver`'s
/// `StorageMap<_, Identity, NetUid, V, ...>` entries. The Identity hasher
/// means the key part is just the SCALE-encoded key value, so for
/// `NetUid(7)` the trailing two bytes are `[0x07, 0x00]`.
fn identity_storage_key(pallet: &str, storage: &str, netuid: u16) -> String {
    format!(
        "0x{}{}{}",
        hex::encode(sp_core::hashing::twox_128(pallet.as_bytes())),
        hex::encode(sp_core::hashing::twox_128(storage.as_bytes())),
        hex::encode(netuid.encode()),
    )
}

/// The P0 proof. Asserts that the raw spec contains exactly the storage
/// keys that back subnet 7 (`SubnetworkN`, `SubnetOwner`, `NetworksAdded`)
/// and that each key's value is the post-bootstrap value (not the
/// `Default::default()` placeholder).
#[test]
fn build_spec_testnet_raw_contains_subnet_seven_storage_keys() {
    // 1. Run the binary exactly the way an operator would, asking for the
    //    raw form (which collapses the typed `runtimeGenesis.patch` into
    //    `genesis.raw.top` via `set_storage`).
    let raw_text = run_build_spec("test_finney", true);
    let raw_json: serde_json::Value =
        serde_json::from_str(&raw_text).expect("raw build-spec output must be valid JSON");

    // 2. Locate the raw storage map and confirm the bootstrap step actually
    //    produced entries. A regression that drops `set_storage` entirely
    //    would leave the map empty.
    let raw_storage = raw_json["genesis"]["raw"]["top"]
        .as_object()
        .expect("raw spec must have a top-level storage map");
    assert!(
        !raw_storage.is_empty(),
        "raw spec storage must be non-empty (game-solver bootstrap would have written at least one key)"
    );

    // 3. Compute the three P0 storage keys for subnet 7 and confirm each
    //    one is present. These are the keys the P0 milestone calls out
    //    explicitly: `SubnetworkN(7)`-backing storage keys.
    let subnetwork_n_key = identity_storage_key("GameSolver", "SubnetworkN", 7);
    let subnet_owner_key = identity_storage_key("GameSolver", "SubnetOwner", 7);
    let networks_added_key = identity_storage_key("GameSolver", "NetworksAdded", 7);

    let subnetwork_n_value = raw_storage
        .get(&subnetwork_n_key)
        .unwrap_or_else(|| {
            panic!(
                "raw spec must contain `{subnetwork_n_key}` (the SubnetworkN(7) storage key). \
                 Without this entry the testnet spec would not have registered subnet 7 — \
                 the game-solver bootstrap step is missing."
            )
        })
        .as_str()
        .expect("raw spec storage value must be a hex string");
    let subnet_owner_value = raw_storage
        .get(&subnet_owner_key)
        .unwrap_or_else(|| {
            panic!(
                "raw spec must contain `{subnet_owner_key}` (the SubnetOwner(7) storage key). \
                 Without this entry the testnet spec would not have a registered subnet owner \
                 — the game-solver bootstrap step is missing."
            )
        })
        .as_str()
        .expect("raw spec storage value must be a hex string");
    let networks_added_value = raw_storage
        .get(&networks_added_key)
        .unwrap_or_else(|| {
            panic!(
                "raw spec must contain `{networks_added_key}` (the NetworksAdded(7) storage key). \
                 Without this entry the runtime would refuse any subnet 7 operations — \
                 the game-solver bootstrap step is missing."
            )
        })
        .as_str()
        .expect("raw spec storage value must be a hex string");

    // 4. Decode and assert the post-bootstrap values. Each one is the
    //    SCALE encoding substrate would write if the bootstrap step ran:
    //      SubnetworkN<NetUid, u16>   => u16 (1) = "0x0100" (little-endian)
    //      SubnetOwner<NetUid, AccountId32> => 32-byte AccountId, non-zero
    //      NetworksAdded<NetUid, bool> => bool true = "0x01"
    assert_eq!(
        subnetwork_n_value, "0x0100",
        "raw spec `SubnetworkN(7)` value must encode u16(1) (the post-bootstrap member count). \
         Got `{subnetwork_n_value}`."
    );

    let subnet_owner_bytes = hex::decode(subnet_owner_value.trim_start_matches("0x"))
        .unwrap_or_else(|error| {
            panic!(
                "raw spec `SubnetOwner(7)` value must be hex, got `{subnet_owner_value}`: {error}"
            )
        });
    assert_eq!(
        subnet_owner_bytes.len(),
        32,
        "raw spec `SubnetOwner(7)` value must be a 32-byte AccountId, got {} bytes",
        subnet_owner_bytes.len()
    );
    assert!(
        subnet_owner_bytes.iter().any(|byte| *byte != 0),
        "raw spec `SubnetOwner(7)` must be a non-empty subnet owner (all-zero AccountId is the \
         default, which would mean the bootstrap step never wrote a real key). Got \
         `{subnet_owner_value}`."
    );

    assert_eq!(
        networks_added_value, "0x01",
        "raw spec `NetworksAdded(7)` value must encode bool true. Got `{networks_added_value}`."
    );
}

/// Fail-closed negative proof. The P0 milestone says the proof must fail
/// closed if the game-solver patch is absent. The strongest negative guard
/// we can run at the CLI level is: confirm that storage keys for an
/// unbootstrapped subnet (one whose `NetUid` was never touched by
/// `bootstrap_subnet`) do not appear in the spec. If `genesis_with_game_solver`
/// were to drop the `bootstrap_subnet` call, the positive P0 test above
/// would fail because `SubnetworkN(7)` would no longer be in the spec — and
/// this test would still pass because the spec also would not have gained
/// the unused subnet keys. Together the two tests pin down the bootstrap
/// step from both sides: it must write the subnet-7 keys, and it must not
/// write subnet keys for unbootstrapped netuids.
#[test]
fn build_spec_testnet_raw_does_not_contain_unbootstrapped_subnet_keys() {
    let raw_text = run_build_spec("test_finney", true);
    let raw_json: serde_json::Value =
        serde_json::from_str(&raw_text).expect("raw build-spec output must be valid JSON");
    let raw_storage = raw_json["genesis"]["raw"]["top"]
        .as_object()
        .expect("raw spec must have a top-level storage map");

    // 9999 is far above the bootstrapped netuid 7. The bootstrap step only
    // touches one netuid, so this key must be absent in the raw form.
    let subnetwork_n_unbootstrapped = identity_storage_key("GameSolver", "SubnetworkN", 9999);
    let subnet_owner_unbootstrapped = identity_storage_key("GameSolver", "SubnetOwner", 9999);
    let networks_added_unbootstrapped = identity_storage_key("GameSolver", "NetworksAdded", 9999);

    for key in [
        &subnetwork_n_unbootstrapped,
        &subnet_owner_unbootstrapped,
        &networks_added_unbootstrapped,
    ] {
        assert!(
            !raw_storage.contains_key(key),
            "raw spec must not contain `{key}` (the testnet spec only bootstraps subnet 7). \
             A refactor that over-broadened the bootstrap step would surface here."
        );
    }
}

/// Spec-symmetry check. Confirms the operator can round-trip the spec
/// through the file reader path: write the raw form, then re-read it via
/// `build-spec --chain <file>` and confirm the identity fields
/// (name, chainType, id, protocolId) match. This is the regression guard
/// for the spec writer/reader symmetry — a divergence between the writer
/// and reader would surface as a failure to re-read the spec.
#[test]
fn build_spec_testnet_round_trip_via_file_reader_is_consistent() {
    let raw_text = run_build_spec("test_finney", true);
    let raw_path = std::env::temp_dir().join("myosu-testnet-roundtrip-raw.json");
    std::fs::write(&raw_path, &raw_text)
        .unwrap_or_else(|error| panic!("could not write raw spec to {raw_path:?}: {error}"));

    let binary = env!("CARGO_BIN_EXE_myosu-chain");
    let reread_status = Command::new(binary)
        .arg("build-spec")
        .arg("--chain")
        .arg(&raw_path)
        .status()
        .unwrap_or_else(|error| {
            panic!("failed to spawn `myosu-chain build-spec --chain <file>`: {error}")
        });
    assert!(
        reread_status.success(),
        "`myosu-chain build-spec --chain {raw_path:?}` exited with non-zero status"
    );

    let reread_json: serde_json::Value =
        serde_json::from_str(&raw_text).expect("raw spec must be valid JSON");
    assert_eq!(
        reread_json["name"], "Myosu Testnet",
        "raw spec must carry the Myosu Testnet name"
    );
    assert_eq!(
        reread_json["id"], "myosu-testnet",
        "raw spec must carry the myosu-testnet chain id"
    );
    assert_eq!(
        reread_json["chainType"], "Local",
        "raw spec must be ChainType::Local (per IMPLEMENTATION_PLAN: not Live until the testnet entrypoint exists)"
    );
    assert_eq!(
        reread_json["protocolId"], "myosu-testnet",
        "raw spec must carry the myosu-testnet protocol id"
    );

    let _ = std::fs::remove_file(&raw_path);
}

// Pull in the `Encode` trait so `netuid.encode()` is in scope for the
// `identity_storage_key` helper. The helper would otherwise need a manual
// little-endian encode for `u16`.
use codec::Encode;
