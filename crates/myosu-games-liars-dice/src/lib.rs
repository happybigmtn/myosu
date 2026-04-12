//! Minimal additive Liar's Dice proof crate for myosu.

pub mod dossier;
pub mod game;
pub mod policy_bundle;
pub mod protocol;
pub mod renderer;
pub mod solver;
pub mod wire;

pub use dossier::{
    LiarsDiceArtifactDossier, LiarsDiceBenchmarkDossier, LiarsDiceDossierError,
    read_liars_dice_artifact_dossier, write_liars_dice_artifact_dossier,
};
pub use game::{
    LiarsDiceClaim, LiarsDiceEdge, LiarsDiceGame, LiarsDiceInfo, LiarsDicePublic, LiarsDiceSecret,
    LiarsDiceTurn,
};
pub use policy_bundle::{
    LIARS_DICE_PROMOTION_THRESHOLD, LiarsDicePolicyBundleError, LiarsDicePolicyBundleEvidence,
    build_liars_dice_policy_bundle, build_liars_dice_policy_bundle_evidence,
};
pub use protocol::{LiarsDiceStrategyQuery, LiarsDiceStrategyResponse, recommended_edge};
pub use renderer::{LiarsDiceRenderer, LiarsDiceSnapshot};
pub use solver::{LiarsDiceSolver, LiarsDiceSolverError, LiarsDiceTrainingSummary};
pub use wire::{
    WireCodecError, decode_info, decode_strategy_query, decode_strategy_response, encode_info,
    encode_strategy_query, encode_strategy_response,
};

#[cfg(test)]
mod tests {
    use myosu_games::{GameRegistry, GameType};

    #[test]
    fn liars_dice_resolves_through_shared_registry() {
        let descriptor = GameRegistry::from_bytes(b"liars_dice")
            .expect("liar's dice should resolve as a built-in game");

        assert_eq!(descriptor.game_type, GameType::LiarsDice);
        assert_eq!(descriptor.num_players, 2);
        assert_eq!(descriptor.description, "Two-player liar's dice");
        assert!(descriptor.builtin);
        assert!(
            GameRegistry::supported()
                .iter()
                .any(|candidate| candidate.game_type == GameType::LiarsDice)
        );
    }

    #[test]
    fn liars_dice_manifest_stays_additive_to_shared_game_boundary() {
        let manifest = include_str!("../Cargo.toml");

        assert!(
            manifest.contains("myosu-games = { path = \"../myosu-games\" }"),
            "liar's dice should keep depending on the shared myosu-games crate"
        );
        assert!(
            !manifest.contains("myosu-games-poker"),
            "liar's dice should stay additive and avoid a poker-crate dependency"
        );
    }
}
