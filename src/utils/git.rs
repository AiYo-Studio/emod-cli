use std::path::PathBuf;
use crate::error::Result;

pub fn clone_remote_project(url: String, temp_dir: &PathBuf) -> Result<()> {
    std::process::Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(format!("{}/tmp", temp_dir.display()))
        .output()?;
    Ok(())
}