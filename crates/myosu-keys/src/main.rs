use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use myosu_keys::{
    default_config_dir, export_active_keyfile, generate_mnemonic, import_keyfile,
    list_stored_accounts, load_active_pair, load_operator_config, save_mnemonic, save_pair,
    set_active_account,
};
use sp_core::{Pair, sr25519};

/// Minimal operator key-management CLI for the current stage-1 surface.
#[derive(Debug, Parser)]
#[command(name = "myosu-keys")]
#[command(about = "Create and inspect the current Myosu operator keystore")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate a new mnemonic, persist the encrypted seed, and set it active.
    Create(CreateArgs),
    /// Import a mnemonic phrase from an environment variable into the keystore.
    ImportMnemonic(ImportMnemonicArgs),
    /// Import a raw 32-byte seed hex string from an environment variable.
    ImportRawSeed(ImportRawSeedArgs),
    /// Import an encrypted Myosu keyfile into the operator config directory.
    ImportKeyfile(ImportKeyfileArgs),
    /// List the currently stored operator accounts for a config directory.
    List(ListArgs),
    /// Export the active encrypted Myosu keyfile to a destination path.
    ExportActiveKeyfile(ExportActiveKeyfileArgs),
    /// Print the current active account metadata without decrypting the key file.
    ShowActive(ShowActiveArgs),
    /// Switch the active configured account to a stored address.
    SwitchActive(SwitchActiveArgs),
    /// Re-encrypt the active operator key with a new password.
    ChangePassword(ChangePasswordArgs),
    /// Print miner and validator bootstrap commands for the active account.
    PrintBootstrap(PrintBootstrapArgs),
}

#[derive(Debug, Args)]
struct CreateArgs {
    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Environment variable containing the keystore password.
    #[arg(long, default_value = "MYOSU_KEY_PASSWORD")]
    password_env: String,

    /// Network label recorded in `config.toml`.
    #[arg(long, default_value = "devnet")]
    network: String,
}

#[derive(Debug, Args)]
struct ImportKeyfileArgs {
    /// Source path for an encrypted Myosu keyfile JSON.
    #[arg(long)]
    source: PathBuf,

    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Network label to use if `config.toml` does not exist yet.
    #[arg(long, default_value = "devnet")]
    network: String,
}

#[derive(Debug, Args)]
struct ImportMnemonicArgs {
    /// Environment variable containing the mnemonic phrase.
    #[arg(long)]
    mnemonic_env: String,

    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Environment variable containing the keystore password.
    #[arg(long, default_value = "MYOSU_KEY_PASSWORD")]
    password_env: String,

    /// Network label recorded in `config.toml`.
    #[arg(long, default_value = "devnet")]
    network: String,
}

#[derive(Debug, Args)]
struct ImportRawSeedArgs {
    /// Environment variable containing a 32-byte seed hex string.
    #[arg(long)]
    seed_env: String,

    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Environment variable containing the keystore password.
    #[arg(long, default_value = "MYOSU_KEY_PASSWORD")]
    password_env: String,

    /// Network label recorded in `config.toml`.
    #[arg(long, default_value = "devnet")]
    network: String,
}

