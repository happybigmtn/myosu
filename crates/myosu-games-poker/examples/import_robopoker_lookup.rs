use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::stdin;
use std::path::PathBuf;

use myosu_games_poker::write_encoder_dir_from_lookup_dump;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(input) = args.next() else {
        return Err(usage().into());
    };
    let Some(encoder_dir) = args.next() else {
        return Err(usage().into());
    };
    if args.next().is_some() {
        return Err("expected exactly two positional arguments".into());
    }

    let encoder_dir = PathBuf::from(encoder_dir);
    let manifest = if input == "-" {
        let stdin = stdin();
        let reader = BufReader::new(stdin.lock());
        write_encoder_dir_from_lookup_dump(reader, &encoder_dir)?
    } else {
        let reader = BufReader::new(File::open(&input)?);
        write_encoder_dir_from_lookup_dump(reader, &encoder_dir)?
    };

    println!("LOOKUP_IMPORT encoder_dir={}", encoder_dir.display());
    println!("LOOKUP_IMPORT total_sha256={}", manifest.total_sha256);
    println!("LOOKUP_IMPORT streets={}", manifest.streets.len());

    Ok(())
}

fn usage() -> &'static str {
    "usage: cargo run -p myosu-games-poker --example import_robopoker_lookup -- <lookup-tsv|- for stdin> <encoder-dir>"
}
