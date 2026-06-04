//! Operator-facing HTTP axon validator-scoring example.
//!
//! NEM-004 requires the validator side of the live HTTP axon round
//! trip to have an executable proof: the new
//! `myosu_validator::http_axon::query_miner_axon_http` function must
//! actually talk to a real `myosu-miner --serve-http` over a TCP
//! socket, decode the live wire-format response, and emit the
//! operator-facing `HTTP_AXON myosu-validator axon ok` summary that
//! the e2e proof harness greps for. The unit tests in
//! `myosu_validator::http_axon::tests` cover the client against an
//! in-process `tokio::net::TcpListener` mock; this example covers it
//! against a real binary.
//!
//! Usage:
//!   cargo run -p myosu-validator --example http_axon_validator_scoring -- \
//!     --endpoint http://127.0.0.1:8191 --subgame 0 --bucket 0 --choices 0
//!
//! The example takes one positional endpoint plus the three NLHE
//! information-set key components (subgame / bucket / choices) and
//! prints a stable `HTTP_AXON myosu-validator axon ok` line on
//! success or a typed `HTTP_AXON myosu-validator axon fail` line on
//! failure. The e2e proof harness (`tests/e2e/http_axon_validator_scoring.sh`)
//! boots a real `myosu-miner --serve-http`, runs this binary, and
//! fail-closes on any missing or malformed key.

use std::process::ExitCode;

use myosu_games_poker::{NlheInfoKey, NlheStrategyQuery};
use myosu_validator::http_axon::{MinerAxonQueryReport, query_miner_axon_http};
use myosu_validator::http_axon_report;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let parsed = match parse_args(&args) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("http_axon_validator_scoring: {error}");
            return ExitCode::from(2);
        }
    };

    let query = NlheStrategyQuery::new(NlheInfoKey {
        subgame: parsed.subgame,
        bucket: parsed.bucket,
        choices: parsed.choices,
    });

    let report = match query_miner_axon_http(&parsed.endpoint, &query).await {
        Ok(report) => report,
        Err(error) => {
            eprintln!("HTTP_AXON myosu-validator axon fail");
            eprintln!("endpoint={}", parsed.endpoint);
            eprintln!("status={}", classify_error(&error));
            eprintln!("error={error}");
            return ExitCode::from(1);
        }
    };

    print_report(&report);
    if report.is_healthy() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

struct ParsedArgs {
    endpoint: String,
    subgame: u64,
    bucket: i16,
    choices: u64,
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut endpoint: Option<String> = None;
    let mut subgame: Option<u64> = None;
    let mut bucket: Option<i16> = None;
    let mut choices: Option<u64> = None;

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| format!("flag `{arg}` requires a value"))?;
        match arg.as_str() {
            "--endpoint" => {
                endpoint = Some(value.clone());
            }
            "--subgame" => {
                subgame = Some(
                    value
                        .parse()
                        .map_err(|error| format!("invalid --subgame `{value}`: {error}"))?,
                );
            }
            "--bucket" => {
                bucket = Some(
                    value
                        .parse()
                        .map_err(|error| format!("invalid --bucket `{value}`: {error}"))?,
                );
            }
            "--choices" => {
                choices = Some(
                    value
                        .parse()
                        .map_err(|error| format!("invalid --choices `{value}`: {error}"))?,
                );
            }
            other => return Err(format!("unknown flag `{other}`")),
        }
    }

    let endpoint = endpoint.ok_or_else(|| "--endpoint is required".to_string())?;
    let subgame = subgame.ok_or_else(|| "--subgame is required".to_string())?;
    let bucket = bucket.ok_or_else(|| "--bucket is required".to_string())?;
    let choices = choices.ok_or_else(|| "--choices is required".to_string())?;

    Ok(ParsedArgs {
        endpoint,
        subgame,
        bucket,
        choices,
    })
}

fn print_report(report: &MinerAxonQueryReport) {
    println!("{}", http_axon_report(report));
}

fn classify_error(error: &myosu_validator::http_axon::HttpAxonError) -> &'static str {
    use myosu_validator::http_axon::HttpAxonError;
    match error {
        HttpAxonError::Connect { .. } => "connect_failed",
        HttpAxonError::Write { .. } => "write_failed",
        HttpAxonError::Read { .. } => "read_failed",
        HttpAxonError::BadStatus { .. } => "bad_status",
        HttpAxonError::Decode { .. } => "decode_failed",
        HttpAxonError::Encode { .. } => "encode_failed",
        HttpAxonError::ResponseTooLarge { .. } => "response_too_large",
        HttpAxonError::MalformedStatusLine { .. } => "malformed_status_line",
    }
}