#[derive(Debug, Args)]
struct ListArgs {
    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ExportActiveKeyfileArgs {
    /// Output path for the encrypted active keyfile JSON.
    #[arg(long)]
    output: PathBuf,

    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ShowActiveArgs {
    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct SwitchActiveArgs {
    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Stored SS58 address to mark active in `config.toml`.
    #[arg(long)]
    address: String,
}

#[derive(Debug, Args)]
struct ChangePasswordArgs {
    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Environment variable containing the current keystore password.
    #[arg(long, default_value = "MYOSU_KEY_PASSWORD")]
    old_password_env: String,

    /// Environment variable containing the replacement keystore password.
    #[arg(long)]
    new_password_env: String,
}

#[derive(Debug, Args)]
struct PrintBootstrapArgs {
    /// Operator config directory. Defaults to `~/.myosu`.
    #[arg(long)]
    config_dir: Option<PathBuf>,

    /// Environment variable holding the password for `--key-config-dir`.
    #[arg(long, default_value = "MYOSU_KEY_PASSWORD")]
    password_env: String,

    /// Chain WebSocket RPC endpoint to place in the printed commands.
    #[arg(long, default_value = "ws://127.0.0.1:9944")]
    chain: String,

    /// Subnet to place in the printed commands.
    #[arg(long)]
    subnet: u16,
}

#[derive(Debug)]
struct CreateResult {
    mnemonic: String,
    address: String,
    network: String,
    config_path: PathBuf,
    key_path: PathBuf,
}

#[derive(Debug)]
struct ActiveAccountSummary {
    address: String,
    network: String,
    config_path: PathBuf,
    key_path: PathBuf,
}

#[derive(Debug)]
struct ListedAccountSummary {
    address: String,
    key_path: PathBuf,
}

#[derive(Debug)]
struct ExportedKeyfileSummary {
    output_path: PathBuf,
}

#[derive(Debug)]
struct BootstrapPlan {
    summary: ActiveAccountSummary,
    password_env: String,
    miner_command: String,
    validator_command: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Create(args) => print_create_result(run_create(args)?),
        Command::ImportMnemonic(args) => print_active_summary(run_import_mnemonic(args)?),
        Command::ImportRawSeed(args) => print_active_summary(run_import_raw_seed(args)?),
        Command::ImportKeyfile(args) => print_active_summary(run_import_keyfile(args)?),
        Command::List(args) => print_listed_accounts(run_list(args)?),
        Command::ExportActiveKeyfile(args) => print_exported_keyfile(run_export_keyfile(args)?),
        Command::ShowActive(args) => print_active_summary(run_show_active(args)?),
        Command::SwitchActive(args) => print_active_summary(run_switch_active(args)?),
        Command::ChangePassword(args) => print_active_summary(run_change_password(args)?),
        Command::PrintBootstrap(args) => print_bootstrap_plan(run_print_bootstrap(args)?),
    }
    Ok(())
}

fn run_create(args: CreateArgs) -> Result<CreateResult> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    let password = load_password_from_env(&args.password_env)?;
    create_account(&config_dir, &password, &args.network)
}

fn run_import_mnemonic(args: ImportMnemonicArgs) -> Result<ActiveAccountSummary> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    let mnemonic = load_env_value(&args.mnemonic_env, "mnemonic phrase")?;
    let password = load_password_from_env(&args.password_env)?;
    import_mnemonic_account(&config_dir, &mnemonic, &password, &args.network)
}

fn run_import_raw_seed(args: ImportRawSeedArgs) -> Result<ActiveAccountSummary> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    let raw_seed = load_env_value(&args.seed_env, "raw seed hex")?;
    let password = load_password_from_env(&args.password_env)?;
    import_raw_seed_account(&config_dir, &raw_seed, &password, &args.network)
}

fn run_import_keyfile(args: ImportKeyfileArgs) -> Result<ActiveAccountSummary> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    import_encrypted_keyfile(&config_dir, &args.source, &args.network)
}

fn run_list(args: ListArgs) -> Result<Vec<ListedAccountSummary>> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    list_accounts(&config_dir)
}

fn run_export_keyfile(args: ExportActiveKeyfileArgs) -> Result<ExportedKeyfileSummary> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    export_keyfile(&config_dir, &args.output)
}

fn run_show_active(args: ShowActiveArgs) -> Result<ActiveAccountSummary> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    summarize_active_account(&config_dir)
}

fn run_switch_active(args: SwitchActiveArgs) -> Result<ActiveAccountSummary> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    switch_active_account(&config_dir, &args.address)
}

