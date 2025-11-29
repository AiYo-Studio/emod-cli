use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde_json::Value;
use walkdir;
use zip::write::SimpleFileOptions;

use crate::commands::ReleaseArgs;
use crate::utils::file;
use crate::{entity, entity::project::ReleaseInfo};
use crate::error::Result;

pub fn execute(args: &ReleaseArgs) {
    if let Err(e) = run_release(args) {
        eprintln!("❌ 组件打包失败: {}", e);
        return;
    }
    println!("🍀 组件打包完成");
}

fn run_release(args: &ReleaseArgs) -> Result<()> {
    let project_dir = file::find_project_dir(&args.path)?;
    let release_info = entity::get_current_release_info(&project_dir)?;
    
    println!("🔖 当前行为包版本: {:?}", release_info.behavior_version);
    println!("🔖 当前资源包版本: {:?}", release_info.resource_version);
    
    release(&args.ver, &project_dir, &release_info)?;
    
    Ok(())
}

fn release(
    version: &Option<String>,
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
) -> Result<()> {
    let new_version = calculate_version(version, &release_info.behavior_version)?;
    let version_value = Value::Array(new_version.iter().map(|v| Value::from(*v)).collect());
    
    println!("📦 开始打包, 版本号: {:?}", &new_version);
    
    update_versions(&project_dir, &release_info, &version_value)?;
    
    let version_str = format!("{}.{}.{}", new_version[0], new_version[1], new_version[2]);
    let output_path = package_project(&project_dir, &release_info, &version_str)?;
    
    println!("📦 打包完成: {}", output_path.replace("\\", "/"));
    Ok(())
}

fn calculate_version(version: &Option<String>, current: &[u32]) -> Result<Vec<u32>> {
    if let Some(ver_str) = version {
        ver_str
            .split(".")
            .map(|s| s.parse::<u32>().map_err(|e| e.into()))
            .collect()
    } else {
        Ok(vec![current[0], current[1], current[2] + 1])
    }
}

fn update_versions(
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
    version: &Value,
) -> Result<()> {
    update_pack_json(&project_dir, &version)?;
    update_manifest_json(&project_dir, &release_info, &version)?;
    Ok(())
}

fn update_pack_json(project_dir: &PathBuf, version: &Value) -> Result<()> {
    let paths = vec![
        project_dir.join("world_behavior_packs.json"),
        project_dir.join("world_resource_packs.json"),
    ];
    
    for path in paths {
        file::update_json_file(&path, |json| {
            json[0]["version"] = version.clone();
            Ok(())
        })?;
    }
    
    Ok(())
}

fn update_manifest_json(
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
    version: &Value,
) -> Result<()> {
    let behavior_dir = project_dir.join(format!(
        "behavior_pack_{}",
        release_info.behavior_identifier
    ));
    let resource_dir = project_dir.join(format!(
        "resource_pack_{}",
        release_info.resource_identifier
    ));
    
    for pack_dir in [behavior_dir, resource_dir] {
        let manifest_path = pack_dir.join("pack_manifest.json");
        file::update_json_file(&manifest_path, |json| {
            json["header"]["version"] = version.clone();
            json["modules"][0]["version"] = version.clone();
            Ok(())
        })?;
    }
    
    Ok(())
}

fn package_project(
    project_dir: &PathBuf,
    release_info: &ReleaseInfo,
    version: &str,
) -> Result<String> {
    let output_path = format!("{}/release_{}.zip", project_dir.display(), version);
    let file = fs::File::create(&output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    
    let behavior_dir = project_dir.join(format!(
        "behavior_pack_{}",
        release_info.behavior_identifier
    ));
    let resource_dir = project_dir.join(format!(
        "resource_pack_{}",
        release_info.resource_identifier
    ));
    
    add_directory_to_zip(&mut zip, &project_dir, &behavior_dir)?;
    add_directory_to_zip(&mut zip, &project_dir, &resource_dir)?;
    
    zip.finish()?;
    Ok(output_path)
}

fn add_directory_to_zip(
    zip: &mut zip::ZipWriter<File>,
    project_dir: &PathBuf,
    src_dir: &PathBuf,
) -> Result<()> {
    if !src_dir.is_dir() {
        return Err(crate::error::CliError::InvalidData(
            format!("{} 不是目录", src_dir.display())
        ));
    }
    
    if count_files(src_dir)? == 0 {
        return Ok(());
    }
    
    let options = SimpleFileOptions::default();
    let mut buffer = Vec::new();
    
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(project_dir)
            .map_err(|e| crate::error::CliError::InvalidData(e.to_string()))?;
        
        let path_str = relative_path
            .to_str()
            .ok_or_else(|| crate::error::CliError::InvalidData(
                format!("{:?} 不是有效的 UTF-8 路径", relative_path)
            ))?;
        
        if path.is_file() {
            if path_str.ends_with(".gitkeep") {
                continue;
            }
            zip.start_file(path_str, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !relative_path.as_os_str().is_empty() && count_files(path)? > 0 {
            zip.add_directory(path_str, options)?;
        }
    }
    
    Ok(())
}

fn count_files(dir: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.path().is_file() 
            && !entry.path().display().to_string().ends_with(".gitkeep") {
            count += 1;
        }
    }
    Ok(count)
}
