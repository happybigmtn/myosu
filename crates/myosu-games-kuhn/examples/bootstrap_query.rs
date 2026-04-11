use std::env;
use std::fs;
use std::path::PathBuf;

use myosu_games::CfrGame;
use myosu_games_kuhn::{KuhnCard, KuhnEdge, KuhnGame, KuhnStrategyQuery, encode_strategy_query};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(query_file) = args.next() else {
        return Err(
            "usage: cargo run -p myosu-games-kuhn --example bootstrap_query -- <query-file>".into(),
        );
    };
    if args.next().is_some() {
        return Err("expected exactly one positional argument".into());
    }

    let query_file = PathBuf::from(query_file);
    let opening = KuhnGame::root().apply(KuhnEdge::Deal {
        p1: KuhnCard::King,
        p2: KuhnCard::Queen,
    });
    let info = opening
        .info()
        .ok_or("opening player turn should expose info")?;
    let query = KuhnStrategyQuery::new(info);
    let query_bytes = encode_strategy_query(&query)?;
    let query_parent = query_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&query_parent)?;
    fs::write(&query_file, query_bytes)?;

    println!("BOOTSTRAP query_file={}", query_file.display());

    Ok(())
}
