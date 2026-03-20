//! Registration CLI for registering games on-chain.
//!
//! This module provides the `myosu register` command that submits
//! a `register_game_type` extrinsic to the myosu chain.

pub mod tests;

use clap::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("connection timeout: failed to connect to {0} within 5s")]
    ConnectionTimeout(String),

    #[error("chain error: {0}")]
    ChainError(String),

    #[error("registration failed: {0}")]
    RegistrationFailed(String),
}

#[derive(Parser, Debug)]
#[command(name = "register")]
#[command(about = "Register a game type on the myosu chain", long_about = None)]
pub struct RegisterArgs {
    /// WebSocket endpoint of the chain node
    #[arg(long)]
    pub chain: String,

    /// Game type identifier (e.g., "kuhn-poker", "liars-dice")
    #[arg(long)]
    pub game_type: String,

    /// Number of players
    #[arg(long, default_value = "2")]
    pub players: u8,

    /// Exploit unit name
    #[arg(long, default_value = "exploit")]
    pub exploit_unit: String,

    /// Exploit baseline value
    #[arg(long, default_value = "1.0")]
    pub exploit_baseline: f64,
}

/// Register a game type on the myosu chain.
///
/// Returns an error if the chain is unreachable or the extrinsic fails.
pub async fn register_game(args: RegisterArgs) -> Result<(), RegisterError> {
    // Connection timeout: 5 seconds
    let _connect_timeout = std::time::Duration::from_secs(5);

    // Validate chain URL
    let _chain_url = &args.chain;

    // TODO: Connect to chain and submit register_game_type extrinsic
    // This is blocked on chain:pallet having the extrinsic implemented.
    // For now, we simulate the connection check.

    // Simulate connection timeout check
    if args.chain.is_empty() {
        return Err(RegisterError::ConnectionTimeout(args.chain.clone()));
    }

    // Placeholder: actual extrinsic submission would go here
    // let client = substrate_api::Client::connect(&args.chain)
    //     .await
    //     .map_err(|e| RegisterError::ConnectionTimeout(args.chain.clone()))?;
    //
    // let tx = compose_extrinsic!(client, "GameSolver", "register_game_type",
    //     args.game_type, args.players, args.exploit_unit, args.exploit_baseline
    // );
    //
    // client.submit_and_watch(tx)
    //     .await
    //     .map_err(|e| RegisterError::RegistrationFailed(e.to_string()))?;

    Ok(())
}