fn run_change_password(args: ChangePasswordArgs) -> Result<ActiveAccountSummary> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    let old_password = load_env_value(&args.old_password_env, "current keystore password")
        .with_context(|| format!("export {}", args.old_password_env))?;
    let new_password = load_env_value(&args.new_password_env, "replacement keystore password")
        .with_context(|| format!("export {}", args.new_password_env))?;
    change_active_password(&config_dir, &old_password, &new_password)
}

fn run_print_bootstrap(args: PrintBootstrapArgs) -> Result<BootstrapPlan> {
    let config_dir = resolve_config_dir(args.config_dir.as_deref())?;
    let summary = summarize_active_account(&config_dir)?;
    Ok(build_bootstrap_plan(
        summary,
        &config_dir,
        &args.password_env,
        &args.chain,
        args.subnet,
    ))
}

fn create_account(config_dir: &Path, password: &str, network: &str) -> Result<CreateResult> {
    let mnemonic = generate_mnemonic();
    let stored = save_mnemonic(config_dir, &mnemonic, password, network).with_context(|| {
        format!(
            "failed to create and persist an operator account under {}",
            config_dir.display()
        )
    })?;

    Ok(CreateResult {
        mnemonic,
        address: stored.address,
        network: stored.network,
        config_path: stored.config_path,
        key_path: stored.key_path,
    })
}

fn list_accounts(config_dir: &Path) -> Result<Vec<ListedAccountSummary>> {
    let accounts = list_stored_accounts(config_dir).with_context(|| {
        format!(
            "failed to list the stored operator accounts under {}",
            config_dir.display()
        )
    })?;

    Ok(accounts
        .into_iter()
        .map(|account| ListedAccountSummary {
            address: account.address,
            key_path: account.key_path,
        })
        .collect())
}

fn import_mnemonic_account(
    config_dir: &Path,
    mnemonic: &str,
    password: &str,
    network: &str,
) -> Result<ActiveAccountSummary> {
    save_mnemonic(config_dir, mnemonic, password, network).with_context(|| {
        format!(
            "failed to import the mnemonic-backed operator account into {}",
            config_dir.display()
        )
    })?;
    summarize_active_account(config_dir)
}

fn import_raw_seed_account(
    config_dir: &Path,
    raw_seed: &str,
    password: &str,
    network: &str,
) -> Result<ActiveAccountSummary> {
    let raw_seed = raw_seed.strip_prefix("0x").unwrap_or(raw_seed);
    let seed_bytes =
        hex::decode(raw_seed).context("raw seed hex must decode from a 32-byte hex string")?;
    let pair = sr25519::Pair::from_seed_slice(&seed_bytes)
        .context("raw seed hex must decode to a valid 32-byte sr25519 seed")?;
    save_pair(config_dir, &pair, password, network).with_context(|| {
        format!(
            "failed to import the raw-seed operator account into {}",
            config_dir.display()
        )
    })?;
    summarize_active_account(config_dir)
}

fn import_encrypted_keyfile(
    config_dir: &Path,
    source_path: &Path,
    network: &str,
) -> Result<ActiveAccountSummary> {
    import_keyfile(config_dir, source_path, network).with_context(|| {
        format!(
            "failed to import the encrypted keyfile {} into {}",
            source_path.display(),
            config_dir.display()
        )
    })?;
    summarize_active_account(config_dir)
}

fn summarize_active_account(config_dir: &Path) -> Result<ActiveAccountSummary> {
    let config = load_operator_config(config_dir).with_context(|| {
        format!(
            "failed to load the active operator config from {}",
            config_dir.display()
        )
    })?;
    let config_path = config_dir.join("config.toml");
    let key_path = config_dir.join("keys").join(&config.key_file);

    Ok(ActiveAccountSummary {
        address: config.active_account,
        network: config.network,
        config_path,
        key_path,
    })
}

fn export_keyfile(config_dir: &Path, output_path: &Path) -> Result<ExportedKeyfileSummary> {
    let output_path = export_active_keyfile(config_dir, output_path).with_context(|| {
        format!(
            "failed to export the active encrypted keyfile from {} to {}",
            config_dir.display(),
            output_path.display()
        )
    })?;
    Ok(ExportedKeyfileSummary { output_path })
}

