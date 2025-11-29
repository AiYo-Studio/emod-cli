use crate::error::Result;
use serde_json::Value;
use std::{fs, path::PathBuf};

pub fn copy_folder(src: &PathBuf, dest: &PathBuf) -> Result<()> {
    if !src.exists() || !src.is_dir() {
        return Err(crate::error::CliError::NotFound(format!(
            "源目录不存在: {}",
            src.display()
        )));
    }
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(src_path.file_name().unwrap());
        if src_path.is_file() {
            fs::copy(&src_path, &dest_path)?;
        } else if src_path.is_dir() {
            copy_folder(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

pub fn read_file_to_json(path: &PathBuf) -> Result<Value> {
    let file = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&file)?;
    Ok(json)
}

pub fn write_json_to_file(path: &PathBuf, value: &Value) -> Result<()> {
    let content = serde_json::to_string_pretty(value)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn update_json_file<F>(path: &PathBuf, updater: F) -> Result<()>
where
    F: FnOnce(&mut Value) -> Result<()>,
{
    let mut json = read_file_to_json(path)?;
    updater(&mut json)?;
    write_json_to_file(path, &json)?;
    Ok(())
}

pub fn find_project_dir(path: &Option<String>) -> Result<PathBuf> {
    let path = path.as_deref().unwrap_or(".");
    Ok(PathBuf::from(path))
}
