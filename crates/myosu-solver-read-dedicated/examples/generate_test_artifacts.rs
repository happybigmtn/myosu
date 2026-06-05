//! Generate test artifacts for the W-07 e2e proof harness.
//!
//! Creates:
//!   1. A Liar's Dice checkpoint (zero-iteration, N=1) at `<output-dir>/liars-dice/checkpoint.bin`
//!   2. An NLHE encoder directory at `<output-dir>/nlhe/encoder`
//!   3. An NLHE checkpoint with a synthetic non-empty profile at `<output-dir>/nlhe/checkpoint.bin`
//!
//! Usage:
//!   cargo run -p myosu-solver-read-dedicated --example generate_test_artifacts -- <output-dir>

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use myosu_games_liars_dice::LiarsDiceSolver;
use myosu_games_poker::{
    PokerSolver, RbpNlheEdge as NlheEdge, RbpNlheInfo as NlheInfo, bootstrap_encoder_streets,
    load_encoder_dir, write_encoder_dir,
};
use rbp_gameplay::{Abstraction, Edge, Odds, Path};
use rbp_mccfr::Encounter;
use rbp_nlhe::NlheProfile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: generate_test_artifacts <output-dir>")?;

    std::fs::create_dir_all(&output_dir)?;

    // Liar's Dice checkpoint (N=1, zero iterations)
    let ld_dir = output_dir.join("liars-dice");
    std::fs::create_dir_all(&ld_dir)?;
    let ld_checkpoint = ld_dir.join("checkpoint.bin");
    let ld_solver = LiarsDiceSolver::<1>::new();
    ld_solver.save(&ld_checkpoint)?;
    println!("ARTIFACT liars_dice_checkpoint={}", ld_checkpoint.display());

    // NLHE encoder directory + checkpoint with synthetic profile
    let nlhe_dir = output_dir.join("nlhe");
    std::fs::create_dir_all(&nlhe_dir)?;
    let nlhe_encoder_dir = nlhe_dir.join("encoder");
    std::fs::create_dir_all(&nlhe_encoder_dir)?;
    let streets = bootstrap_encoder_streets();
    write_encoder_dir(&nlhe_encoder_dir, streets)?;
    let nlhe_checkpoint = nlhe_dir.join("checkpoint.bin");

    let encoder = load_encoder_dir(&nlhe_encoder_dir)?;

    // Build the exact info + profile shape the poker solver tests use
    // so the checkpoint answers a query without returning empty.
    let subgame = vec![Edge::Check, Edge::Raise(Odds::new(1, 2))]
        .into_iter()
        .collect::<Path>();
    let choices = vec![Edge::Fold, Edge::Call, Edge::Raise(Odds::new(1, 1))]
        .into_iter()
        .collect::<Path>();
    let bucket = Abstraction::from(42_i16);
    let info = NlheInfo::from((subgame, bucket, choices));

    let profile = NlheProfile {
        iterations: 1,
        encounters: BTreeMap::from([(
            info,
            BTreeMap::from([
                (
                    NlheEdge::from(Edge::Fold),
                    Encounter::new(0.05, 0.0, 0.0, 1),
                ),
                (
                    NlheEdge::from(Edge::Call),
                    Encounter::new(0.80, 0.0, 0.0, 1),
                ),
                (
                    NlheEdge::from(Edge::Raise(Odds::new(1, 1))),
                    Encounter::new(0.15, 0.0, 0.0, 1),
                ),
            ]),
        )]),
        metrics: rbp_mccfr::Metrics::with_epoch(1),
    };

    let nlhe_solver = PokerSolver::from_parts(profile, encoder);
    nlhe_solver.save(&nlhe_checkpoint)?;
    println!("ARTIFACT nlhe_encoder_dir={}", nlhe_encoder_dir.display());
    println!("ARTIFACT nlhe_checkpoint={}", nlhe_checkpoint.display());

    Ok(())
}
