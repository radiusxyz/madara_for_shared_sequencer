use std::path::PathBuf;

use sc_cli::RunCmd;

/// Available Sealing methods.
#[derive(Debug, Copy, Clone, clap::ValueEnum, Default)]
pub enum Sealing {
    // Seal using rpc method.
    #[default]
    Manual,
    // Seal when transaction is executed.
    Instant,
}

/// Available testnets.
#[derive(Debug, Copy, Clone, PartialEq, clap::ValueEnum)]
pub enum Testnet {
    Local,
    Sharingan,
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: ExtendedRunCmd,
    // /// Choose sealing method.
    // #[arg(long, value_enum, ignore_case = true)]
    // pub sealing: Option<Sealing>,
}

#[derive(Debug, clap::Args)]
pub struct ExtendedRunCmd {
    #[clap(flatten)]
    pub run_cmd: RunCmd,

    #[clap(long)]
    pub testnet: Option<Testnet>,

    #[clap(long)]
    pub madara_path: Option<PathBuf>,

    #[arg(long, value_enum, ignore_case = true)]
    pub sealing: Option<Sealing>,

    #[clap(long)]
    pub encrypted_mempool: bool,
}

#[derive(Debug)]
pub struct ExtendedConfiguration {
    pub sealing: Option<Sealing>,
    pub encrypted_mempool: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management cli utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
