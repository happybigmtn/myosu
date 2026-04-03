use crate::{
    client::{FullBackend, FullClient},
    consensus::{AuraConsensus, BabeConsensus},
    service::new_chain_ops,
};
use myosu_chain_runtime::opaque::Block;
use sc_cli::RunCmd;
use sc_consensus::BasicQueue;
use sc_service::{Configuration, TaskManager};
use std::sync::Arc;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// Run a bounded local devnet smoke test and exit.
    #[arg(long)]
    pub smoke_test: bool,

    /// Run the full stage-0 local miner/validator loop smoke test and exit.
    #[arg(long)]
    pub stage0_local_loop_smoke: bool,

    /// Run a bounded live smoke that registers two subnets back-to-back.
    #[arg(long)]
    pub dual_register_smoke: bool,

    #[clap(flatten)]
    pub run: RunCmd,

    /// Choose sealing method.
    #[arg(long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,

    /// Whether to try Aura or Babe consensus on first start.
    ///
    /// After starting, the consensus used by the node will automatically
    /// switch to whatever is required to continue validating / syncing.
    ///
    /// Retain this override while first-start consensus compatibility still
    /// needs to cover both Aura and legacy Babe expectations.
    #[arg(long, value_enum, ignore_case = true, default_value_t=SupportedConsensusMechanism::default())]
    pub initial_consensus: SupportedConsensusMechanism,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    // Key management cli utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    // Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    // Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    // Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    // Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    // Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    // Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    // Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    // Sub-commands concerned with benchmarking.
    #[cfg(feature = "runtime-benchmarks")]
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    // Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}

/// Available Sealing methods.
#[derive(Copy, Clone, Debug, Default, clap::ValueEnum)]
pub enum Sealing {
    /// Seal using rpc method.
    #[default]
    Manual,
    /// Seal when transaction is executed.
    Instant,
}

/// Supported consensus mechanisms.
#[derive(Copy, Clone, Debug, Default, clap::ValueEnum)]
pub enum SupportedConsensusMechanism {
    // Babe
    Babe,
    /// Aura
    #[default]
    Aura,
}

// Convinience methods for static dispatch of different service methods with
// different consensus mechanisms.
impl SupportedConsensusMechanism {
    pub fn new_chain_ops(
        &self,
        config: &mut Configuration,
    ) -> Result<
        (
            Arc<FullClient>,
            Arc<FullBackend>,
            BasicQueue<Block>,
            TaskManager,
        ),
        sc_service::Error,
    > {
        match self {
            SupportedConsensusMechanism::Aura => new_chain_ops::<AuraConsensus>(config),
            SupportedConsensusMechanism::Babe => new_chain_ops::<BabeConsensus>(config),
        }
    }
}
