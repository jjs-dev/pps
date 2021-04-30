//! This module implements compiling source package into invoker package
pub(crate) mod build;
mod builder;

use crate::operation::{Operation, ProgressWriter};
use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct CompileRequest {
    /// Path to problem source directory
    pub problem_path: PathBuf,
    /// Where to put compiled package
    pub out_path: PathBuf,
    /// Ignore existing files in out_path
    pub force: bool,
    /// Path to directory containing JJS binaries (such as svaluer)
    pub jjs_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CompileUpdate {
    /// Contains some warnings that should be displayed to used.
    /// Appears at most once.
    Warnings(Vec<String>),
    /// Solution with given name is being built
    BuildSolution(String),
    /// Test generator with given name is being built
    BuildTestgen(String),
    /// Checker building started
    BuildChecker,
    /// Test generation started. `count` tests will be processed.
    /// Appears at most once before `GenerateTest` updates.
    GenerateTests { count: usize },
    /// Test `test_id` is being generated. Total test count is `count`.
    /// `test_id`s are in range 1..=`count`. It is gu
    GenerateTest { test_id: usize },
    /// Valuer config is being copied
    CopyValuerConfig,
}

async fn do_exec(
    req: CompileRequest,
    pw: &mut ProgressWriter<CompileUpdate>,
) -> anyhow::Result<()> {
    if req.force {
        tokio::fs::remove_dir_all(&req.out_path).await.ok();
        tokio::fs::create_dir_all(&req.out_path).await?;
    } else {
        crate::check_dir(&req.out_path, false /* TODO */).await?;
    }
    let toplevel_manifest = req.problem_path.join("problem.toml");
    let toplevel_manifest = tokio::fs::read_to_string(toplevel_manifest).await?;

    let raw_problem_cfg: crate::manifest::RawProblem =
        toml::from_str(&toplevel_manifest).context("problem.toml parse error")?;
    let (problem_cfg, warnings) = raw_problem_cfg.postprocess()?;

    pw.send(CompileUpdate::Warnings(warnings)).await;

    let out_dir = tokio::fs::canonicalize(&req.out_path)
        .await
        .context("resolve out dir")?;
    let problem_dir = tokio::fs::canonicalize(&req.problem_path)
        .await
        .context("resolve problem dir")?;

    let mut builder = builder::ProblemBuilder {
        cfg: &problem_cfg,
        problem_dir: &problem_dir,
        out_dir: &out_dir,
        build_env: &req.jjs_path,
        build_backend: &build::Pibs {
            jjs_dir: Path::new(&req.jjs_path),
        },
        pw,
    };
    builder.build().await?;
    Ok(())
}

/// Executes CompileRequest
pub fn exec(req: CompileRequest) -> Operation<CompileUpdate> {
    let (op, mut pw) = crate::operation::start();
    tokio::task::spawn(async move {
        let res = do_exec(req, &mut pw).await;
        pw.finish(res).await;
    });

    op
}
