#![doc = include_str!("../README.md")]

pub mod action;
pub mod artifacts;
pub mod benchmark;
pub mod codexpoker;
pub mod lookup_dump;
pub mod renderer;
pub mod request;
pub mod robopoker;
pub mod solver;
pub mod state;
pub mod wire;

pub use action::NlheAction;
pub use artifacts::{
    ArtifactCodecError, NlheAbstractionArtifactEntry, NlheAbstractionManifest,
    NlheAbstractionStreet, NlheBootstrapScenario, NlheEncoderArtifactBundle,
    NlheEncoderArtifactSummary, bootstrap_encoder_streets, bootstrap_scenarios, decode_encoder,
    encode_encoder, encoder_from_lookup, load_encoder_bundle, load_encoder_dir, write_encoder_dir,
};
pub use benchmark::{
    NlheScenarioBenchmarkError, NlheScenarioBenchmarkReport, NlheScenarioBenchmarkRow,
    benchmark_against_bootstrap_reference, benchmark_solver_against_reference,
    bootstrap_reference_solver,
};
pub use benchmark::{PokerBenchmarkError, PokerBenchmarkPoint, benchmark_points_from_encoder_dir};
pub use codexpoker::{CodexpokerBlueprint, CodexpokerBlueprintError};
pub use lookup_dump::{LookupDumpError, write_encoder_dir_from_lookup_dump};
pub use renderer::NlheRenderer;
pub use request::{NlheHistoryAction, NlheStrategyRequest, StrategyRequestError};
pub use robopoker::{
    NlheBlueprint, NlheFlagshipSolver, NlheInfoKey, NlheStrategyQuery, NlheStrategyResponse,
    RbpNlheEdge, RbpNlheEncoder, RbpNlheGame, RbpNlheInfo, RbpNlheProfile, RbpNlheStrategy,
    recommended_edge,
};
pub use solver::{PokerSolver, PokerSolverError, PokerTrainingSummary};
pub use state::{NlheActor, NlhePlayerState, NlheSnapshot, NlheStreet, NlheTablePosition};
pub use wire::{
    WireCodecError, decode_info_key, decode_strategy_query, decode_strategy_response,
    encode_info_key, encode_strategy_query, encode_strategy_response,
};
