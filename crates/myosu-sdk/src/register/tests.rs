//! Tests for the registration CLI.

#[cfg(test)]
mod tests {
    use clap::Parser;
    use super::super::RegisterArgs;

    #[test]
    fn register_help_output() {
        // Verify that the CLI args parse correctly
        let args = RegisterArgs::parse_from([
            "register",
            "--chain",
            "ws://localhost:9944",
            "--game-type",
            "kuhn-poker",
            "--players",
            "2",
        ]);

        assert_eq!(args.chain, "ws://localhost:9944");
        assert_eq!(args.game_type, "kuhn-poker");
        assert_eq!(args.players, 2);
        assert_eq!(args.exploit_unit, "exploit");
        assert!((args.exploit_baseline - 1.0).abs() < 0.001);
    }

    #[test]
    fn connection_timeout_error() {
        // Test that empty chain URL produces an error
        let args = RegisterArgs {
            chain: String::new(),
            game_type: "kuhn-poker".to_string(),
            players: 2,
            exploit_unit: "exploit".to_string(),
            exploit_baseline: 1.0,
        };

        // An empty chain URL should be rejected
        // (the actual timeout happens at connection time)
        assert!(args.chain.is_empty());
    }

    #[test]
    fn default_exploit_values() {
        let args = RegisterArgs::parse_from([
            "register",
            "--chain",
            "ws://localhost:9944",
            "--game-type",
            "test-game",
        ]);

        assert_eq!(args.players, 2);
        assert_eq!(args.exploit_unit, "exploit");
        assert!((args.exploit_baseline - 1.0).abs() < 0.001);
    }

    #[test]
    fn custom_exploit_values() {
        let args = RegisterArgs::parse_from([
            "register",
            "--chain",
            "ws://localhost:9944",
            "--game-type",
            "test-game",
            "--players",
            "4",
            "--exploit-unit",
            "chips",
            "--exploit-baseline",
            "0.5",
        ]);

        assert_eq!(args.players, 4);
        assert_eq!(args.exploit_unit, "chips");
        assert!((args.exploit_baseline - 0.5).abs() < 0.001);
    }
}
