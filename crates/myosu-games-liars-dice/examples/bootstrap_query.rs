use std::env;
use std::fs;
use std::path::PathBuf;

use myosu_games::CfrGame;
use myosu_games_liars_dice::{
    LiarsDiceEdge, LiarsDiceGame, LiarsDiceStrategyQuery, encode_strategy_query,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(query_file) = args.next() else {
        return Err(
            "usage: cargo run -p myosu-games-liars-dice --example bootstrap_query -- <query-file>"
                .into(),
        );
    };
    if args.next().is_some() {
        return Err("expected exactly one positional argument".into());
    }

    let query_file = PathBuf::from(query_file);
    let opening = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 });
    let info = opening
        .info()
        .ok_or("opening player turn should expose info")?;
    let query = LiarsDiceStrategyQuery::new(info);
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
