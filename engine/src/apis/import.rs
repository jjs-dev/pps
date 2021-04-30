mod problem_importer;
mod template;
mod valuer_cfg;

use crate::operation::{Operation, ProgressWriter};
use anyhow::{bail, Context as _};
use problem_importer::Importer;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct ImportRequest {
    /// this path specifies file or files that should be imported
    pub src_path: PathBuf,
    /// where to put generated problem source
    pub out_path: PathBuf,
    /// do not check that dest is empty
    pub force: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ImportUpdate {
    /// Contains one property of discovered problem.
    /// Each `property_name` will be reported at most once.
    Property {
        property_name: PropertyName,
        property_value: String,
    },
    /// Contains one warnings. May appear multiple times.
    Warning(String),
    /// Started importing checker
    ImportChecker,
    /// Started importing tests
    ImportTests,
    /// Finished importing tests. `count` tests imported.
    ImportTestsDone { count: usize },
    /// Started importing solutions
    ImportSolutions,
    /// Started importing solution with specific name
    ImportSolution(String),
    /// Valuer config is detected and will be imported
    ImportValuerConfig,
    /// Valuer config was not found, default will be used
    DefaultValuerConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PropertyName {
    /// Value is time limit in milliseconds.
    TimeLimit,
    /// Value is memory limit in milliseconds.
    MemoryLimit,
    /// Value is printf-style pattern of input files.
    InputPathPattern,
    /// Value is printf-style pattern of output files.
    OutputPathPattern,
    /// Value is problem title.
    ProblemTitle,
}

/// Executes ImportRequest
pub fn exec(req: ImportRequest) -> Operation<ImportUpdate> {
    let (op, mut pw) = crate::operation::start();
    tokio::task::spawn(async move {
        let res = do_exec(req, &mut pw).await;
        pw.finish(res).await;
    });

    op
}

async fn do_exec(req: ImportRequest, tx: &mut ProgressWriter<ImportUpdate>) -> anyhow::Result<()> {
    match detect_import_kind(&req.src_path)? {
        ImportKind::Problem => (),
        ImportKind::Contest => anyhow::bail!("TODO: import contests"),
    }
    import_problem(&req.src_path, &req.out_path, tx).await?;

    Ok(())
}

async fn import_problem(
    src: &Path,
    dest: &Path,
    pw: &mut ProgressWriter<ImportUpdate>,
) -> anyhow::Result<()> {
    let manifest_path = src.join("problem.xml");
    let manifest = std::fs::read_to_string(manifest_path).context("failed read problem.xml")?;
    let doc = roxmltree::Document::parse(&manifest).context("parse error")?;

    let mut importer = Importer {
        src: &src,
        dest: &dest,
        problem_cfg: Default::default(),
        known_generators: HashSet::new(),
        doc: doc.root_element(),
        limits: pom::Limits::default(),
        pw,
    };

    importer.run().await?;

    let manifest_path = dest.join("problem.toml");
    let manifest_toml =
        toml::Value::try_from(importer.problem_cfg.clone()).context("serialize ppc config")?;
    let manifest_data = toml::ser::to_string_pretty(&manifest_toml)
        .with_context(|| format!("stringify ppc config: {:#?}", &importer.problem_cfg))?;
    std::fs::write(manifest_path, manifest_data).expect("write ppc manifest");

    Ok(())
}

enum ImportKind {
    Problem,
    Contest,
}

fn detect_import_kind(path: &Path) -> anyhow::Result<ImportKind> {
    if !path.exists() {
        bail!("path {} does not exists", path.display());
    }

    if path.join("problem.xml").exists() {
        return Ok(ImportKind::Problem);
    }
    if path.join("contest.xml").exists() {
        return Ok(ImportKind::Contest);
    }

    bail!("unknown src")
}
