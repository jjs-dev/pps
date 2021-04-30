mod compile;
mod import;
mod progress_notifier;

use anyhow::Context as _;
use clap::Clap;
use std::path::Path;

#[derive(Clap, Debug)]
#[clap(author, about)]
pub enum Args {
    Compile(compile::CompileArgs),
    Import(import::ImportArgs),
}

fn check_dir(path: &Path, allow_nonempty: bool) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("error: path {} not exists", path.display());
    }
    if !path.is_dir() {
        anyhow::bail!("error: path {} is not directory", path.display());
    }
    if !allow_nonempty && path.read_dir().unwrap().next().is_some() {
        anyhow::bail!("error: dir {} is not empty", path.display());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();
    process_args(args).await.context("failed to process args")?;
    Ok(())
}

#[tracing::instrument(skip(args))]
async fn process_args(args: Args) -> anyhow::Result<()> {
    tracing::info!(args=?args, "executing requested command");
    match args {
        Args::Compile(compile_args) => compile::exec(compile_args).await,
        Args::Import(import_args) => import::exec(import_args).await,
    }
}
