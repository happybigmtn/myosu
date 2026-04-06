use std::env;
use std::error::Error;
use std::io;
use std::time::Duration;

use myosu_chain_client::ChainClient;

fn usage_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "usage: cargo run -p myosu-chain-client --example fund_account -- <endpoint> <signer-uri> <dest-ss58> <amount>",
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(endpoint) = args.next() else {
        return Err(usage_error().into());
    };
    let Some(signer_uri) = args.next() else {
        return Err(usage_error().into());
    };
    let Some(dest_ss58) = args.next() else {
        return Err(usage_error().into());
    };
    let Some(amount) = args.next() else {
        return Err(usage_error().into());
    };
    if args.next().is_some() {
        return Err(usage_error().into());
    }

    let amount: u64 = amount.parse()?;
    let dest = ChainClient::account_id_from_ss58(&dest_ss58)?;
    let client = ChainClient::connect(&endpoint).await?;
    let report = client
        .transfer_keep_alive(&signer_uri, &dest, amount, Duration::from_secs(60))
        .await?;

    println!("TRANSFER myosu-chain-client keep-alive ok");
    println!("from={}", report.signer);
    println!("dest={}", report.dest);
    println!("value={}", report.value);
    println!("extrinsic_hash={}", report.extrinsic_hash);
    println!("inclusion_block={}", report.inclusion_block);

    Ok(())
}
