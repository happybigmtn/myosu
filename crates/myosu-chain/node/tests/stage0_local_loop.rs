use std::collections::BTreeMap;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
struct Stage0GameSummary {
    subnet: u16,
    alice_uid: u16,
    bob_uid: u16,
    bob_has_validator_permit: bool,
    bob_weights: String,
    gameplay_advice_source: String,
    gameplay_final_state: String,
    gameplay_discovered_miner_uid: Option<u16>,
    gameplay_discovered_miner_endpoint: Option<String>,
    gameplay_live_miner_connect_endpoint: Option<String>,
    alice_miner_incentive: u16,
    bob_validator_dividend: u16,
    alice_miner_emission: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct Stage0LoopSummary {
    poker: Stage0GameSummary,
    liars_dice: Stage0GameSummary,
    best_imported: u64,
    best_finalized: u64,
}

fn parse_kv_lines(output: &str) -> BTreeMap<String, String> {
    output
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn require_value(lines: &BTreeMap<String, String>, key: &str) -> String {
    match lines.get(key) {
        Some(value) => value.clone(),
        None => panic!("smoke should emit `{key}`"),
    }
}

fn parse_required_u16(lines: &BTreeMap<String, String>, key: &str) -> u16 {
    let value = require_value(lines, key);
    match value.parse::<u16>() {
        Ok(parsed) => parsed,
        Err(error) => panic!("`{key}` should parse as u16, got `{value}`: {error}"),
    }
}

fn parse_required_u64(lines: &BTreeMap<String, String>, key: &str) -> u64 {
    let value = require_value(lines, key);
    match value.parse::<u64>() {
        Ok(parsed) => parsed,
        Err(error) => panic!("`{key}` should parse as u64, got `{value}`: {error}"),
    }
}

fn parse_required_bool(lines: &BTreeMap<String, String>, key: &str) -> bool {
    let value = require_value(lines, key);
    match value.parse::<bool>() {
        Ok(parsed) => parsed,
        Err(error) => panic!("`{key}` should parse as bool, got `{value}`: {error}"),
    }
}

fn parse_optional_u16(lines: &BTreeMap<String, String>, key: &str) -> Option<u16> {
    lines.get(key).map(|value| match value.parse::<u16>() {
        Ok(parsed) => parsed,
        Err(error) => panic!("`{key}` should parse as u16, got `{value}`: {error}"),
    })
}

fn parse_optional_string(lines: &BTreeMap<String, String>, key: &str) -> Option<String> {
    lines.get(key).cloned()
}

fn parse_game_summary(lines: &BTreeMap<String, String>, prefix: &str) -> Stage0GameSummary {
    Stage0GameSummary {
        subnet: parse_required_u16(lines, &format!("{prefix}_subnet")),
        alice_uid: parse_required_u16(lines, &format!("{prefix}_alice_uid")),
        bob_uid: parse_required_u16(lines, &format!("{prefix}_bob_uid")),
        bob_has_validator_permit: parse_required_bool(
            lines,
            &format!("{prefix}_bob_has_validator_permit"),
        ),
        bob_weights: require_value(lines, &format!("{prefix}_bob_weights")),
        gameplay_advice_source: require_value(lines, &format!("{prefix}_gameplay_advice_source")),
        gameplay_final_state: require_value(lines, &format!("{prefix}_gameplay_final_state")),
        gameplay_discovered_miner_uid: parse_optional_u16(
            lines,
            &format!("{prefix}_gameplay_discovered_miner_uid"),
        ),
        gameplay_discovered_miner_endpoint: parse_optional_string(
            lines,
            &format!("{prefix}_gameplay_discovered_miner_endpoint"),
        ),
        gameplay_live_miner_connect_endpoint: parse_optional_string(
            lines,
            &format!("{prefix}_gameplay_live_miner_connect_endpoint"),
        ),
        alice_miner_incentive: parse_required_u16(
            lines,
            &format!("{prefix}_alice_miner_incentive"),
        ),
        bob_validator_dividend: parse_required_u16(
            lines,
            &format!("{prefix}_bob_validator_dividend"),
        ),
        alice_miner_emission: parse_required_u64(lines, &format!("{prefix}_alice_miner_emission")),
    }
}

fn parse_stage0_summary(stdout: &str) -> Stage0LoopSummary {
    assert!(
        stdout.contains("STAGE0 myosu-chain local-loop ok"),
        "stage-0 local loop smoke did not report success\nstdout:\n{}",
        stdout
    );
    let lines = parse_kv_lines(stdout);

    Stage0LoopSummary {
        poker: parse_game_summary(&lines, "poker"),
        liars_dice: parse_game_summary(&lines, "liars_dice"),
        best_imported: parse_required_u64(&lines, "best_imported"),
        best_finalized: parse_required_u64(&lines, "best_finalized"),
    }
}

fn assert_poker_contract(summary: &Stage0GameSummary) {
    assert!(summary.subnet > 0, "poker subnet should be non-root");
    assert_ne!(
        summary.alice_uid, summary.bob_uid,
        "poker miner and validator should register to distinct uids"
    );
    assert_eq!(
        summary.gameplay_advice_source, "artifact",
        "poker gameplay advice source should stay artifact-backed"
    );
    assert_eq!(
        summary.gameplay_final_state, "complete",
        "poker gameplay final state should stay complete"
    );
    assert!(
        summary.bob_has_validator_permit,
        "poker bob should retain validator permit"
    );
    assert_eq!(
        summary.gameplay_discovered_miner_uid,
        Some(summary.alice_uid),
        "poker discovered miner uid should match alice uid"
    );
    assert_eq!(
        summary.bob_weights,
        format!("[({}, 65535)]", summary.alice_uid),
        "poker bob weights should continue pointing fully at alice"
    );
    let discovered_endpoint = summary
        .gameplay_discovered_miner_endpoint
        .as_deref()
        .expect("poker smoke should emit discovered endpoint");
    assert!(
        discovered_endpoint.starts_with("0.0.0.0:"),
        "poker discovered endpoint should advertise wildcard bind address, got `{}`",
        discovered_endpoint
    );
    let live_endpoint = summary
        .gameplay_live_miner_connect_endpoint
        .as_deref()
        .expect("poker smoke should emit live connect endpoint");
    assert!(
        live_endpoint.starts_with("127.0.0.1:"),
        "poker live connect endpoint should use localhost, got `{}`",
        live_endpoint
    );
    let discovered_port = discovered_endpoint.rsplit(':').next();
    let live_port = live_endpoint.rsplit(':').next();
    assert_eq!(
        discovered_port, live_port,
        "poker discovered and live miner endpoints should agree on port"
    );
    assert_eq!(
        summary.alice_miner_incentive,
        u16::MAX,
        "poker alice miner incentive should stay saturated"
    );
    assert_eq!(
        summary.bob_validator_dividend,
        u16::MAX,
        "poker bob validator dividend should stay saturated"
    );
    assert!(
        summary.alice_miner_emission > 0,
        "poker alice miner emission should be positive"
    );
}

fn assert_liars_dice_contract(summary: &Stage0GameSummary) {
    assert!(summary.subnet > 0, "liar's dice subnet should be non-root");
    assert_ne!(
        summary.alice_uid, summary.bob_uid,
        "liar's dice miner and validator should register to distinct uids"
    );
    assert_eq!(
        summary.gameplay_advice_source, "artifact",
        "liar's dice gameplay advice source should stay artifact-backed"
    );
    assert_eq!(
        summary.gameplay_final_state, "static_demo",
        "liar's dice gameplay final state should stay static_demo"
    );
    assert!(
        summary.bob_has_validator_permit,
        "liar's dice bob should retain validator permit"
    );
    assert_eq!(
        summary.bob_weights,
        format!("[({}, 65535)]", summary.alice_uid),
        "liar's dice bob weights should continue pointing fully at alice"
    );
    assert!(
        summary.gameplay_discovered_miner_uid.is_none(),
        "liar's dice smoke should not emit chain discovery yet"
    );
    assert!(
        summary.gameplay_discovered_miner_endpoint.is_none(),
        "liar's dice smoke should not emit discovered endpoint yet"
    );
    assert!(
        summary.gameplay_live_miner_connect_endpoint.is_none(),
        "liar's dice smoke should not emit live connect endpoint yet"
    );
    assert_eq!(
        summary.alice_miner_incentive,
        u16::MAX,
        "liar's dice alice miner incentive should stay saturated"
    );
    assert_eq!(
        summary.bob_validator_dividend,
        u16::MAX,
        "liar's dice bob validator dividend should stay saturated"
    );
    assert!(
        summary.alice_miner_emission > 0,
        "liar's dice alice miner emission should be positive"
    );
}

fn assert_stage0_contract(summary: &Stage0LoopSummary) {
    assert!(
        summary.best_finalized > 0,
        "best finalized height should be positive"
    );
    assert!(
        summary.best_imported >= summary.best_finalized,
        "best imported height should not lag finalized height"
    );
    assert_ne!(
        summary.poker.subnet, summary.liars_dice.subnet,
        "poker and liar's dice should occupy distinct subnets"
    );
    assert_poker_contract(&summary.poker);
    assert_liars_dice_contract(&summary.liars_dice);
}

fn fixture_stage0_output() -> &'static str {
    "\
STAGE0 myosu-chain local-loop ok
poker_subnet=2
poker_alice_uid=0
poker_bob_uid=1
poker_bob_has_validator_permit=true
poker_bob_weights=[(0, 65535)]
poker_gameplay_advice_source=artifact
poker_gameplay_final_state=complete
poker_gameplay_discovered_miner_uid=0
poker_gameplay_discovered_miner_endpoint=0.0.0.0:45123
poker_gameplay_live_miner_connect_endpoint=127.0.0.1:45123
poker_alice_miner_incentive=65535
poker_bob_validator_dividend=65535
poker_alice_miner_emission=1230006864
liars_dice_subnet=3
liars_dice_alice_uid=0
liars_dice_bob_uid=1
liars_dice_bob_has_validator_permit=true
liars_dice_bob_weights=[(0, 65535)]
liars_dice_gameplay_advice_source=artifact
liars_dice_gameplay_final_state=static_demo
liars_dice_alice_miner_incentive=65535
liars_dice_bob_validator_dividend=65535
liars_dice_alice_miner_emission=4560006864
best_imported=16
best_finalized=13
"
}

#[test]
fn fixture_stage0_output_satisfies_contract() {
    let stdout = fixture_stage0_output();
    let summary = parse_stage0_summary(stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "smoke should emit `liars_dice_alice_miner_emission`")]
fn fixture_stage0_output_missing_required_field_panics() {
    let stdout =
        fixture_stage0_output().replace("liars_dice_alice_miner_emission=4560006864\n", "");

    let _summary = parse_stage0_summary(&stdout);
}

#[test]
#[should_panic(expected = "poker discovered and live miner endpoints should agree on port")]
fn fixture_stage0_output_mismatched_poker_miner_ports_panics() {
    let stdout = fixture_stage0_output().replace("127.0.0.1:45123", "127.0.0.1:45124");
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "best imported height should not lag finalized height")]
fn fixture_stage0_output_imported_below_finalized_panics() {
    let stdout = fixture_stage0_output().replace("best_imported=16", "best_imported=12");
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "best finalized height should be positive")]
fn fixture_stage0_output_zero_finalized_height_panics() {
    let stdout = fixture_stage0_output().replace("best_finalized=13", "best_finalized=0");
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "liar's dice alice miner emission should be positive")]
fn fixture_stage0_output_zero_liars_dice_emission_panics() {
    let stdout = fixture_stage0_output().replace(
        "liars_dice_alice_miner_emission=4560006864",
        "liars_dice_alice_miner_emission=0",
    );
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "poker discovered miner uid should match alice uid")]
fn fixture_stage0_output_wrong_poker_discovered_miner_uid_panics() {
    let stdout = fixture_stage0_output().replace(
        "poker_gameplay_discovered_miner_uid=0",
        "poker_gameplay_discovered_miner_uid=9",
    );
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "poker and liar's dice should occupy distinct subnets")]
fn fixture_stage0_output_duplicate_subnets_panics() {
    let stdout = fixture_stage0_output().replace("liars_dice_subnet=3", "liars_dice_subnet=2");
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "liar's dice bob weights should continue pointing fully at alice")]
fn fixture_stage0_output_wrong_liars_dice_weight_target_panics() {
    let stdout = fixture_stage0_output().replace(
        "liars_dice_bob_weights=[(0, 65535)]",
        "liars_dice_bob_weights=[(1, 65535)]",
    );
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "liar's dice smoke should not emit chain discovery yet")]
fn fixture_stage0_output_unexpected_liars_dice_discovery_panics() {
    let stdout = format!(
        "{}{}",
        fixture_stage0_output(),
        "liars_dice_gameplay_discovered_miner_uid=0\n"
    );
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "poker gameplay advice source should stay artifact-backed")]
fn fixture_stage0_output_wrong_poker_advice_source_panics() {
    let stdout = fixture_stage0_output().replace(
        "poker_gameplay_advice_source=artifact",
        "poker_gameplay_advice_source=live_http",
    );
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[should_panic(expected = "liar's dice gameplay final state should stay static_demo")]
fn fixture_stage0_output_wrong_liars_dice_final_state_panics() {
    let stdout = fixture_stage0_output().replace(
        "liars_dice_gameplay_final_state=static_demo",
        "liars_dice_gameplay_final_state=in_progress",
    );
    let summary = parse_stage0_summary(&stdout);

    assert_stage0_contract(&summary);
}

#[test]
#[ignore = "runs the full local stage-0 chain/miner/validator/gameplay loop"]
fn stage0_local_loop_smoke_contract_holds() {
    let binary = env!("CARGO_BIN_EXE_myosu-chain");
    let output = match Command::new(binary)
        .arg("--stage0-local-loop-smoke")
        .output()
    {
        Ok(output) => output,
        Err(error) => panic!("stage-0 local loop smoke should spawn: {error}"),
    };
    let stdout = match String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        Err(error) => panic!("stdout should be valid UTF-8: {error}"),
    };
    let stderr = match String::from_utf8(output.stderr) {
        Ok(stderr) => stderr,
        Err(error) => panic!("stderr should be valid UTF-8: {error}"),
    };

    assert!(
        output.status.success(),
        "stage-0 local loop smoke failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr
    );

    let summary = parse_stage0_summary(&stdout);
    assert_stage0_contract(&summary);
}
