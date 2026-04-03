//! Kuhn poker proof crate for validating additive game integration.

pub mod game;
pub mod solver;

pub use game::{KuhnCard, KuhnEdge, KuhnGame, KuhnHistory, KuhnInfo, KuhnPublic, KuhnTurn};
pub use solver::KuhnSolver;

#[cfg(test)]
mod tests {
    use myosu_games::{GameRegistry, GameType};

    #[test]
    fn kuhn_resolves_through_shared_registry() {
        let descriptor = GameRegistry::from_bytes(b"kuhn_poker")
            .expect("kuhn poker should resolve as a built-in game");

        assert_eq!(descriptor.game_type, GameType::KuhnPoker);
        assert_eq!(descriptor.num_players, 2);
        assert_eq!(descriptor.description, "Two-player Kuhn poker");
        assert!(descriptor.builtin);
        assert!(
            GameRegistry::supported()
                .iter()
                .any(|candidate| candidate.game_type == GameType::KuhnPoker)
        );
    }

    #[test]
    fn kuhn_manifest_stays_additive_to_shared_game_boundary() {
        let manifest = include_str!("../Cargo.toml");

        assert!(
            manifest.contains("myosu-games = { path = \"../myosu-games\" }"),
            "kuhn poker should keep depending on the shared myosu-games crate"
        );
        assert!(
            !manifest.contains("myosu-games-poker"),
            "kuhn poker should stay additive and avoid a poker-crate dependency"
        );
        assert!(
            !manifest.contains("myosu-games-liars-dice"),
            "kuhn poker should stay additive and avoid a liar's dice dependency"
        );
    }
}
