use std::{io, path::PathBuf};

pub fn clone_remote_project(url: String, temp_dir: &PathBuf) -> io::Result<()> {
    std::process::Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(format!("{}/tmp", temp_dir.display()))
        .output()?;
    Ok(())
}
