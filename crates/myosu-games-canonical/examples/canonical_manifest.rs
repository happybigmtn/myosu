use std::io;

use myosu_games_canonical::{
    CANONICAL_TEN, canonical_action_specs, canonical_bootstrap_snapshot, canonical_game_spec,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for game in CANONICAL_TEN {
        let spec = canonical_game_spec(game).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("missing canonical spec for {}", game.slug()),
            )
        })?;
        let actions = canonical_action_specs(game).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("missing canonical action specs for {}", game.slug()),
            )
        })?;
        canonical_bootstrap_snapshot(game)?;

        println!(
            "CANONICAL_GAME slug={} chain_id={} ruleset_version={} actions={} rule_file={} snapshot=ok",
            spec.slug,
            spec.chain_id,
            spec.ruleset_version,
            actions.len(),
            spec.rule_file.as_deref().unwrap_or("none"),
        );
    }

    Ok(())
}
