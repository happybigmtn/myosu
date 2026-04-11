use std::io;

use myosu_games_canonical::{
    PlaytracePolicy, PlaytraceRequest, canonical_ten_playtrace_requests,
    research_playtrace_requests, run_playtrace,
};
use myosu_games_portfolio::ResearchGame;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let requests = if args.all_canonical_ten {
        canonical_ten_playtrace_requests(args.max_steps, args.seed, args.policy)
    } else if args.all_research_games {
        research_playtrace_requests(args.max_steps, args.seed, args.policy)
    } else {
        vec![PlaytraceRequest {
            game: args.game.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "pass --game <slug>, --all-canonical-ten, or --all-research-games",
                )
            })?,
            max_steps: args.max_steps,
            seed: args.seed,
            policy: args.policy,
        }]
    };

    for request in requests {
        let report = run_playtrace(request)?;
        let payoff = report
            .payoff
            .as_ref()
            .map(|payoff| {
                payoff
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_else(|| "none".to_string());
        println!(
            "PLAYTRACE game={} status={} steps={} strategy_source={} terminal={} payoff={} truth_hash={}",
            report.game.slug(),
            report.status,
            report.steps,
            report.strategy_source,
            report.terminal,
            payoff,
            report.truth_hash
        );
    }

    Ok(())
}

struct Args {
    game: Option<ResearchGame>,
    all_canonical_ten: bool,
    all_research_games: bool,
    max_steps: usize,
    seed: u64,
    policy: PlaytracePolicy,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut game = None;
    let mut all_canonical_ten = false;
    let mut all_research_games = false;
    let mut max_steps = 200;
    let mut seed = 1;
    let mut policy = PlaytracePolicy::BestLocal;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => {
                let value = args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--game requires a slug")
                })?;
                game = Some(ResearchGame::parse(&value)?);
            }
            "--all-canonical-ten" => {
                all_canonical_ten = true;
            }
            "--all-research-games" => {
                all_research_games = true;
            }
            "--max-steps" => {
                let value = args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--max-steps requires a value")
                })?;
                max_steps = value.parse()?;
            }
            "--seed" => {
                let value = args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--seed requires a value")
                })?;
                seed = value.parse()?;
            }
            "--policy" => {
                let value = args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--policy requires a value")
                })?;
                policy = PlaytracePolicy::parse(&value)?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument `{arg}`"),
                )
                .into());
            }
        }
    }

    Ok(Args {
        game,
        all_canonical_ten,
        all_research_games,
        max_steps,
        seed,
        policy,
    })
}
