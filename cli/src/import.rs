use anyhow::Context as _;
use pps_engine::{
    apis::import::{ImportRequest, ImportUpdate, PropertyName},
    operation::Outcome,
};
use std::path::{Path, PathBuf};

#[derive(clap::Clap, Debug)]
pub struct ImportArgs {
    /// Path to package being imported
    #[clap(long = "in", short = 'I')]
    pub in_path: PathBuf,
    /// Out path (will contain pps package)
    #[clap(long = "out", short = 'O')]
    pub out_path: PathBuf,
    /// Rewrite dir
    #[clap(long, short = 'F')]
    pub force: bool,
    /// Imported contest name
    /// This option can only be used when importing contest
    #[clap(long, short = 'N')]
    pub contest_name: Option<String>,
}

async fn import_one_problem(src: &Path, dest: &Path, force: bool) -> anyhow::Result<()> {
    let import_req = ImportRequest {
        src_path: src.to_path_buf(),
        out_path: dest.to_path_buf(),
        force,
    };
    let mut op = pps_engine::apis::import::exec(import_req);
    while let Some(upd) = op.next_update().await {
        match upd {
            ImportUpdate::Property {
                property_name,
                property_value,
            } => match property_name {
                PropertyName::TimeLimit => println!("Time limit: {} ms", property_value),
                PropertyName::MemoryLimit => {
                    let ml = property_value.parse::<u64>()?;
                    println!("Memory limit: {} bytes ({} MiBs)", ml, ml / (1 << 20));
                }
                PropertyName::InputPathPattern => {
                    println!("Test input file path pattern: {}", property_value)
                }
                PropertyName::OutputPathPattern => {
                    println!("Test output file path pattern: {}", property_value)
                }
                PropertyName::ProblemTitle => println!("Problem title: {}", property_value),
            },
            ImportUpdate::Warning(warning) => eprintln!("warning: {}", warning),
            ImportUpdate::ImportChecker => println!("Importing checker"),
            ImportUpdate::ImportTests => println!("Importing tests"),
            ImportUpdate::ImportTestsDone { count } => println!("{} tests imported", count),
            ImportUpdate::ImportSolutions => println!("Importing solutions"),
            ImportUpdate::ImportSolution(solution) => println!("Importing solution '{}'", solution),
            ImportUpdate::ImportValuerConfig => println!("Importing valuer config"),
            ImportUpdate::DefaultValuerConfig => println!("Defaulting valuer config"),
        }
    }
    match op.outcome() {
        Outcome::Finish => {
            println!("Problem imported successfully");
        }
        Outcome::Error(err) => {
            anyhow::bail!("import failed: {:#}", err);
        }
        Outcome::Cancelled => {
            println!("Operation was cancelled");
        }
    }
    Ok(())
}

#[tracing::instrument(skip(args))]
pub(crate) async fn exec(args: ImportArgs) -> anyhow::Result<()> {
    if args.force {
        std::fs::remove_dir_all(&args.out_path).ok();
        std::fs::create_dir(&args.out_path).context("create out dir")?;
    } else {
        crate::check_dir(&PathBuf::from(&args.out_path), false /* TODO */)?;
    }

    let src = &args.in_path;
    let dest = &args.out_path;

    import_one_problem(src, dest, args.force).await?;

    // TODO support importing contests

    Ok(())
}