fn change_active_password(
    config_dir: &Path,
    old_password: &str,
    new_password: &str,
) -> Result<ActiveAccountSummary> {
    let loaded = load_active_pair(config_dir, old_password).with_context(|| {
        format!(
            "failed to unlock the active operator account under {} with the current password",
            config_dir.display()
        )
    })?;
    save_pair(
        config_dir,
        &loaded.pair,
        new_password,
        &loaded.config.network,
    )
    .with_context(|| {
        format!(
            "failed to re-encrypt the active operator account under {} with the new password",
            config_dir.display()
        )
    })?;
    summarize_active_account(config_dir)
}

fn switch_active_account(config_dir: &Path, address: &str) -> Result<ActiveAccountSummary> {
    set_active_account(config_dir, address).with_context(|| {
        format!(
            "failed to switch the active operator account under {} to {}",
            config_dir.display(),
            address
        )
    })?;
    summarize_active_account(config_dir)
}

fn build_bootstrap_plan(
    summary: ActiveAccountSummary,
    config_dir: &Path,
    password_env: &str,
    chain: &str,
    subnet: u16,
) -> BootstrapPlan {
    let config_dir = config_dir.to_string_lossy();
    let miner_command =
        bootstrap_command("myosu-miner", chain, subnet, &config_dir, password_env);
    let validator_command =
        bootstrap_command("myosu-validator", chain, subnet, &config_dir, password_env);

    BootstrapPlan {
        summary,
        password_env: password_env.to_owned(),
        miner_command,
        validator_command,
    }
}

fn bootstrap_command(
    package: &str,
    chain: &str,
    subnet: u16,
    config_dir: &str,
    password_env: &str,
) -> String {
    format!(
        "cargo run -p {package} -- --chain {} --subnet {subnet} --key-config-dir {} \
         --key-password-env {password_env}",
        shell_quote(chain),
        shell_quote(config_dir),
    )
}

fn resolve_config_dir(config_dir: Option<&Path>) -> Result<PathBuf> {
    let Some(config_dir) = config_dir else {
        return default_config_dir()
            .context("HOME is required to resolve the default myosu config directory");
    };
    Ok(config_dir.to_path_buf())
}

fn load_password_from_env(password_env: &str) -> Result<String> {
    load_env_value(password_env, "keystore password").with_context(|| {
        format!(
            "export {password_env} before running a myosu-keys command that writes encrypted keys"
        )
    })
}

fn load_env_value(env_name: &str, purpose: &str) -> Result<String> {
    env::var(env_name)
        .with_context(|| format!("environment variable {env_name} is required for the {purpose}"))
}

fn print_create_result(result: CreateResult) {
    println!("Mnemonic: {}", result.mnemonic);
    println!("Address: {}", result.address);
    println!("Network: {}", result.network);
    println!("Config: {}", result.config_path.display());
    println!("Key File: {}", result.key_path.display());
    println!("Note: write this mnemonic down now; it is not stored on disk.");
}

fn print_active_summary(summary: ActiveAccountSummary) {
    println!("Active Address: {}", summary.address);
    println!("Network: {}", summary.network);
    println!("Config: {}", summary.config_path.display());
    println!("Key File: {}", summary.key_path.display());
}

fn print_listed_accounts(accounts: Vec<ListedAccountSummary>) {
    if accounts.is_empty() {
        println!("No stored accounts found.");
        return;
    }

    for account in accounts {
        println!("Address: {}", account.address);
        println!("Key File: {}", account.key_path.display());
    }
}

fn print_exported_keyfile(summary: ExportedKeyfileSummary) {
    println!("Exported Key File: {}", summary.output_path.display());
}

fn print_bootstrap_plan(plan: BootstrapPlan) {
    print_active_summary(plan.summary);
    println!(
        "Password Env Prep: export {}='replace-me'",
        plan.password_env
    );
    println!("Miner Command: {}", plan.miner_command);
    println!("Validator Command: {}", plan.validator_command);
}

