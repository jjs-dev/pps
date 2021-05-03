use anyhow::Context;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone)]
pub(crate) struct Task {
    /// Directory with source files, or path to single file
    pub(crate) src: PathBuf,
    /// Directory for build artifacts
    pub(crate) dest: PathBuf,
    /// Directort for temporary data
    pub(crate) tmp: PathBuf,
}

pub(crate) struct TaskSuccess {
    pub(crate) command: crate::command::Command,
}

#[derive(Debug, Error)]
#[error("child command {} errored: code {:?}", _0, _1.status.code())]
pub(crate) struct ExitCodeNonZeroError(pub(crate) String, pub(crate) std::process::Output);

#[derive(Debug, Error)]
enum TaskErrors {
    //#[error("child command {} errored: code {:?}", _0, _1.status.code())]
    //ExitCodeNonZero(String, std::process::Output),
    #[error("feature not supported: {feature}")]
    FeatureNotSupported { feature: &'static str },
}

impl Task {
    fn multi_file(&self) -> bool {
        self.src.is_dir()
    }
}

#[async_trait::async_trait]
trait CommandExt {
    async fn run(&mut self) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl CommandExt for tokio::process::Command {
    async fn run(&mut self) -> anyhow::Result<()> {
        let out = self.output().await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(ExitCodeNonZeroError(format!("{:?}", self), out).into())
        }
    }
}

#[async_trait::async_trait]
pub(crate) trait BuildBackend: Send + Sync {
    async fn process_task(&self, task: Task) -> anyhow::Result<TaskSuccess>;
}

/// Ppc-integrated build system
pub(crate) struct Pibs<'a> {
    pub(crate) jjs_dir: &'a Path,
}

impl<'a> Pibs<'a> {
    async fn process_cmake_task(&self, task: Task) -> anyhow::Result<TaskSuccess> {
        tokio::process::Command::new("cmake")
            .arg("-S")
            .arg(&task.src)
            .arg("-B")
            .arg(&task.tmp)
            .run()
            .await?;

        tokio::process::Command::new("cmake")
            .arg("--build")
            .arg(&task.tmp)
            .run()
            .await?;

        let dst = task.dest.join("bin");
        let src = task.tmp.join("Out");
        tokio::fs::copy(&src, &dst)
            .await
            .with_context(|| format!("failed to copy {} to {}", src.display(), dst.display()))?;
        let run_cmd = crate::command::Command::new(dst);
        Ok(TaskSuccess { command: run_cmd })
    }
}

#[async_trait::async_trait]
impl<'a> BuildBackend for Pibs<'a> {
    async fn process_task(&self, task: Task) -> anyhow::Result<TaskSuccess> {
        if task.multi_file() {
            let cmake_lists_path = task.src.join("CMakeLists.txt");
            if cmake_lists_path.exists() {
                return self.process_cmake_task(task).await;
            }
            let python_path = task.src.join("main.py");
            if python_path.exists() {
                let out_path = task.dest.join("out.py");
                tokio::fs::copy(&python_path, &out_path)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to copy {} to {}",
                            python_path.display(),
                            out_path.display()
                        )
                    })?;
                let mut command = crate::command::Command::new("python3");
                command.arg(&out_path);
                return Ok(TaskSuccess { command });
            }
            return Err(TaskErrors::FeatureNotSupported {
                feature: "multi-file sources",
            }
            .into());
        }

        let incl_arg = format!("-I{}/include", self.jjs_dir.display());
        let link_arg = format!("-L{}/lib", self.jjs_dir.display());

        let dest_file = task.dest.join("bin");
        tokio::process::Command::new("g++")
            .arg("-std=c++17")
            .arg(incl_arg)
            .arg(link_arg)
            .arg("-DPPC=1")
            .arg(task.src)
            .arg("-o")
            .arg(&dest_file)
            .arg("-ljtl")
            .arg("-lpthread")
            .arg("-ldl")
            .run()
            .await?;

        let command = crate::command::Command::new(&dest_file);
        Ok(TaskSuccess { command })
    }
}
