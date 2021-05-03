use anyhow::Context as _;
use pps_engine::{
    apis::compile::{CompileRequest, CompileUpdate},
    operation::Outcome,
};
use std::path::PathBuf;

#[derive(Debug, clap::Clap)]
pub struct CompileArgs {
    /// Path to problem package root
    #[clap(long = "pkg", short = 'P')]
    pub pkg_path: Vec<PathBuf>,
    /// Output path
    #[clap(long = "out", short = 'O')]
    pub out_path: Vec<PathBuf>,
    /// Rewrite dir
    #[clap(long, short = 'F')]
    pub force: bool,
}

#[tracing::instrument(skip(compile_args))]
pub async fn exec(compile_args: CompileArgs) -> anyhow::Result<()> {
    if compile_args.out_path.len() != compile_args.pkg_path.len() {
        anyhow::bail!("count(--pkg) != count(--out)");
    }
    let jjs_path = std::env::var_os("JJS_PATH").context("JJS_PATH environment variable missing")?;
    for (out_path, pkg_path) in compile_args.out_path.iter().zip(&compile_args.pkg_path) {
        let req = CompileRequest {
            out_path: out_path.clone(),
            problem_path: pkg_path.clone(),
            force: compile_args.force,
            jjs_path: jjs_path.clone().into(),
        };
        let mut op = pps_engine::apis::compile::exec(req);
        let mut notifier = None;
        while let Some(upd) = op.next_update().await {
            match upd {
                CompileUpdate::Warnings(warnings) => {
                    if !warnings.is_empty() {
                        eprintln!("{} warnings", warnings.len());
                        for warn in warnings {
                            eprintln!("- {}", warn);
                        }
                    }
                }
                CompileUpdate::BuildSolution(solution_name) => {
                    println!("Building solution {}", &solution_name);
                }
                CompileUpdate::BuildTestgen(testgen_name) => {
                    println!("Building generator {}", testgen_name);
                }
                CompileUpdate::BuildChecker => {
                    println!("Building checker");
                }
                CompileUpdate::GenerateTests { count } => {
                    notifier = Some(crate::progress_notifier::Notifier::new(count));
                }
                CompileUpdate::GenerateTest { test_id } => {
                    notifier
                        .as_mut()
                        .expect("GenerateTest received before GenerateTests")
                        .maybe_notify(test_id);
                }
                CompileUpdate::CopyValuerConfig => {
                    println!("Valuer config");
                }
            }
        }
        match op.outcome() {
            Outcome::Finish => {
                println!("Problem compiled successfully");
            }
            Outcome::Error(err) => {
                anyhow::bail!("compilation failed: {:#}", err,);
            }
            Outcome::Cancelled => {
                println!("Operation was cancelled");
            }
        }
    }
    Ok(())
}