fn shell_quote(value: &str) -> String {
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"-_./:=@".contains(&byte))
    {
        return value.to_owned();
    }

    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::{
        BootstrapPlan, ChangePasswordArgs, Cli, Command, CreateArgs, ExportActiveKeyfileArgs,
        ImportKeyfileArgs, ImportMnemonicArgs, ImportRawSeedArgs, ListArgs, PrintBootstrapArgs,
        ShowActiveArgs, SwitchActiveArgs, build_bootstrap_plan, change_active_password,
        create_account, export_keyfile, import_encrypted_keyfile, import_mnemonic_account,
        import_raw_seed_account, list_accounts, summarize_active_account, switch_active_account,
    };
    use clap::Parser;
    use myosu_keys::load_active_pair;
    use sp_core::{Pair, sr25519};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-keys-cli-{label}-{nanos}"))
    }

    #[test]
    fn cli_parses_create_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "create",
            "--config-dir",
            "/tmp/myosu",
            "--password-env",
            "MYOSU_PASSWORD",
            "--network",
            "test_finney",
        ]);

        let Command::Create(CreateArgs {
            config_dir,
            password_env,
            network,
        }) = cli.command
        else {
            panic!("expected create command");
        };

        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(password_env, "MYOSU_PASSWORD");
        assert_eq!(network, "test_finney");
    }

    #[test]
    fn cli_parses_import_mnemonic_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "import-mnemonic",
            "--mnemonic-env",
            "MYOSU_MNEMONIC",
            "--config-dir",
            "/tmp/myosu",
            "--password-env",
            "MYOSU_PASSWORD",
            "--network",
            "test_finney",
        ]);

        let Command::ImportMnemonic(ImportMnemonicArgs {
            mnemonic_env,
            config_dir,
            password_env,
            network,
        }) = cli.command
        else {
            panic!("expected import-mnemonic command");
        };

        assert_eq!(mnemonic_env, "MYOSU_MNEMONIC");
        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(password_env, "MYOSU_PASSWORD");
        assert_eq!(network, "test_finney");
    }

    #[test]
    fn cli_parses_import_raw_seed_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "import-raw-seed",
            "--seed-env",
            "MYOSU_RAW_SEED",
            "--config-dir",
            "/tmp/myosu",
            "--password-env",
            "MYOSU_PASSWORD",
            "--network",
            "test_finney",
        ]);

        let Command::ImportRawSeed(ImportRawSeedArgs {
            seed_env,
            config_dir,
            password_env,
            network,
        }) = cli.command
        else {
            panic!("expected import-raw-seed command");
        };

        assert_eq!(seed_env, "MYOSU_RAW_SEED");
        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(password_env, "MYOSU_PASSWORD");
        assert_eq!(network, "test_finney");
    }

    #[test]
    fn cli_parses_import_keyfile_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "import-keyfile",
            "--source",
            "/tmp/source.json",
            "--config-dir",
            "/tmp/myosu",
            "--network",
            "test_finney",
        ]);

        let Command::ImportKeyfile(ImportKeyfileArgs {
            source,
            config_dir,
            network,
        }) = cli.command
        else {
            panic!("expected import-keyfile command");
        };

        assert_eq!(source, std::path::PathBuf::from("/tmp/source.json"));
        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(network, "test_finney");
    }

    #[test]
    fn cli_parses_list_command() {
        let cli = Cli::parse_from(["myosu-keys", "list", "--config-dir", "/tmp/myosu"]);

        let Command::List(ListArgs { config_dir }) = cli.command else {
            panic!("expected list command");
        };

        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
    }

    #[test]
    fn cli_parses_export_active_keyfile_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "export-active-keyfile",
            "--output",
            "/tmp/exported.json",
            "--config-dir",
            "/tmp/myosu",
        ]);

        let Command::ExportActiveKeyfile(ExportActiveKeyfileArgs { output, config_dir }) =
            cli.command
        else {
            panic!("expected export-active-keyfile command");
        };

        assert_eq!(output, std::path::PathBuf::from("/tmp/exported.json"));
        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
    }

    #[test]
    fn cli_parses_show_active_command() {
        let cli = Cli::parse_from(["myosu-keys", "show-active", "--config-dir", "/tmp/myosu"]);

        let Command::ShowActive(ShowActiveArgs { config_dir }) = cli.command else {
            panic!("expected show-active command");
        };

        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
    }

    #[test]
    fn cli_parses_switch_active_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "switch-active",
            "--config-dir",
            "/tmp/myosu",
            "--address",
            "5F3sa2TJAWMqDhXG6jhV4N8ko9y7v5xz6N7YkK2sH1gJZ1kA",
        ]);

        let Command::SwitchActive(SwitchActiveArgs {
            config_dir,
            address,
        }) = cli.command
        else {
            panic!("expected switch-active command");
        };

        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(address, "5F3sa2TJAWMqDhXG6jhV4N8ko9y7v5xz6N7YkK2sH1gJZ1kA");
    }

    #[test]
    fn cli_parses_change_password_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "change-password",
            "--config-dir",
            "/tmp/myosu",
            "--old-password-env",
            "MYOSU_OLD_PASSWORD",
            "--new-password-env",
            "MYOSU_NEW_PASSWORD",
        ]);

        let Command::ChangePassword(ChangePasswordArgs {
            config_dir,
            old_password_env,
            new_password_env,
        }) = cli.command
        else {
            panic!("expected change-password command");
        };

        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(old_password_env, "MYOSU_OLD_PASSWORD");
        assert_eq!(new_password_env, "MYOSU_NEW_PASSWORD");
    }

    #[test]
    fn cli_parses_print_bootstrap_command() {
        let cli = Cli::parse_from([
            "myosu-keys",
            "print-bootstrap",
            "--config-dir",
            "/tmp/myosu",
            "--password-env",
            "MYOSU_PASSWORD",
            "--chain",
            "ws://10.0.0.5:9944",
            "--subnet",
            "9",
        ]);

        let Command::PrintBootstrap(PrintBootstrapArgs {
            config_dir,
            password_env,
            chain,
            subnet,
        }) = cli.command
        else {
            panic!("expected print-bootstrap command");
        };

        assert_eq!(
            config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(password_env, "MYOSU_PASSWORD");
        assert_eq!(chain, "ws://10.0.0.5:9944");
        assert_eq!(subnet, 9);
    }

    #[test]
    fn create_account_persists_and_sets_active_metadata() {
        let root = temp_config_dir("create");
        let created =
            create_account(&root, "hunter2", "devnet").expect("create should persist account");
        let active = summarize_active_account(&root).expect("active account should load");

        assert_eq!(active.address, created.address);
        assert_eq!(active.network, "devnet");
        assert_eq!(active.config_path, root.join("config.toml"));
        assert_eq!(active.key_path, created.key_path);
        assert_eq!(created.mnemonic.split_whitespace().count(), 12);
    }

    #[test]
    fn list_accounts_returns_stored_addresses() {
        let root = temp_config_dir("list");
        let first = create_account(&root, "hunter2", "devnet").expect("first create should work");
        let second = create_account(&root, "hunter2", "devnet").expect("second create should work");

        let listed = list_accounts(&root).expect("list should work");

        assert_eq!(listed.len(), 2);
        assert!(
            listed
                .iter()
                .any(|account| account.address == first.address)
        );
        assert!(
            listed
                .iter()
                .any(|account| account.address == second.address)
        );
    }

    #[test]
    fn switch_active_account_updates_summary() {
        let root = temp_config_dir("switch");
        let first = create_account(&root, "hunter2", "devnet").expect("first create should work");
        let second = create_account(&root, "hunter2", "devnet").expect("second create should work");

        let summary =
            switch_active_account(&root, &first.address).expect("switch active should work");

        assert_eq!(summary.address, first.address);
        assert_eq!(summary.network, "devnet");
        assert_ne!(summary.address, second.address);
    }

    #[test]
    fn import_encrypted_keyfile_sets_active_summary() {
        let source_root = temp_config_dir("import-source");
        let dest_root = temp_config_dir("import-dest");
        let created = create_account(&source_root, "hunter2", "devnet").expect("create works");

        let summary = import_encrypted_keyfile(&dest_root, &created.key_path, "test_finney")
            .expect("import should work");

        assert_eq!(summary.address, created.address);
        assert_eq!(summary.network, "test_finney");
    }

    #[test]
    fn export_keyfile_writes_destination() {
        let root = temp_config_dir("export");
        create_account(&root, "hunter2", "devnet").expect("create should work");
        let output = root.join("copied-key.json");

        let exported = export_keyfile(&root, &output).expect("export should work");

        assert_eq!(exported.output_path, output);
        assert!(exported.output_path.exists());
    }

    #[test]
    fn import_mnemonic_account_sets_active_summary() {
        let root = temp_config_dir("import-mnemonic");
        let created = create_account(&temp_config_dir("source"), "hunter2", "devnet")
            .expect("source create should work");
        let summary = import_mnemonic_account(&root, &created.mnemonic, "hunter2", "test_finney")
            .expect("mnemonic import should work");

        assert_eq!(summary.address, created.address);
        assert_eq!(summary.network, "test_finney");
    }

    #[test]
    fn import_raw_seed_account_sets_active_summary() {
        let root = temp_config_dir("import-seed");
        let created = create_account(&temp_config_dir("seed-source"), "hunter2", "devnet")
            .expect("source create should work");
        let (_, seed) =
            sr25519::Pair::from_phrase(&created.mnemonic, None).expect("created mnemonic is valid");
        let seed_hex = format!("0x{}", hex::encode(seed.as_ref()));

        let summary = import_raw_seed_account(&root, &seed_hex, "hunter2", "test_finney")
            .expect("raw-seed import should work");

        assert_eq!(summary.address, created.address);
        assert_eq!(summary.network, "test_finney");
    }

    #[test]
    fn change_active_password_reencrypts_active_key() {
        let root = temp_config_dir("change-password");
        let created = create_account(&root, "hunter2", "devnet").expect("create should work");

        let summary = change_active_password(&root, "hunter2", "new-hunter2")
            .expect("password change should work");
        let old_error = load_active_pair(&root, "hunter2");
        let loaded = load_active_pair(&root, "new-hunter2").expect("new password should work");

        assert_eq!(summary.address, created.address);
        assert!(old_error.is_err());
        assert_eq!(loaded.config.active_account, created.address);
    }

    #[test]
    fn bootstrap_plan_prints_active_operator_commands() {
        let root = temp_config_dir("print-bootstrap");
        create_account(&root, "hunter2", "devnet").expect("create should work");
        let summary = summarize_active_account(&root).expect("summary should load");

        let plan = build_bootstrap_plan(summary, &root, "MYOSU_PASSWORD", "ws://10.0.0.5:9944", 9);

        assert_bootstrap_plan(&plan, &root);
    }

    fn assert_bootstrap_plan(plan: &BootstrapPlan, config_dir: &std::path::Path) {
        assert_eq!(plan.password_env, "MYOSU_PASSWORD");
        assert!(plan.miner_command.contains("cargo run -p myosu-miner --"));
        assert!(plan.miner_command.contains("--chain ws://10.0.0.5:9944"));
        assert!(plan.miner_command.contains("--subnet 9"));
        assert!(
            plan.miner_command
                .contains(&config_dir.to_string_lossy().into_owned())
        );
        assert!(
            plan.miner_command
                .contains("--key-password-env MYOSU_PASSWORD")
        );
        assert!(
            plan.validator_command
                .contains("cargo run -p myosu-validator --")
        );
        assert!(plan.validator_command.contains("--subnet 9"));
    }
}
