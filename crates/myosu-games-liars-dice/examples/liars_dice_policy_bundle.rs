use std::{env, error::Error, fs, path::PathBuf};

use myosu_games_liars_dice::{
    LIARS_DICE_PROMOTION_THRESHOLD, LiarsDiceSolver, build_liars_dice_policy_bundle_evidence,
};

const SOLVER_TREES: usize = 1 << 10;
const DEFAULT_ITERATIONS: usize = 512;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse()?;
    let mut solver = LiarsDiceSolver::<SOLVER_TREES>::new();
    solver.train_select_best(args.iterations)?;
    let evidence = build_liars_dice_policy_bundle_evidence(&solver, "opening-p1-die-2")?;

    let output_parent = args
        .output
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_parent)?;

    write_json(&args.output, &evidence.bundle)?;
    write_json(
        output_parent.join("benchmark-summary.json"),
        &evidence.bundle.provenance.benchmark,
    )?;
    write_json(
        output_parent.join("artifact-manifest.json"),
        &evidence.artifact_dossier,
    )?;

    println!(
        "POLICY_BUNDLE game=liars-dice output={}",
        args.output.display()
    );
    println!("POLICY_BUNDLE bundle_hash={}", evidence.bundle.bundle_hash);
    println!(
        "POLICY_BUNDLE exact_exploitability={:.6} threshold={:.6} passing={}",
        evidence.artifact_dossier.exploitability,
        LIARS_DICE_PROMOTION_THRESHOLD,
        evidence.artifact_dossier.passing
    );

    Ok(())
}

#[derive(Debug)]
struct Args {
    output: PathBuf,
    iterations: usize,
}

impl Args {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut output = None;
        let mut iterations = DEFAULT_ITERATIONS;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--output" => {
                    let Some(value) = args.next() else {
                        return Err("missing value for --output".into());
                    };
                    output = Some(PathBuf::from(value));
                }
                "--iterations" => {
                    let Some(value) = args.next() else {
                        return Err("missing value for --iterations".into());
                    };
                    iterations = value.parse::<usize>()?;
                }
                "--help" | "-h" => {
                    return Err(usage().into());
                }
                _ => {
                    return Err(format!("unknown argument `{arg}`\n{}", usage()).into());
                }
            }
        }

        let Some(output) = output else {
            return Err(usage().into());
        };

        Ok(Self { output, iterations })
    }
}

fn usage() -> &'static str {
    "usage: cargo run -p myosu-games-liars-dice --example liars_dice_policy_bundle -- --output <bundle.json> [--iterations <n>]"
}

fn write_json(
    path: impl Into<PathBuf>,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn Error>> {
    let path = path.into();
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;

    Ok(())
}
