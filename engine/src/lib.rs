pub mod apis;
mod command;
mod manifest;
pub mod operation;

use std::path::Path;

async fn check_dir(path: &Path, allow_nonempty: bool) -> anyhow::Result<()> {
    let meta = match tokio::fs::metadata(path).await {
        Ok(m) => m,
        Err(err) => anyhow::bail!(
            "error: path {} not exists or not available: {:#}",
            path.display(),
            err
        ),
    };
    if !meta.is_dir() {
        anyhow::bail!("error: path {} is not directory", path.display());
    }
    if !allow_nonempty {
        let mut iter = tokio::fs::read_dir(path).await?;
        let it = iter.next_entry().await?;
        if it.is_some() {
            anyhow::bail!("error: dir {} is not empty", path.display());
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[tracing::instrument]
fn tune_linux() -> anyhow::Result<()> {
    let mut current_limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    unsafe {
        if libc::prlimit(0, libc::RLIMIT_STACK, std::ptr::null(), &mut current_limit) != 0 {
            anyhow::bail!("get current RLIMIT_STACK");
        }
    }
    let new_limit = libc::rlimit {
        rlim_cur: current_limit.rlim_max,
        rlim_max: current_limit.rlim_max,
    };
    unsafe {
        if libc::prlimit(0, libc::RLIMIT_STACK, &new_limit, std::ptr::null_mut()) != 0 {
            anyhow::bail!("update RLIMIT_STACK");
        }
    }

    Ok(())
}

#[tracing::instrument]
fn tune_resource_limits() -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    tune_linux()?;

    Ok(())
}
