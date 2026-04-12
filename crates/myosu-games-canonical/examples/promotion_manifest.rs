use std::{env, io, path::PathBuf};

use myosu_games_canonical::{read_solver_promotion_ledger, solver_promotion_manifest_rows};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = env::args().nth(1).unwrap_or_else(|| "table".to_string());
    let ledger_path = env::var("MYOSU_SOLVER_PROMOTION_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root().join("ops/solver_promotion.yaml"));
    let ledger = read_solver_promotion_ledger(ledger_path)?;
    let rows = solver_promotion_manifest_rows(&ledger)?;

    match mode.as_str() {
        "table" => print_table(&rows),
        "json" => println!("{}", serde_json::to_string_pretty(&rows)?),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: cargo run -p myosu-games-canonical --example promotion_manifest -- [table|json]",
            )
            .into());
        }
    }

    Ok(())
}

fn print_table(rows: &[myosu_games_canonical::SolverPromotionManifestRow]) {
    println!("SOLVER_PROMOTION total={}", rows.len());
    for row in rows {
        println!(
            "SOLVER_PROMOTION_GAME slug={} chain_id={} route={} tier={} benchmark_surface={} benchmark_threshold={} artifact_requirement={} declared_bundle_support={} code_bundle_support={} bitino_target_phase={} notes={}",
            row.slug,
            row.chain_id,
            row.route.as_str(),
            row.tier.as_str(),
            shell_token(&row.benchmark_surface),
            shell_token(&row.benchmark_threshold),
            shell_token(&row.artifact_requirement),
            row.declared_bundle_support.as_str(),
            row.code_bundle_support.as_str(),
            shell_token(&row.bitino_target_phase),
            shell_token(&row.notes),
        );
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn shell_token(value: &str) -> String {
    value.replace(char::is_whitespace, "_")
}
